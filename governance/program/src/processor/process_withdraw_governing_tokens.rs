//! Program state processor

use {
    crate::{
        error::GovernanceError,
        state::{
            dabra::{get_dabra_address_seeds, get_dabra_data},
            dabra_config::get_dabra_config_data_for_dabra,
            token_owner_record::{
                get_token_owner_record_address_seeds, get_token_owner_record_data_for_seeds,
            },
        },
        tools::spl_token::{get_spl_token_mint, transfer_spl_tokens_signed},
    },
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        clock::Clock,
        entrypoint::ProgramResult,
        pubkey::Pubkey,
        sysvar::Sysvar,
    },
};

/// Processes WithdrawGoverningTokens instruction
pub fn process_withdraw_governing_tokens(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let dabra_info = next_account_info(account_info_iter)?; // 0
    let governing_token_holding_info = next_account_info(account_info_iter)?; // 1
    let governing_token_destination_info = next_account_info(account_info_iter)?; // 2
    let governing_token_owner_info = next_account_info(account_info_iter)?; // 3
    let token_owner_record_info = next_account_info(account_info_iter)?; // 4
    let spl_token_info = next_account_info(account_info_iter)?; // 5
    let dabra_config_info = next_account_info(account_info_iter)?; // 6
    let clock = Clock::get()?;

    if !governing_token_owner_info.is_signer {
        return Err(GovernanceError::GoverningTokenOwnerMustSign.into());
    }

    let dabra_data = get_dabra_data(program_id, dabra_info)?;
    let governing_token_mint = get_spl_token_mint(governing_token_holding_info)?;

    dabra_data.assert_is_valid_governing_token_mint_and_holding(
        program_id,
        dabra_info.key,
        &governing_token_mint,
        governing_token_holding_info.key,
    )?;

    let dabra_config_data =
        get_dabra_config_data_for_dabra(program_id, dabra_config_info, dabra_info.key)?;

    dabra_config_data.assert_can_withdraw_governing_token(&dabra_data, &governing_token_mint)?;

    let token_owner_record_address_seeds = get_token_owner_record_address_seeds(
        dabra_info.key,
        &governing_token_mint,
        governing_token_owner_info.key,
    );

    let mut token_owner_record_data = get_token_owner_record_data_for_seeds(
        program_id,
        token_owner_record_info,
        &token_owner_record_address_seeds,
    )?;

    token_owner_record_data.assert_can_withdraw_governing_tokens(clock.unix_timestamp)?;

    transfer_spl_tokens_signed(
        governing_token_holding_info,
        governing_token_destination_info,
        dabra_info,
        &get_dabra_address_seeds(&dabra_data.name),
        program_id,
        token_owner_record_data.governing_token_deposit_amount,
        spl_token_info,
    )?;

    token_owner_record_data.governing_token_deposit_amount = 0;
    token_owner_record_data.serialize(&mut token_owner_record_info.data.borrow_mut()[..])?;

    Ok(())
}
