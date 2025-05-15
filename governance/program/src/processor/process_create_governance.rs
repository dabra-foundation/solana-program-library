//! Program state processor

use {
    crate::{
        state::{
            enums::GovernanceAccountType,
            governance::{
                assert_valid_create_governance_args, get_governance_address_seeds,
                GovernanceConfig, GovernanceV2,
            },
            dabra::get_dabra_data,
        },
        tools::structs::Reserved119,
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

/// Processes CreateGovernance instruction
pub fn process_create_governance(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    config: GovernanceConfig,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let dabra_info = next_account_info(account_info_iter)?; // 0
    let governance_info = next_account_info(account_info_iter)?; // 1
    let governance_seed_info = next_account_info(account_info_iter)?; // 2

    let token_owner_record_info = next_account_info(account_info_iter)?; // 3

    let payer_info = next_account_info(account_info_iter)?; // 4
    let system_info = next_account_info(account_info_iter)?; // 5

    let rent = Rent::get()?;

    let create_authority_info = next_account_info(account_info_iter)?; // 6

    assert_valid_create_governance_args(program_id, &config, dabra_info)?;

    let dabra_data = get_dabra_data(program_id, dabra_info)?;

    dabra_data.assert_create_authority_can_create_governance(
        program_id,
        dabra_info.key,
        token_owner_record_info,
        create_authority_info,
        account_info_iter, // dabra_config_info 7, voter_weight_record_info 8
    )?;

    let governance_data = GovernanceV2 {
        account_type: GovernanceAccountType::GovernanceV2,
        dabra: *dabra_info.key,
        governance_seed: *governance_seed_info.key,
        config,
        reserved1: 0,
        reserved_v2: Reserved119::default(),
        required_signatories_count: 0,
        active_proposal_count: 0,
    };

    create_and_serialize_account_signed::<GovernanceV2>(
        payer_info,
        governance_info,
        &governance_data,
        &get_governance_address_seeds(dabra_info.key, governance_seed_info.key),
        program_id,
        system_info,
        &rent,
        0,
    )?;

    Ok(())
}
