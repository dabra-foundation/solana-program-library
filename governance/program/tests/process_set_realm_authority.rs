#![cfg(feature = "test-sbf")]

use {solana_program::pubkey::Pubkey, solana_program_test::*};

mod program_test;

use {
    program_test::*, spl_governance::error::GovernanceError,
    spl_governance_tools::error::GovernanceToolsError,
};

#[tokio::test]
async fn test_set_dabra_authority() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let dabra_cookie = governance_test.with_dabra().await;

    let token_owner_record_cookie = governance_test
        .with_community_token_deposit(&dabra_cookie)
        .await
        .unwrap();

    let governance_cookie = governance_test
        .with_governance(&dabra_cookie, &token_owner_record_cookie)
        .await
        .unwrap();

    let new_dabra_authority = governance_cookie.address;

    // Act
    governance_test
        .set_dabra_authority(&dabra_cookie, Some(&new_dabra_authority))
        .await
        .unwrap();

    // Assert
    let dabra_account = governance_test
        .get_dabra_account(&dabra_cookie.address)
        .await;

    assert_eq!(dabra_account.authority, Some(new_dabra_authority));
}

#[tokio::test]
async fn test_set_dabra_authority_with_non_existing_new_authority_error() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let dabra_cookie = governance_test.with_dabra().await;

    let new_dabra_authority = Pubkey::new_unique();

    // Act
    let err = governance_test
        .set_dabra_authority(&dabra_cookie, Some(&new_dabra_authority))
        .await
        .err()
        .unwrap();

    // Assert
    assert_eq!(err, GovernanceToolsError::AccountDoesNotExist.into());
}

#[tokio::test]
async fn test_set_dabra_authority_to_none() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let dabra_cookie = governance_test.with_dabra().await;

    // Act
    governance_test
        .set_dabra_authority(&dabra_cookie, None)
        .await
        .unwrap();

    // Assert
    let dabra_account = governance_test
        .get_dabra_account(&dabra_cookie.address)
        .await;

    assert_eq!(dabra_account.authority, None);
}

#[tokio::test]
async fn test_set_dabra_authority_unchecked() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let dabra_cookie = governance_test.with_dabra().await;

    let new_dabra_authority = Pubkey::new_unique();

    // Act
    governance_test
        .set_dabra_authority_impl(&dabra_cookie, Some(&new_dabra_authority), false)
        .await
        .unwrap();

    // Assert
    let dabra_account = governance_test
        .get_dabra_account(&dabra_cookie.address)
        .await;

    assert_eq!(dabra_account.authority, Some(new_dabra_authority));
}

#[tokio::test]
async fn test_set_dabra_authority_with_no_authority_error() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let dabra_cookie = governance_test.with_dabra().await;

    governance_test
        .set_dabra_authority(&dabra_cookie, None)
        .await
        .unwrap();

    let new_dabra_authority = Pubkey::new_unique();

    // Act
    let err = governance_test
        .set_dabra_authority(&dabra_cookie, Some(&new_dabra_authority))
        .await
        .err()
        .unwrap();

    // Assert
    assert_eq!(err, GovernanceError::DabraHasNoAuthority.into());
}

#[tokio::test]
async fn test_set_dabra_authority_with_invalid_authority_error() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let mut dabra_cookie = governance_test.with_dabra().await;
    let dabra_cookie2 = governance_test.with_dabra().await;

    let new_dabra_authority = Pubkey::new_unique();

    // Try to use authority from other dabra
    dabra_cookie.dabra_authority = dabra_cookie2.dabra_authority;

    // Act
    let err = governance_test
        .set_dabra_authority(&dabra_cookie, Some(&new_dabra_authority))
        .await
        .err()
        .unwrap();

    // Assert
    assert_eq!(err, GovernanceError::InvalidAuthorityForDabra.into());
}

#[tokio::test]
async fn test_set_dabra_authority_with_authority_must_sign_error() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let dabra_cookie = governance_test.with_dabra().await;

    let new_dabra_authority = Pubkey::new_unique();

    // Act
    let err = governance_test
        .set_dabra_authority_using_instruction(
            &dabra_cookie,
            Some(&new_dabra_authority),
            true,
            |i| i.accounts[1].is_signer = false, // dabra_authority
            Some(&[]),
        )
        .await
        .err()
        .unwrap();

    // Assert
    assert_eq!(err, GovernanceError::DabraAuthorityMustSign.into());
}

#[tokio::test]
async fn test_set_dabra_authority_with_governance_from_other_dabra_error() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let dabra_cookie = governance_test.with_dabra().await;

    // Setup other dabra
    let dabra_cookie2 = governance_test.with_dabra().await;

    let token_owner_record_cookie2 = governance_test
        .with_community_token_deposit(&dabra_cookie2)
        .await
        .unwrap();

    let governance_cookie2 = governance_test
        .with_governance(&dabra_cookie2, &token_owner_record_cookie2)
        .await
        .unwrap();

    let new_dabra_authority = governance_cookie2.address;

    // Act
    let err = governance_test
        .set_dabra_authority(&dabra_cookie, Some(&new_dabra_authority))
        .await
        .err()
        .unwrap();

    // Assert
    assert_eq!(err, GovernanceError::InvalidDabraForGovernance.into());
}
