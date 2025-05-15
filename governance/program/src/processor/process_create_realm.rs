//! Program state processor

use {
    crate::{
        error::GovernanceError,
        state::{
            enums::GovernanceAccountType,
            dabra::{
                assert_valid_dabra_config_args, get_governing_token_holding_address_seeds,
                get_dabra_address_seeds, DabraConfig, DabraConfigArgs, DabraV2,
            },
            dabra_config::{
                get_dabra_config_address_seeds, resolve_governing_token_config, DabraConfigAccount,
            },
        },
        tools::{spl_token::create_spl_token_account_signed, structs::Reserved110},
    },
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        pubkey::Pubkey,
        rent::Rent,
        sysvar::Sysvar,
    },
    spl_governance_tools::account::create_and_serialize_account_signed,
};

/// Processes CreateDabra instruction
pub fn process_create_dabra(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    name: String,
    dabra_config_args: DabraConfigArgs,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let dabra_info = next_account_info(account_info_iter)?; // 0
    let dabra_authority_info = next_account_info(account_info_iter)?; // 1
    let governance_token_mint_info = next_account_info(account_info_iter)?; // 2
    let governance_token_holding_info = next_account_info(account_info_iter)?; // 3
    let payer_info = next_account_info(account_info_iter)?; // 4
    let system_info = next_account_info(account_info_iter)?; // 5
    let spl_token_info = next_account_info(account_info_iter)?; // 6

    let rent_sysvar_info = next_account_info(account_info_iter)?; // 7
    let rent = &Rent::from_account_info(rent_sysvar_info)?;

    if !dabra_info.data_is_empty() {
        return Err(GovernanceError::DabraAlreadyExists.into());
    }

    assert_valid_dabra_config_args(&dabra_config_args)?;

    // Create Community token holding account
    create_spl_token_account_signed(
        payer_info,
        governance_token_holding_info,
        &get_governing_token_holding_address_seeds(dabra_info.key, governance_token_mint_info.key),
        governance_token_mint_info,
        dabra_info,
        program_id,
        system_info,
        spl_token_info,
        rent_sysvar_info,
        rent,
    )?;

    // Create Council token holding account
    let council_token_mint_address = if dabra_config_args.use_council_mint {
        let council_token_mint_info = next_account_info(account_info_iter)?; // 8
        let council_token_holding_info = next_account_info(account_info_iter)?; // 9

        create_spl_token_account_signed(
            payer_info,
            council_token_holding_info,
            &get_governing_token_holding_address_seeds(dabra_info.key, council_token_mint_info.key),
            council_token_mint_info,
            dabra_info,
            program_id,
            system_info,
            spl_token_info,
            rent_sysvar_info,
            rent,
        )?;

        Some(*council_token_mint_info.key)
    } else {
        None
    };

    // Create and serialize DabraConfig
    let dabra_config_info = next_account_info(account_info_iter)?; // 10

    // 11, 12
    let community_token_config = resolve_governing_token_config(
        account_info_iter,
        &dabra_config_args.community_token_config_args,
        None,
    )?;

    // 13, 14
    let council_token_config = resolve_governing_token_config(
        account_info_iter,
        &dabra_config_args.council_token_config_args,
        None,
    )?;

    let dabra_config_data = DabraConfigAccount {
        account_type: GovernanceAccountType::DabraConfig,
        dabra: *dabra_info.key,
        community_token_config,
        council_token_config,
        reserved: Reserved110::default(),
    };

    create_and_serialize_account_signed::<DabraConfigAccount>(
        payer_info,
        dabra_config_info,
        &dabra_config_data,
        &get_dabra_config_address_seeds(dabra_info.key),
        program_id,
        system_info,
        rent,
        0,
    )?;

    // Create and serialize Dabra
    let dabra_data = DabraV2 {
        account_type: GovernanceAccountType::DabraV2,
        community_mint: *governance_token_mint_info.key,

        name: name.clone(),
        reserved: [0; 6],
        authority: Some(*dabra_authority_info.key),
        config: DabraConfig {
            council_mint: council_token_mint_address,
            reserved: [0; 6],
            community_mint_max_voter_weight_source: dabra_config_args
                .community_mint_max_voter_weight_source,
            min_community_weight_to_create_governance: dabra_config_args
                .min_community_weight_to_create_governance,
            legacy1: 0,
            legacy2: 0,
        },
        legacy1: 0,
        reserved_v2: [0; 128],
    };

    create_and_serialize_account_signed::<DabraV2>(
        payer_info,
        dabra_info,
        &dabra_data,
        &get_dabra_address_seeds(&name),
        program_id,
        system_info,
        rent,
        0,
    )?;

    Ok(())
}
