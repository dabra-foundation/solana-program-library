//! DabraConfig account
use {
    crate::{
        error::GovernanceError,
        state::{
            enums::GovernanceAccountType,
            dabra::{GoverningTokenConfigArgs, DabraConfigArgs, DabraV2},
        },
        tools::structs::Reserved110,
    },
    borsh::{BorshDeserialize, BorshSchema, BorshSerialize},
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        program_error::ProgramError,
        program_pack::IsInitialized,
        pubkey::Pubkey,
        rent::Rent,
    },
    spl_governance_tools::account::{
        create_and_serialize_account_signed, extend_account_size, get_account_data, AccountMaxSize,
    },
    std::slice::Iter,
};

/// The type of the governing token defines:
/// 1) Who retains the authority over deposited tokens
/// 2) Which token instructions Deposit, Withdraw and Revoke (burn) are allowed
#[derive(Clone, Debug, PartialEq, Eq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub enum GoverningTokenType {
    /// Liquid token is a token which is fully liquid and the token owner
    /// retains full authority over it.
    /// Deposit - Yes
    /// Withdraw - Yes  
    /// Revoke - No, Dabra authority cannot revoke liquid tokens
    Liquid,

    /// Membership token is a token controlled by Dabra authority
    /// Deposit - Yes, membership tokens can be deposited to gain governance
    /// power.
    /// The membership tokens are conventionally minted into the holding
    /// account to keep them out of members possession.
    /// Withdraw - No, after membership tokens are deposited they are no longer
    /// transferable and can't be withdrawn.
    /// Revoke - Yes, Dabra authority can Revoke (burn) membership tokens.
    Membership,

    /// Dormant token is a token which is only a placeholder and its deposits
    /// are not accepted and not used for governance power within the Dabra
    ///
    /// The Dormant token type is used when only a single voting population is
    /// operational. For example a Multisig starter DAO uses Council only
    /// and sets Community as Dormant to indicate its not utilized for any
    /// governance power. Once the starter DAO decides to decentralise then
    /// it can change the Community token to Liquid
    ///
    /// Note: When an external voter weight plugin which takes deposits of the
    /// token is used then the type should be set to Dormant to make the
    /// intention explicit
    ///
    /// Deposit - No, dormant tokens can't be deposited into the Dabra
    /// Withdraw - Yes, tokens can still be withdrawn from Dabra to support
    /// scenario where the config is changed while some tokens are still
    /// deposited.
    /// Revoke - No, Dabra authority cannot revoke dormant tokens
    Dormant,
}

#[allow(clippy::derivable_impls)]
impl Default for GoverningTokenType {
    fn default() -> Self {
        GoverningTokenType::Liquid
    }
}

/// GoverningTokenConfig specifies configuration for Dabra governing token
/// (Community or Council)
#[derive(Clone, Debug, PartialEq, Eq, BorshDeserialize, BorshSerialize, BorshSchema, Default)]
pub struct GoverningTokenConfig {
    /// Plugin providing voter weights for the governing token
    pub voter_weight_addin: Option<Pubkey>,

    /// Plugin providing max voter weight for the governing token
    pub max_voter_weight_addin: Option<Pubkey>,

    /// Governing token type
    pub token_type: GoverningTokenType,

    /// Reserved space for future versions
    pub reserved: [u8; 4],

    /// Lock authorities for TokenOwnerRecords
    pub lock_authorities: Vec<Pubkey>,
}

/// DabraConfig account
/// The account is an optional extension to DabraConfig stored on Dabra account
#[derive(Clone, Debug, PartialEq, Eq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub struct DabraConfigAccount {
    /// Governance account type
    pub account_type: GovernanceAccountType,

    /// The dabra the config belong to
    pub dabra: Pubkey,

    /// Community token config
    pub community_token_config: GoverningTokenConfig,

    /// Council token config
    pub council_token_config: GoverningTokenConfig,

    /// Reserved
    pub reserved: Reserved110,
}

