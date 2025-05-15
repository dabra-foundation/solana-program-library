#![cfg(feature = "test-sbf")]

use {solana_program::pubkey::Pubkey, solana_program_test::*, solana_sdk::signer::Signer};

mod program_test;

use {
    crate::program_test::args::DabraSetupArgs,
    program_test::*,
    spl_governance::{
        error::GovernanceError,
        state::{dabra::GoverningTokenConfigAccountArgs, dabra_config::GoverningTokenType},
    },
};

#[tokio::test]
async fn test_set_dabra_config() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let mut dabra_cookie = governance_test.with_dabra().await;

    let dabra_setup_args = DabraSetupArgs::default();

    // Act

    governance_test
        .set_dabra_config(&mut dabra_cookie, &dabra_setup_args)
        .await
        .unwrap();

    // Assert
    let dabra_account = governance_test
        .get_dabra_account(&dabra_cookie.address)
        .await;

    assert_eq!(dabra_cookie.account, dabra_account);
}

#[tokio::test]
async fn test_set_dabra_config_with_authority_must_sign_error() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let mut dabra_cookie = governance_test.with_dabra().await;

    let dabra_setup_args = DabraSetupArgs::default();

    // Act

    let err = governance_test
        .set_dabra_config_using_instruction(
            &mut dabra_cookie,
            &dabra_setup_args,
            |i| i.accounts[1].is_signer = false,
            Some(&[]),
        )
        .await
        .err()
        .unwrap();

    // Assert
    assert_eq!(err, GovernanceError::DabraAuthorityMustSign.into());
}

#[tokio::test]
async fn test_set_dabra_config_with_no_authority_error() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let mut dabra_cookie = governance_test.with_dabra().await;

    let dabra_setup_args = DabraSetupArgs::default();

    governance_test
        .set_dabra_authority(&dabra_cookie, None)
        .await
        .unwrap();

    // Act

    let err = governance_test
        .set_dabra_config_using_instruction(
            &mut dabra_cookie,
            &dabra_setup_args,
            |i| i.accounts[1].is_signer = false,
            Some(&[]),
        )
        .await
        .err()
        .unwrap();

    // Assert
    assert_eq!(err, GovernanceError::DabraHasNoAuthority.into());
}

#[tokio::test]
async fn test_set_dabra_config_with_invalid_authority_error() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let mut dabra_cookie = governance_test.with_dabra().await;

    let dabra_setup_args = DabraSetupArgs::default();

    let dabra_cookie2 = governance_test.with_dabra().await;

    // Try to use authority from other dabra
    dabra_cookie.dabra_authority = dabra_cookie2.dabra_authority;

    // Act

    let err = governance_test
        .set_dabra_config(&mut dabra_cookie, &dabra_setup_args)
        .await
        .err()
        .unwrap();

    // Assert
    assert_eq!(err, GovernanceError::InvalidAuthorityForDabra.into());
}

#[tokio::test]
async fn test_set_dabra_config_with_remove_council() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let mut dabra_cookie = governance_test.with_dabra().await;

    let dabra_setup_args = DabraSetupArgs {
        use_council_mint: false,
        ..Default::default()
    };

    // Act
    governance_test
        .set_dabra_config(&mut dabra_cookie, &dabra_setup_args)
        .await
        .unwrap();

    // Assert
    let dabra_account = governance_test
        .get_dabra_account(&dabra_cookie.address)
        .await;

    assert_eq!(dabra_cookie.account, dabra_account);
    assert_eq!(None, dabra_account.config.council_mint);
}

#[tokio::test]
async fn test_set_dabra_config_with_council_change_error() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let mut dabra_cookie = governance_test.with_dabra().await;

    let dabra_setup_args = DabraSetupArgs::default();

    // Try to replace council mint
    dabra_cookie.account.config.council_mint = serde::__private::Some(Pubkey::new_unique());

    // Act
    let err = governance_test
        .set_dabra_config(&mut dabra_cookie, &dabra_setup_args)
        .await
        .err()
        .unwrap();

    // Assert
    assert_eq!(
        err,
        GovernanceError::DabraCouncilMintChangeIsNotSupported.into()
    );
}

#[tokio::test]
async fn test_set_dabra_config_with_council_restore_error() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let mut dabra_cookie = governance_test.with_dabra().await;

    let mut dabra_setup_args = DabraSetupArgs {
        use_council_mint: false,
        ..Default::default()
    };

    governance_test
        .set_dabra_config(&mut dabra_cookie, &dabra_setup_args)
        .await
        .unwrap();

    // Try to restore council mint after removing it
    dabra_setup_args.use_council_mint = true;
    dabra_cookie.account.config.council_mint = serde::__private::Some(Pubkey::new_unique());

    // Act
    let err = governance_test
        .set_dabra_config(&mut dabra_cookie, &dabra_setup_args)
        .await
        .err()
        .unwrap();

    // Assert
    assert_eq!(
        err,
        GovernanceError::DabraCouncilMintChangeIsNotSupported.into()
    );
}

