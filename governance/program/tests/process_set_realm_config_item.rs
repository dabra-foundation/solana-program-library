#![cfg(feature = "test-sbf")]

mod program_test;

use {
    program_test::*,
    solana_program::pubkey::Pubkey,
    solana_program_test::tokio,
    solana_sdk::{signature::Keypair, signer::Signer},
    spl_governance::{
        error::GovernanceError,
        state::{
            enums::GovernanceAccountType,
            dabra::SetDabraConfigItemArgs,
            dabra_config::{GoverningTokenConfig, DabraConfigAccount},
        },
        tools::structs::{Reserved110, SetConfigItemActionType},
    },
    spl_governance_tools::account::AccountMaxSize,
};

#[tokio::test]
async fn test_add_community_token_owner_record_lock_authority() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let dabra_cookie = governance_test.with_dabra().await;

    // Act
    let token_owner_record_lock_authority_cookie = governance_test
        .with_community_token_owner_record_lock_authority(&dabra_cookie)
        .await
        .unwrap();

    // Assert
    let dabra_config_account = governance_test
        .get_dabra_config_account(&dabra_cookie.dabra_config.address)
        .await;

    assert_eq!(
        1,
        dabra_config_account
            .community_token_config
            .lock_authorities
            .len()
    );

    assert_eq!(
        &token_owner_record_lock_authority_cookie.authority.pubkey(),
        dabra_config_account
            .community_token_config
            .lock_authorities
            .first()
            .unwrap()
    );
}

#[tokio::test]
async fn test_remove_community_token_owner_record_lock_authority() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let dabra_cookie = governance_test.with_dabra().await;

    let token_owner_record_lock_authority_cookie = governance_test
        .with_community_token_owner_record_lock_authority(&dabra_cookie)
        .await
        .unwrap();

    // Act

    let args = SetDabraConfigItemArgs::TokenOwnerRecordLockAuthority {
        action: SetConfigItemActionType::Remove,
        governing_token_mint: dabra_cookie.account.community_mint,
        authority: token_owner_record_lock_authority_cookie.authority.pubkey(),
    };

    governance_test
        .set_dabra_config_item(&dabra_cookie, args)
        .await
        .unwrap();

    // Assert
    let dabra_config_account = governance_test
        .get_dabra_config_account(&dabra_cookie.dabra_config.address)
        .await;

    assert_eq!(
        0,
        dabra_config_account
            .community_token_config
            .lock_authorities
            .len()
    );
}

#[tokio::test]
async fn test_add_council_token_owner_record_lock_authority() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let dabra_cookie = governance_test.with_dabra().await;

    // Act
    let token_owner_record_lock_authority_cookie = governance_test
        .with_council_token_owner_record_lock_authority(&dabra_cookie)
        .await
        .unwrap();

    // Assert
    let dabra_config_account = governance_test
        .get_dabra_config_account(&dabra_cookie.dabra_config.address)
        .await;

    assert_eq!(
        1,
        dabra_config_account
            .council_token_config
            .lock_authorities
            .len()
    );

    assert_eq!(
        &token_owner_record_lock_authority_cookie.authority.pubkey(),
        dabra_config_account
            .council_token_config
            .lock_authorities
            .first()
            .unwrap()
    );
}

#[tokio::test]
async fn test_set_dabra_config_item_with_dabra_authority_must_sign_error() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let dabra_cookie = governance_test.with_dabra().await;

    let args = SetDabraConfigItemArgs::TokenOwnerRecordLockAuthority {
        action: SetConfigItemActionType::Add,
        governing_token_mint: dabra_cookie.account.community_mint,
        authority: Keypair::new().pubkey(),
    };

    // Act
    let err = governance_test
        .set_dabra_config_item_using_ix(
            &dabra_cookie,
            args,
            |i| i.accounts[2].is_signer = false,
            Some(&[]),
        )
        .await
        .err()
        .unwrap();

    // Assert
    assert_eq!(err, GovernanceError::DabraAuthorityMustSign.into());
}

#[tokio::test]
async fn test_set_dabra_config_item_with_invalid_dabra_authority_error() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let dabra_cookie = governance_test.with_dabra().await;

    let args = SetDabraConfigItemArgs::TokenOwnerRecordLockAuthority {
        action: SetConfigItemActionType::Add,
        governing_token_mint: dabra_cookie.account.community_mint,
        authority: Keypair::new().pubkey(),
    };

    let dabra_authority = Keypair::new();

    // Act
    let err = governance_test
        .set_dabra_config_item_using_ix(
            &dabra_cookie,
            args,
            |i| i.accounts[2].pubkey = dabra_authority.pubkey(),
            Some(&[&dabra_authority]),
        )
        .await
        .err()
        .unwrap();

    // Assert
    assert_eq!(err, GovernanceError::InvalidAuthorityForDabra.into());
}