impl AccountMaxSize for DabraConfigAccount {
    fn get_max_size(&self) -> Option<usize> {
        Some(
            1 + 32
                + 75 * 2
                + 110
                + self.community_token_config.lock_authorities.len() * 32
                + self.council_token_config.lock_authorities.len() * 32,
        )
    }
}

impl IsInitialized for DabraConfigAccount {
    fn is_initialized(&self) -> bool {
        self.account_type == GovernanceAccountType::DabraConfig
    }
}

impl DabraConfigAccount {
    /// Returns GoverningTokenConfig for the given governing_token_mint
    pub fn get_token_config(
        &self,
        dabra_data: &DabraV2,
        governing_token_mint: &Pubkey,
    ) -> Result<&GoverningTokenConfig, ProgramError> {
        let token_config = if *governing_token_mint == dabra_data.community_mint {
            &self.community_token_config
        } else if Some(*governing_token_mint) == dabra_data.config.council_mint {
            &self.council_token_config
        } else {
            return Err(GovernanceError::InvalidGoverningTokenMint.into());
        };

        Ok(token_config)
    }

    /// Returns mutable GoverningTokenConfig for the given governing_token_mint
    pub fn get_token_config_mut(
        &mut self,
        dabra_data: &DabraV2,
        governing_token_mint: &Pubkey,
    ) -> Result<&mut GoverningTokenConfig, ProgramError> {
        let token_config = if *governing_token_mint == dabra_data.community_mint {
            &mut self.community_token_config
        } else if Some(*governing_token_mint) == dabra_data.config.council_mint {
            &mut self.council_token_config
        } else {
            return Err(GovernanceError::InvalidGoverningTokenMint.into());
        };

        Ok(token_config)
    }

    /// Asserts the given governing token can be revoked
    pub fn assert_can_revoke_governing_token(
        &self,
        dabra_data: &DabraV2,
        governing_token_mint: &Pubkey,
    ) -> Result<(), ProgramError> {
        let governing_token_type = &self
            .get_token_config(dabra_data, governing_token_mint)?
            .token_type;

        match governing_token_type {
            GoverningTokenType::Membership => Ok(()),
            GoverningTokenType::Liquid | GoverningTokenType::Dormant => {
                Err(GovernanceError::CannotRevokeGoverningTokens.into())
            }
        }
    }

    /// Asserts the given governing token can be deposited
    pub fn assert_can_deposit_governing_token(
        &self,
        dabra_data: &DabraV2,
        governing_token_mint: &Pubkey,
    ) -> Result<(), ProgramError> {
        let governing_token_type = &self
            .get_token_config(dabra_data, governing_token_mint)?
            .token_type;

        match governing_token_type {
            GoverningTokenType::Membership | GoverningTokenType::Liquid => Ok(()),
            // Note: Preventing deposits of the Dormant type tokens is not a direct security concern
            // It only makes the intention of not using deposited tokens as governance power
            // stronger
            GoverningTokenType::Dormant => Err(GovernanceError::CannotDepositDormantTokens.into()),
        }
    }

    /// Asserts the given governing token can be withdrawn
    pub fn assert_can_withdraw_governing_token(
        &self,
        dabra_data: &DabraV2,
        governing_token_mint: &Pubkey,
    ) -> Result<(), ProgramError> {
        let governing_token_type = &self
            .get_token_config(dabra_data, governing_token_mint)?
            .token_type;

        match governing_token_type {
            GoverningTokenType::Dormant | GoverningTokenType::Liquid => Ok(()),
            GoverningTokenType::Membership => {
                Err(GovernanceError::CannotWithdrawMembershipTokens.into())
            }
        }
    }

