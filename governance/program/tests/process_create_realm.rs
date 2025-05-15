#![cfg(feature = "test-sbf")]

use solana_program_test::*;

mod program_test;

use {
    crate::program_test::args::DabraSetupArgs,
    program_test::*,
    spl_governance::state::{enums::MintMaxVoterWeightSource, dabra::get_dabra_address},
};

#[tokio::test]
async fn test_create_dabra() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_new().await;

    // Act
    let dabra_cookie = governance_test.with_dabra().await;

    // Assert
    let dabra_account = governance_test
        .get_dabra_account(&dabra_cookie.address)
        .await;

    assert_eq!(dabra_cookie.account, dabra_account);
}

#[tokio::test]
async fn test_create_dabra_with_non_default_config() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let dabra_setup_args = DabraSetupArgs {
        use_council_mint: false,
        community_mint_max_voter_weight_source: MintMaxVoterWeightSource::SupplyFraction(1),
        min_community_weight_to_create_governance: 1,
        ..Default::default()
    };

    // Act
    let dabra_cookie = governance_test
        .with_dabra_using_args(&dabra_setup_args)
        .await;

    // Assert
    let dabra_account = governance_test
        .get_dabra_account(&dabra_cookie.address)
        .await;

    assert_eq!(dabra_cookie.account, dabra_account);
}

#[tokio::test]
async fn test_create_dabra_with_max_voter_weight_absolute_value() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let dabra_setup_args = DabraSetupArgs {
        community_mint_max_voter_weight_source: MintMaxVoterWeightSource::Absolute(1),
        ..Default::default()
    };

    // Act
    let dabra_cookie = governance_test
        .with_dabra_using_args(&dabra_setup_args)
        .await;

    // Assert
    let dabra_account = governance_test
        .get_dabra_account(&dabra_cookie.address)
        .await;

    assert_eq!(dabra_cookie.account, dabra_account);
    assert_eq!(
        dabra_cookie
            .account
            .config
            .community_mint_max_voter_weight_source,
        MintMaxVoterWeightSource::Absolute(1)
    );
}

#[tokio::test]
async fn test_create_dabra_for_existing_pda() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let dabra_name = format!("Dabra #{}", governance_test.next_dabra_id).to_string();
    let dabra_address = get_dabra_address(&governance_test.program_id, &dabra_name);

    let rent_exempt = governance_test.bench.rent.minimum_balance(0);

    governance_test
        .bench
        .transfer_sol(&dabra_address, rent_exempt)
        .await;

    // Act
    let dabra_cookie = governance_test.with_dabra().await;

    // Assert
    let dabra_account = governance_test
        .get_dabra_account(&dabra_cookie.address)
        .await;

    assert_eq!(dabra_cookie.account, dabra_account);
}