#[tokio::test]
async fn test_set_dabra_config_item_with_invalid_dabra_config_error() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let dabra_cookie = governance_test.with_dabra().await;

    let args = SetDabraConfigItemArgs::TokenOwnerRecordLockAuthority {
        action: SetConfigItemActionType::Add,
        governing_token_mint: dabra_cookie.account.community_mint,
        authority: Keypair::new().pubkey(),
    };

    let dabra_cookie2 = governance_test.with_dabra().await;

    // Act
    let err = governance_test
        .set_dabra_config_item_using_ix(
            &dabra_cookie,
            args,
            |i| i.accounts[1].pubkey = dabra_cookie2.dabra_config.address,
            None,
        )
        .await
        .err()
        .unwrap();

    // Assert
    assert_eq!(err, GovernanceError::InvalidDabraConfigForDabra.into());
}

#[tokio::test]
async fn test_add_token_owner_record_lock_authority_with_invalid_governing_token_mint() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let dabra_cookie = governance_test.with_dabra().await;

    let args = SetDabraConfigItemArgs::TokenOwnerRecordLockAuthority {
        action: SetConfigItemActionType::Add,
        governing_token_mint: Pubkey::new_unique(), // Use invalid mint
        authority: Keypair::new().pubkey(),
    };

    // Act
    let err = governance_test
        .set_dabra_config_item(&dabra_cookie, args)
        .await
        .err()
        .unwrap();

    // Assert
    assert_eq!(err, GovernanceError::InvalidGoverningTokenMint.into());
}

#[tokio::test]
async fn test_add_token_owner_record_lock_authority_with_authority_already_exists_error() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let dabra_cookie = governance_test.with_dabra().await;

    let args = SetDabraConfigItemArgs::TokenOwnerRecordLockAuthority {
        action: SetConfigItemActionType::Add,
        governing_token_mint: dabra_cookie.account.config.council_mint.unwrap(),
        // Set the same authority
        authority: Pubkey::new_unique(),
    };

    governance_test
        .set_dabra_config_item(&dabra_cookie, args.clone())
        .await
        .unwrap();

    // Advance the clock to accept the same transaction
    governance_test.advance_clock().await;

    // Act
    let err = governance_test
        .set_dabra_config_item(&dabra_cookie, args)
        .await
        .err()
        .unwrap();

    // Assert
    assert_eq!(
        err,
        GovernanceError::TokenOwnerRecordLockAuthorityAlreadyExists.into()
    );
}

#[tokio::test]
async fn test_set_dabra_config_item_without_existing_dabra_config() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let dabra_cookie = governance_test.with_dabra().await;

    governance_test.remove_dabra_config_account(&dabra_cookie.dabra_config.address);

    // Act
    governance_test
        .with_community_token_owner_record_lock_authority(&dabra_cookie)
        .await
        .unwrap();

    // Assert
    let dabra_config_account = governance_test
        .get_dabra_config_account(&dabra_cookie.dabra_config.address)
        .await;

    assert_eq!(
        1,
        dabra_config_account
            .community_token_config
            .lock_authorities
            .len()
    );
}

#[tokio::test]
async fn test_set_dabra_config_item_with_extended_account_size() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let dabra_cookie = governance_test.with_dabra().await;

    // Act
    governance_test
        .with_community_token_owner_record_lock_authority(&dabra_cookie)
        .await
        .unwrap();

    // Assert
    let dabra_config_account = governance_test
        .bench
        .get_account(&dabra_cookie.dabra_config.address)
        .await
        .unwrap();

    // DabraConfig without any lock authorities
    let dabra_config = DabraConfigAccount {
        account_type: GovernanceAccountType::DabraConfig,
        dabra: dabra_cookie.address,
        community_token_config: GoverningTokenConfig::default(),
        council_token_config: GoverningTokenConfig::default(),
        reserved: Reserved110::default(),
    };

    assert_eq!(
        dabra_config.get_max_size().unwrap() + 32,
        dabra_config_account.data.len()
    );
}

#[tokio::test]
async fn test_remove_token_owner_record_lock_authority_with_authority_not_found_error() {
    // Arrange
    let mut governance_test = GovernanceProgramTest::start_new().await;

    let dabra_cookie = governance_test.with_dabra().await;

    let token_owner_record_lock_authority_cookie = governance_test
        .with_community_token_owner_record_lock_authority(&dabra_cookie)
        .await
        .unwrap();

    let args = SetDabraConfigItemArgs::TokenOwnerRecordLockAuthority {
        action: SetConfigItemActionType::Remove,
        governing_token_mint: dabra_cookie.account.community_mint,
        authority: token_owner_record_lock_authority_cookie.authority.pubkey(),
    };

    governance_test
        .set_dabra_config_item(&dabra_cookie, args.clone())
        .await
        .unwrap();

    // Advance the clock to accept the same transaction
    governance_test.advance_clock().await;

    // Act
    let err = governance_test
        .set_dabra_config_item(&dabra_cookie, args)
        .await
        .err()
        .unwrap();

    // Assert
    assert_eq!(
        err,
        GovernanceError::TokenOwnerRecordLockAuthorityNotFound.into()
    );
}
