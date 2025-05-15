//! Program state processor

use {
    crate::{
        error::GovernanceError,
        state::{
            dabra::{
                assert_valid_dabra_config_args, get_dabra_data_for_authority, DabraConfigArgs,
            },
            dabra_config::{get_dabra_config_data_for_dabra, resolve_governing_token_config},
        },
    },
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        pubkey::Pubkey,
        rent::Rent,
        sysvar::Sysvar,
    },
};

/// Processes SetDabraConfig instruction
pub fn process_set_dabra_config(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    dabra_config_args: DabraConfigArgs,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let dabra_info = next_account_info(account_info_iter)?; // 0
    let dabra_authority_info = next_account_info(account_info_iter)?; // 1

    let mut dabra_data =
        get_dabra_data_for_authority(program_id, dabra_info, dabra_authority_info.key)?;

    if !dabra_authority_info.is_signer {
        return Err(GovernanceError::DabraAuthorityMustSign.into());
    }

    // Note: Config change leaves voting proposals in unpredictable state and it's
    // DAOs responsibility to ensure the changes are made when there are no
    // proposals in voting state For example changing voter-weight or
    // max-voter-weight addin could accidentally make proposals to succeed which
    // would otherwise be defeated

    assert_valid_dabra_config_args(&dabra_config_args)?;

    // Setup council
    if dabra_config_args.use_council_mint {
        let council_token_mint_info = next_account_info(account_info_iter)?; // 2
        let _council_token_holding_info = next_account_info(account_info_iter)?; // 3

        // Council mint can only be at present set to None (removed) and changing it to
        // other mint is not supported It might be implemented in future
        // versions but it needs careful planning It can potentially open a can
        // of warms like what happens with existing deposits or pending proposals
        if let Some(council_token_mint) = dabra_data.config.council_mint {
            // Council mint can't be changed to different one
            if council_token_mint != *council_token_mint_info.key {
                return Err(GovernanceError::DabraCouncilMintChangeIsNotSupported.into());
            }
        } else {
            // Council mint can't be restored (changed from None)
            return Err(GovernanceError::DabraCouncilMintChangeIsNotSupported.into());
        }
    } else {
        // Remove council mint from dabra
        // Note: In the current implementation this also makes it impossible to withdraw
        // council tokens
        dabra_data.config.council_mint = None;
    }

    let system_info = next_account_info(account_info_iter)?; // 4

    let dabra_config_info = next_account_info(account_info_iter)?; // 5
    let mut dabra_config_data =
        get_dabra_config_data_for_dabra(program_id, dabra_config_info, dabra_info.key)?;

    dabra_config_data.assert_can_change_config(&dabra_config_args)?;

    // Setup configs for tokens (plugins and token types)

    // 6, 7
    let community_token_config = resolve_governing_token_config(
        account_info_iter,
        &dabra_config_args.community_token_config_args,
        Some(dabra_config_data.community_token_config.clone()),
    )?;

    // 8, 9
    let council_token_config = resolve_governing_token_config(
        account_info_iter,
        &dabra_config_args.council_token_config_args,
        Some(dabra_config_data.council_token_config.clone()),
    )?;

    dabra_config_data.community_token_config = community_token_config;
    dabra_config_data.council_token_config = council_token_config;

    let payer_info = next_account_info(account_info_iter)?; // 10
    let rent = Rent::get()?;

    dabra_config_data.serialize(
        program_id,
        dabra_config_info,
        payer_info,
        system_info,
        &rent,
    )?;

    // Update DabraConfig (Dabra.config field)
    dabra_data.config.community_mint_max_voter_weight_source =
        dabra_config_args.community_mint_max_voter_weight_source;

    dabra_data.config.min_community_weight_to_create_governance =
        dabra_config_args.min_community_weight_to_create_governance;

    dabra_data.config.legacy1 = 0;
    dabra_data.config.legacy2 = 0;

    dabra_data.serialize(&mut dabra_info.data.borrow_mut()[..])?;

    Ok(())
}