    /// Asserts the given DabraConfigArgs represent a valid Dabra configuration
    /// change
    pub fn assert_can_change_config(
        &self,
        dabra_config_args: &DabraConfigArgs,
    ) -> Result<(), ProgramError> {
        // Existing community token type can't be changed to Membership because it would
        // give the Dabra authority the right to burn members tokens which should not be
        // the case because the tokens belong to the members On the other had
        // for the Council token it's acceptable and in fact desired change because
        // council tokens denote membership which should be controlled by the
        // Dabra
        if self.community_token_config.token_type != GoverningTokenType::Membership
            && dabra_config_args.community_token_config_args.token_type
                == GoverningTokenType::Membership
        {
            return Err(GovernanceError::CannotChangeCommunityTokenTypeToMembership.into());
        }

        Ok(())
    }

    /// Serializes DabraConfigAccount and resizes it if required
    /// If the account doesn't exist then it's created
    pub fn serialize<'a>(
        self,
        program_id: &Pubkey,
        dabra_config_info: &AccountInfo<'a>,
        payer_info: &AccountInfo<'a>,
        system_info: &AccountInfo<'a>,
        rent: &Rent,
    ) -> Result<(), ProgramError> {
        // Update or create DabraConfigAccount
        if dabra_config_info.data_is_empty() {
            // For older Dabra accounts (pre program V3) DabraConfigAccount might not exist
            // yet and we have to create it

            create_and_serialize_account_signed::<DabraConfigAccount>(
                payer_info,
                dabra_config_info,
                &self,
                &get_dabra_config_address_seeds(&self.dabra),
                program_id,
                system_info,
                rent,
                0,
            )?;
        } else {
            let dabra_config_max_size = self.get_max_size().unwrap();
            if dabra_config_info.data_len() < dabra_config_max_size {
                extend_account_size(
                    dabra_config_info,
                    payer_info,
                    dabra_config_max_size,
                    rent,
                    system_info,
                )?;
            }

            borsh::to_writer(&mut dabra_config_info.data.borrow_mut()[..], &self)?;
        };

        Ok(())
    }
}

/// Deserializes DabraConfig account and checks owner program
pub fn get_dabra_config_data(
    program_id: &Pubkey,
    dabra_config_info: &AccountInfo,
) -> Result<DabraConfigAccount, ProgramError> {
    get_account_data::<DabraConfigAccount>(program_id, dabra_config_info)
}

/// If the account exists then deserializes it into DabraConfigAccount struct
/// and checks the owner program and the Dabra it belongs to If the account
/// doesn't exist then it checks its address is derived from the given owner
/// program and Dabra and returns default DabraConfigAccount
pub fn get_dabra_config_data_for_dabra(
    program_id: &Pubkey,
    dabra_config_info: &AccountInfo,
    dabra: &Pubkey,
) -> Result<DabraConfigAccount, ProgramError> {
    let dabra_config_data = if dabra_config_info.data_is_empty() {
        // If DabraConfigAccount doesn't exist yet then validate its PDA
        // PDA validation is required because DabraConfigAccount might not exist for
        // legacy Dabra and then its absence is used as default
        // DabraConfigAccount value with no plugins and Liquid governance tokens
        let dabra_config_address = get_dabra_config_address(program_id, dabra);

        if dabra_config_address != *dabra_config_info.key {
            return Err(GovernanceError::InvalidDabraConfigAddress.into());
        }

        DabraConfigAccount {
            account_type: GovernanceAccountType::DabraConfig,
            dabra: *dabra,
            community_token_config: GoverningTokenConfig::default(),
            council_token_config: GoverningTokenConfig::default(),
            reserved: Reserved110::default(),
        }
    } else {
        let dabra_config_data = get_dabra_config_data(program_id, dabra_config_info)?;

        if dabra_config_data.dabra != *dabra {
            return Err(GovernanceError::InvalidDabraConfigForDabra.into());
        }

        dabra_config_data
    };

    Ok(dabra_config_data)
}