#[tokio::test]
async fn test_set_dabra_config_with_liquid_community_token_cannot_be_changed_to_memebership_error()
{
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let mut dabra_cookie = governance_test.with_dabra().await;

    let mut dabra_setup_args = DabraSetupArgs::default();

    // Try to change Community token type to Membership
    dabra_setup_args.community_token_config_args.token_type = GoverningTokenType::Membership;

    // Act
    let err = governance_test
        .set_dabra_config(&mut dabra_cookie, &dabra_setup_args)
        .await
        .err()
        .unwrap();

    // Assert
    assert_eq!(
        err,
        GovernanceError::CannotChangeCommunityTokenTypeToMembership.into()
    );
}

#[tokio::test]
async fn test_set_dabra_config_for_community_token_config() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let mut dabra_cookie = governance_test.with_dabra().await;

    // Change Community token type to Dormant and set plugins
    let dabra_setup_args = DabraSetupArgs {
        community_token_config_args: GoverningTokenConfigAccountArgs {
            voter_weight_addin: Some(Pubkey::new_unique()),
            max_voter_weight_addin: Some(Pubkey::new_unique()),
            token_type: GoverningTokenType::Dormant,
        },
        ..Default::default()
    };

    // Act

    governance_test
        .set_dabra_config(&mut dabra_cookie, &dabra_setup_args)
        .await
        .unwrap();

    // Assert

    let dabra_config_account = governance_test
        .get_dabra_config_account(&dabra_cookie.dabra_config.address)
        .await;

    assert_eq!(
        dabra_config_account.community_token_config.token_type,
        GoverningTokenType::Dormant
    );

    assert_eq!(
        dabra_config_account
            .community_token_config
            .voter_weight_addin,
        dabra_setup_args
            .community_token_config_args
            .voter_weight_addin
    );

    assert_eq!(
        dabra_config_account
            .community_token_config
            .max_voter_weight_addin,
        dabra_setup_args
            .community_token_config_args
            .max_voter_weight_addin
    );
}

#[tokio::test]
async fn test_set_dabra_config_for_council_token_config() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let mut dabra_cookie = governance_test.with_dabra().await;

    // Change Council token type to Membership and set plugins
    let dabra_setup_args = DabraSetupArgs {
        council_token_config_args: GoverningTokenConfigAccountArgs {
            voter_weight_addin: Some(Pubkey::new_unique()),
            max_voter_weight_addin: Some(Pubkey::new_unique()),
            token_type: GoverningTokenType::Membership,
        },
        ..Default::default()
    };

    // Act

    governance_test
        .set_dabra_config(&mut dabra_cookie, &dabra_setup_args)
        .await
        .unwrap();

    // Assert

    let dabra_config_account = governance_test
        .get_dabra_config_account(&dabra_cookie.dabra_config.address)
        .await;

    assert_eq!(
        dabra_config_account.council_token_config.token_type,
        GoverningTokenType::Membership
    );

    assert_eq!(
        dabra_config_account.council_token_config.voter_weight_addin,
        dabra_setup_args
            .council_token_config_args
            .voter_weight_addin
    );

    assert_eq!(
        dabra_config_account
            .council_token_config
            .max_voter_weight_addin,
        dabra_setup_args
            .council_token_config_args
            .max_voter_weight_addin
    );
}

#[tokio::test]
async fn test_set_dabra_config_without_existing_dabra_config() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let mut dabra_cookie = governance_test.with_dabra().await;

    let dabra_setup_args = DabraSetupArgs::default();

    governance_test.remove_dabra_config_account(&dabra_cookie.dabra_config.address);

    // Act

    governance_test
        .set_dabra_config(&mut dabra_cookie, &dabra_setup_args)
        .await
        .unwrap();

    // Assert
    let dabra_account = governance_test
        .get_dabra_account(&dabra_cookie.address)
        .await;

    assert_eq!(dabra_cookie.account, dabra_account);
}

#[tokio::test]
async fn test_set_dabra_config_with_token_owner_record_lock_authorities() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let mut dabra_cookie = governance_test.with_dabra().await;

    let community_token_owner_record_lock_authority_cookie = governance_test
        .with_community_token_owner_record_lock_authority(&dabra_cookie)
        .await
        .unwrap();

    let council_token_owner_record_lock_authority_cookie = governance_test
        .with_council_token_owner_record_lock_authority(&dabra_cookie)
        .await
        .unwrap();

    let dabra_setup_args = DabraSetupArgs::default();

    // Act

    governance_test
        .set_dabra_config(&mut dabra_cookie, &dabra_setup_args)
        .await
        .unwrap();

    // Assert
    let dabra_config_account = governance_test
        .get_dabra_config_account(&dabra_cookie.dabra_config.address)
        .await;

    assert_eq!(
        vec![community_token_owner_record_lock_authority_cookie
            .authority
            .pubkey()],
        dabra_config_account.community_token_config.lock_authorities
    );

    assert_eq!(
        vec![council_token_owner_record_lock_authority_cookie
            .authority
            .pubkey()],
        dabra_config_account.council_token_config.lock_authorities
    );
}
