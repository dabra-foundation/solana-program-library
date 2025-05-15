#![cfg(feature = "test-sbf")]

use {solana_program::pubkey::Pubkey, solana_program_test::*};

mod program_test;

use {crate::program_test::args::DabraSetupArgs, program_test::*};

#[tokio::test]
async fn test_create_dabra_with_voter_weight_addin() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_with_voter_weight_addin().await;

    let mut dabra_setup_args = DabraSetupArgs::default();

    dabra_setup_args
        .community_token_config_args
        .voter_weight_addin = governance_test.voter_weight_addin_id;

    // Act

    let dabra_cookie = governance_test
        .with_dabra_using_args(&dabra_setup_args)
        .await;

    // Assert

    let dabra_config_data = governance_test
        .get_dabra_config_account(&dabra_cookie.dabra_config.address)
        .await;

    assert_eq!(dabra_cookie.dabra_config.account, dabra_config_data);

    assert!(dabra_config_data
        .community_token_config
        .voter_weight_addin
        .is_some());
}

#[tokio::test]
async fn test_set_dabra_voter_weight_addin_for_dabra_without_addins() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_with_voter_weight_addin().await;

    let mut dabra_setup_args = DabraSetupArgs::default();
    dabra_setup_args
        .community_token_config_args
        .voter_weight_addin = None;

    let mut dabra_cookie = governance_test
        .with_dabra_using_args(&dabra_setup_args)
        .await;

    dabra_setup_args
        .community_token_config_args
        .voter_weight_addin = governance_test.voter_weight_addin_id;

    // Act

    governance_test
        .set_dabra_config(&mut dabra_cookie, &dabra_setup_args)
        .await
        .unwrap();

    // Assert

    let dabra_config_data = governance_test
        .get_dabra_config_account(&dabra_cookie.dabra_config.address)
        .await;

    assert_eq!(dabra_cookie.dabra_config.account, dabra_config_data);

    assert!(dabra_config_data
        .community_token_config
        .voter_weight_addin
        .is_some());
}

#[tokio::test]
async fn test_set_dabra_voter_weight_addin_for_dabra_without_council_and_addins() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_with_voter_weight_addin().await;

    let mut dabra_setup_args = DabraSetupArgs {
        use_council_mint: false,
        ..Default::default()
    };

    let mut dabra_cookie = governance_test
        .with_dabra_using_args(&dabra_setup_args)
        .await;

    dabra_setup_args
        .community_token_config_args
        .voter_weight_addin = governance_test.voter_weight_addin_id;

    // Act

    governance_test
        .set_dabra_config(&mut dabra_cookie, &dabra_setup_args)
        .await
        .unwrap();

    // Assert

    let dabra_config_data = governance_test
        .get_dabra_config_account(&dabra_cookie.dabra_config.address)
        .await;

    assert_eq!(dabra_cookie.dabra_config.account, dabra_config_data);

    assert!(dabra_config_data
        .community_token_config
        .voter_weight_addin
        .is_some());
}

#[tokio::test]
async fn test_set_dabra_voter_weight_addin_for_dabra_with_existing_voter_weight_addin() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_with_voter_weight_addin().await;

    let mut dabra_setup_args = DabraSetupArgs::default();

    dabra_setup_args
        .community_token_config_args
        .voter_weight_addin = governance_test.voter_weight_addin_id;

    let mut dabra_cookie = governance_test
        .with_dabra_using_args(&dabra_setup_args)
        .await;

    let community_voter_weight_addin_address = Pubkey::new_unique();
    dabra_setup_args
        .community_token_config_args
        .voter_weight_addin = Some(community_voter_weight_addin_address);

    // Act

    governance_test
        .set_dabra_config(&mut dabra_cookie, &dabra_setup_args)
        .await
        .unwrap();

    // Assert

    let dabra_config_data = governance_test
        .get_dabra_config_account(&dabra_cookie.dabra_config.address)
        .await;

    assert_eq!(dabra_cookie.dabra_config.account, dabra_config_data);
    assert_eq!(
        dabra_config_data.community_token_config.voter_weight_addin,
        Some(community_voter_weight_addin_address)
    );

    assert!(dabra_config_data
        .community_token_config
        .voter_weight_addin
        .is_some());
}

#[tokio::test]
async fn test_set_dabra_config_with_no_voter_weight_addin_for_dabra_without_addins() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_with_voter_weight_addin().await;

    let mut dabra_setup_args = DabraSetupArgs::default();

    dabra_setup_args
        .community_token_config_args
        .voter_weight_addin = None;

    let mut dabra_cookie = governance_test
        .with_dabra_using_args(&dabra_setup_args)
        .await;

    dabra_setup_args
        .community_token_config_args
        .voter_weight_addin = None;

    // Act

    governance_test
        .set_dabra_config(&mut dabra_cookie, &dabra_setup_args)
        .await
        .unwrap();

    // Assert

    let dabra_config_data = governance_test
        .get_dabra_config_account(&dabra_cookie.dabra_config.address)
        .await;

    assert!(dabra_config_data
        .community_token_config
        .voter_weight_addin
        .is_none());
}

#[tokio::test]
async fn test_set_dabra_config_with_no_voter_weight_addin_for_dabra_with_existing_addin() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_with_voter_weight_addin().await;
    let mut dabra_cookie = governance_test.with_dabra().await;

    let dabra_setup_args = DabraSetupArgs::default();

    // Act

    governance_test
        .set_dabra_config(&mut dabra_cookie, &dabra_setup_args)
        .await
        .unwrap();

    // Assert

    let dabra_config_data = governance_test
        .get_dabra_config_account(&dabra_cookie.dabra_config.address)
        .await;

    assert!(dabra_config_data
        .community_token_config
        .voter_weight_addin
        .is_none());
}
