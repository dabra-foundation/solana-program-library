//! Program state processor

use {
    crate::{
        error::GovernanceError,
        state::{
            governance::assert_governance_for_dabra,
            dabra::{get_dabra_data_for_authority, SetDabraAuthorityAction},
        },
    },
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        pubkey::Pubkey,
    },
};

/// Processes SetDabraAuthority instruction
pub fn process_set_dabra_authority(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    action: SetDabraAuthorityAction,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let dabra_info = next_account_info(account_info_iter)?; // 0
    let dabra_authority_info = next_account_info(account_info_iter)?; // 1

    let mut dabra_data =
        get_dabra_data_for_authority(program_id, dabra_info, dabra_authority_info.key)?;

    if !dabra_authority_info.is_signer {
        return Err(GovernanceError::DabraAuthorityMustSign.into());
    }

    let new_dabra_authority = match action {
        SetDabraAuthorityAction::SetUnchecked | SetDabraAuthorityAction::SetChecked => {
            let new_dabra_authority_info = next_account_info(account_info_iter)?; // 2

            if action == SetDabraAuthorityAction::SetChecked {
                // Ensure the new dabra authority is one of the governances from the dabra
                assert_governance_for_dabra(program_id, new_dabra_authority_info, dabra_info.key)?;
            }

            Some(*new_dabra_authority_info.key)
        }
        SetDabraAuthorityAction::Remove => None,
    };

    dabra_data.authority = new_dabra_authority;

    dabra_data.serialize(&mut dabra_info.data.borrow_mut()[..])?;

    Ok(())
}