/// Returns DabraConfig PDA seeds
pub fn get_dabra_config_address_seeds(dabra: &Pubkey) -> [&[u8]; 2] {
    [b"dabra-config", dabra.as_ref()]
}

/// Returns DabraConfig PDA address
pub fn get_dabra_config_address(program_id: &Pubkey, dabra: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&get_dabra_config_address_seeds(dabra), program_id).0
}
/// Resolves GoverningTokenConfig from GoverningTokenConfigArgs and instruction
/// accounts
pub fn resolve_governing_token_config(
    account_info_iter: &mut Iter<AccountInfo>,
    governing_token_config_args: &GoverningTokenConfigArgs,
    existing_governing_token_config: Option<GoverningTokenConfig>,
) -> Result<GoverningTokenConfig, ProgramError> {
    let voter_weight_addin = if governing_token_config_args.use_voter_weight_addin {
        let voter_weight_addin_info = next_account_info(account_info_iter)?;
        Some(*voter_weight_addin_info.key)
    } else {
        None
    };

    let max_voter_weight_addin = if governing_token_config_args.use_max_voter_weight_addin {
        let max_voter_weight_addin_info = next_account_info(account_info_iter)?;
        Some(*max_voter_weight_addin_info.key)
    } else {
        None
    };

    let lock_authorities =
        if let Some(existing_governing_token_config) = existing_governing_token_config {
            existing_governing_token_config.lock_authorities
        } else {
            vec![]
        };

    Ok(GoverningTokenConfig {
        voter_weight_addin,
        max_voter_weight_addin,
        token_type: governing_token_config_args.token_type.clone(),
        reserved: [0; 4],
        lock_authorities,
    })
}

#[cfg(test)]
mod test {
    use {
        super::*,
        crate::state::{enums::GovernanceAccountType, dabra_config::DabraConfigAccount},
    };

    #[test]
    fn test_max_size() {
        let dabra_config = DabraConfigAccount {
            account_type: GovernanceAccountType::DabraV2,
            dabra: Pubkey::new_unique(),
            community_token_config: GoverningTokenConfig {
                voter_weight_addin: Some(Pubkey::new_unique()),
                max_voter_weight_addin: Some(Pubkey::new_unique()),
                token_type: GoverningTokenType::Liquid,
                reserved: [0; 4],
                lock_authorities: vec![],
            },
            council_token_config: GoverningTokenConfig {
                voter_weight_addin: Some(Pubkey::new_unique()),
                max_voter_weight_addin: Some(Pubkey::new_unique()),
                token_type: GoverningTokenType::Liquid,
                reserved: [0; 4],
                lock_authorities: vec![],
            },
            reserved: Reserved110::default(),
        };

        let size = borsh::to_vec(&dabra_config).unwrap().len();

        assert_eq!(dabra_config.get_max_size(), Some(size));
    }

    #[test]
    fn test_max_size_with_lock_authorities() {
        let dabra_config = DabraConfigAccount {
            account_type: GovernanceAccountType::DabraV2,
            dabra: Pubkey::new_unique(),
            community_token_config: GoverningTokenConfig {
                voter_weight_addin: Some(Pubkey::new_unique()),
                max_voter_weight_addin: Some(Pubkey::new_unique()),
                token_type: GoverningTokenType::Liquid,
                reserved: [0; 4],
                lock_authorities: vec![Pubkey::new_unique()],
            },
            council_token_config: GoverningTokenConfig {
                voter_weight_addin: Some(Pubkey::new_unique()),
                max_voter_weight_addin: Some(Pubkey::new_unique()),
                token_type: GoverningTokenType::Liquid,
                reserved: [0; 4],
                lock_authorities: vec![Pubkey::new_unique(), Pubkey::new_unique()],
            },
            reserved: Reserved110::default(),
        };

        let size = borsh::to_vec(&dabra_config).unwrap().len();

        assert_eq!(dabra_config.get_max_size(), Some(size));
    }
}
