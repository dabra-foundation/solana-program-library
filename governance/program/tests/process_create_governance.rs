#![cfg(feature = "test-sbf")]
mod program_test;

use {
    crate::program_test::args::DabraSetupArgs,
    program_test::*,
    solana_program_test::*,
    solana_sdk::signature::Keypair,
    spl_governance::{error::GovernanceError, state::enums::VoteThreshold},
    spl_governance_tools::error::GovernanceToolsError,
};

#[tokio::test]
async fn test_create_governance() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let dabra_cookie = governance_test.with_dabra().await;

    let token_owner_record_cookie = governance_test
        .with_community_token_deposit(&dabra_cookie)
        .await
        .unwrap();

    // Act
    let governance_cookie = governance_test
        .with_governance(&dabra_cookie, &token_owner_record_cookie)
        .await
        .unwrap();

    // Assert
    let governance_account = governance_test
        .get_governance_account(&governance_cookie.address)
        .await;

    assert_eq!(governance_cookie.account, governance_account);
}

#[tokio::test]
async fn test_create_governance_with_invalid_dabra_error() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let mut dabra_cookie = governance_test.with_dabra().await;

    let token_owner_record_cookie = governance_test
        .with_community_token_deposit(&dabra_cookie)
        .await
        .unwrap();

    let governance_cookie = governance_test
        .with_governance(&dabra_cookie, &token_owner_record_cookie)
        .await
        .unwrap();

    dabra_cookie.address = governance_cookie.address;

    // Act
    let err = governance_test
        .with_governance(&dabra_cookie, &token_owner_record_cookie)
        .await
        .err()
        .unwrap();

    // Assert

    assert_eq!(err, GovernanceToolsError::InvalidAccountType.into());
}

#[tokio::test]
async fn test_create_governance_with_invalid_config_error() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let dabra_cookie = governance_test.with_dabra().await;

    let token_owner_record_cookie = governance_test
        .with_community_token_deposit(&dabra_cookie)
        .await
        .unwrap();

    // Arrange
    let mut config = governance_test.get_default_governance_config();
    config.community_vote_threshold = VoteThreshold::YesVotePercentage(0); // below 1% threshold

    // Act
    let err = governance_test
        .with_governance_using_config(&dabra_cookie, &token_owner_record_cookie, &config)
        .await
        .err()
        .unwrap();

    // Assert

    assert_eq!(err, GovernanceError::InvalidVoteThresholdPercentage.into());

    // Arrange
    let mut config = governance_test.get_default_governance_config();
    config.community_vote_threshold = VoteThreshold::YesVotePercentage(101); // Above 100% threshold

    // Act
    let err = governance_test
        .with_governance_using_config(&dabra_cookie, &token_owner_record_cookie, &config)
        .await
        .err()
        .unwrap();

    // Assert

    assert_eq!(err, GovernanceError::InvalidVoteThresholdPercentage.into());
}

#[tokio::test]
async fn test_create_governance_with_not_enough_community_tokens_error() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let dabra_cookie = governance_test.with_dabra().await;

    // Set token deposit amount below the required threshold
    let token_amount = 4;

    let token_owner_record_cookie = governance_test
        .with_community_token_deposit_amount(&dabra_cookie, token_amount)
        .await
        .unwrap();

    // Act
    let err = governance_test
        .with_governance(&dabra_cookie, &token_owner_record_cookie)
        .await
        .err()
        .unwrap();

    // Assert
    assert_eq!(
        err,
        GovernanceError::NotEnoughTokensToCreateGovernance.into()
    );
}

#[tokio::test]
async fn test_create_governance_with_not_enough_council_tokens_error() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let dabra_cookie = governance_test.with_dabra().await;

    // Set token deposit amount below the required threshold
    let token_amount: u64 = 0;

    let token_owner_record_cookie = governance_test
        .with_council_token_deposit_amount(&dabra_cookie, token_amount)
        .await
        .unwrap();

    // Act
    let err = governance_test
        .with_governance(&dabra_cookie, &token_owner_record_cookie)
        .await
        .err()
        .unwrap();

    // Assert
    assert_eq!(
        err,
        GovernanceError::NotEnoughTokensToCreateGovernance.into()
    );
}

#[tokio::test]
async fn test_create_governance_using_dabra_authority() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let dabra_cookie = governance_test.with_dabra().await;

    let config = governance_test.get_default_governance_config();
    let dabra_authority = dabra_cookie.dabra_authority.as_ref().unwrap();

    // Act
    let governance_cookie = governance_test
        .with_governance_impl(&dabra_cookie, None, dabra_authority, None, &config, None)
        .await
        .unwrap();

    // Assert
    let governance_account = governance_test
        .get_governance_account(&governance_cookie.address)
        .await;

    assert_eq!(governance_cookie.account, governance_account);
}

#[tokio::test]
async fn test_create_governance_using_dabra_authority_with_authority_must_sign_error() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let dabra_cookie = governance_test.with_dabra().await;

    let config = governance_test.get_default_governance_config();
    let dabra_authority = dabra_cookie.dabra_authority.as_ref().unwrap();

    // Act
    let err = governance_test
        .with_governance_impl(
            &dabra_cookie,
            None,
            dabra_authority,
            None,
            &config,
            Some(&[]),
        )
        .await
        .err()
        .unwrap();

    // Assert
    assert_eq!(err, GovernanceError::DabraAuthorityMustSign.into());
}

#[tokio::test]
async fn test_create_governance_using_dabra_authority_with_wrong_authority_sign_error() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let dabra_cookie = governance_test.with_dabra().await;

    let token_owner_record_cookie = governance_test
        .with_community_token_deposit(&dabra_cookie)
        .await
        .unwrap();

    let config = governance_test.get_default_governance_config();
    let authority = Keypair::new();

    // Act
    let err = governance_test
        .with_governance_impl(
            &dabra_cookie,
            Some(&token_owner_record_cookie.address),
            &authority,
            None,
            &config,
            Some(&[&authority]),
        )
        .await
        .err()
        .unwrap();

    // Assert
    assert_eq!(
        err,
        GovernanceError::GoverningTokenOwnerOrDelegateMustSign.into()
    );
}

#[tokio::test]
async fn test_create_governance_with_community_disabled_error() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let dabra_config_args = DabraSetupArgs {
        min_community_weight_to_create_governance: u64::MAX,
        ..Default::default()
    };

    let dabra_cookie = governance_test
        .with_dabra_using_args(&dabra_config_args)
        .await;

    // Set token deposit amount to max
    let token_amount = u64::MAX;

    let token_owner_record_cookie = governance_test
        .with_community_token_deposit_amount(&dabra_cookie, token_amount)
        .await
        .unwrap();

    // Act
    let err = governance_test
        .with_governance(&dabra_cookie, &token_owner_record_cookie)
        .await
        .err()
        .unwrap();

    // Assert
    assert_eq!(err, GovernanceError::VoterWeightThresholdDisabled.into());
}
