//! Program state processor

use {
    crate::{
        error::GovernanceError,
        state::{
            dabra::get_dabra_data, dabra_config::get_dabra_config_data_for_dabra,
            token_owner_record::get_token_owner_record_data_for_dabra,
        },
    },
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        clock::Clock,
        entrypoint::ProgramResult,
        pubkey::Pubkey,
        sysvar::Sysvar,
    },
};

/// Processes RelinquishTokenOwnerRecordLocks instruction
pub fn process_relinquish_token_owner_record_locks(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    lock_ids: Option<Vec<u8>>,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let dabra_info = next_account_info(account_info_iter)?; // 0
    let dabra_config_info = next_account_info(account_info_iter)?; // 1
    let token_owner_record_info = next_account_info(account_info_iter)?; // 2

    let dabra_data = get_dabra_data(program_id, dabra_info)?;
    let dabra_config_data =
        get_dabra_config_data_for_dabra(program_id, dabra_config_info, dabra_info.key)?;

    let mut token_owner_record_data = get_token_owner_record_data_for_dabra(
        program_id,
        token_owner_record_info,
        &dabra_config_data.dabra,
    )?;

    if let Some(lock_ids) = lock_ids {
        let token_owner_record_lock_authority_info = next_account_info(account_info_iter)?; // 3

        if dabra_config_data
            .get_token_config(&dabra_data, &token_owner_record_data.governing_token_mint)?
            .lock_authorities
            .contains(token_owner_record_lock_authority_info.key)
        {
            // If the authority is a configured lock authority it must sign the transaction
            if !token_owner_record_lock_authority_info.is_signer {
                return Err(GovernanceError::TokenOwnerRecordLockAuthorityMustSign.into());
            }
        }

        // Remove the locks
        for lock_id in lock_ids {
            token_owner_record_data
                .remove_lock(lock_id, token_owner_record_lock_authority_info.key)?;
        }
    }

    // Trim expired locks
    let clock = Clock::get()?;
    token_owner_record_data.remove_expired_locks(clock.unix_timestamp);

    token_owner_record_data.serialize(&mut token_owner_record_info.data.borrow_mut()[..])?;

    Ok(())
}
