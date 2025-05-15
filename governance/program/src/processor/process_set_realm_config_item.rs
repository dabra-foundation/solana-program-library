//! Program state processor

use {
    crate::{
        error::GovernanceError,
        state::{
            dabra::{get_dabra_data_for_authority, SetDabraConfigItemArgs},
            dabra_config::get_dabra_config_data_for_dabra,
        },
        tools::structs::SetConfigItemActionType,
    },
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        pubkey::Pubkey,
        rent::Rent,
        sysvar::Sysvar,
    },
};

/// Processes SetDabraConfigItem instruction
pub fn process_set_dabra_config_item(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    args: SetDabraConfigItemArgs,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let dabra_info = next_account_info(account_info_iter)?; // 0
    let dabra_config_info = next_account_info(account_info_iter)?; // 1
    let dabra_authority_info = next_account_info(account_info_iter)?; // 2
    let payer_info = next_account_info(account_info_iter)?; // 3
    let system_info = next_account_info(account_info_iter)?; // 4

    let rent = Rent::get()?;

    let dabra_data =
        get_dabra_data_for_authority(program_id, dabra_info, dabra_authority_info.key)?;

    if !dabra_authority_info.is_signer {
        return Err(GovernanceError::DabraAuthorityMustSign.into());
    }

    let mut dabra_config_data =
        get_dabra_config_data_for_dabra(program_id, dabra_config_info, dabra_info.key)?;

    match args {
        SetDabraConfigItemArgs::TokenOwnerRecordLockAuthority {
            action,
            governing_token_mint,
            authority,
        } => {
            let token_config =
                dabra_config_data.get_token_config_mut(&dabra_data, &governing_token_mint)?;

            match action {
                SetConfigItemActionType::Add => {
                    if token_config.lock_authorities.contains(&authority) {
                        return Err(
                            GovernanceError::TokenOwnerRecordLockAuthorityAlreadyExists.into()
                        );
                    }

                    token_config.lock_authorities.push(authority);
                }
                SetConfigItemActionType::Remove => {
                    if let Some(lock_authority_index) = token_config
                        .lock_authorities
                        .iter()
                        .position(|lock_authority| lock_authority == &authority)
                    {
                        token_config.lock_authorities.remove(lock_authority_index);
                    } else {
                        return Err(GovernanceError::TokenOwnerRecordLockAuthorityNotFound.into());
                    }
                }
            }
        }
    }

    dabra_config_data.serialize(
        program_id,
        dabra_config_info,
        payer_info,
        system_info,
        &rent,
    )?;

    Ok(())
}
