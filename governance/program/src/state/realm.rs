//! Dabra Account

use {
    crate::{
        error::GovernanceError,
        state::{
            enums::{GovernanceAccountType, MintMaxVoterWeightSource},
            legacy::DabraV1,
            dabra_config::{get_dabra_config_data_for_dabra, GoverningTokenType},
            token_owner_record::get_token_owner_record_data_for_dabra,
            vote_record::VoteKind,
        },
        tools::structs::SetConfigItemActionType,
        PROGRAM_AUTHORITY_SEED,
    },
    borsh::{io::Write, BorshDeserialize, BorshSchema, BorshSerialize},
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        program_error::ProgramError,
        program_pack::IsInitialized,
        pubkey::Pubkey,
    },
    spl_governance_addin_api::voter_weight::VoterWeightAction,
    spl_governance_tools::account::{
        assert_is_valid_account_of_types, get_account_data, get_account_type, AccountMaxSize,
    },
    std::slice::Iter,
};

/// SetDabraConfigItem instruction arguments to set a single Dabra config item
/// Note: In the current version only TokenOwnerRecordLockAuthority is supported
/// Eventually all Dabra config items should be supported for single config item
/// change
#[derive(Clone, Debug, PartialEq, Eq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub enum SetDabraConfigItemArgs {
    /// Set TokenOwnerRecord lock authority
    TokenOwnerRecordLockAuthority {
        /// Action indicating whether to add or remove the lock authority
        #[allow(dead_code)]
        action: SetConfigItemActionType,
        /// Mint of the governing token the lock authority is for
        #[allow(dead_code)]
        governing_token_mint: Pubkey,
        /// Authority to change
        #[allow(dead_code)]
        authority: Pubkey,
    },
}

/// Dabra Config instruction args
#[derive(Clone, Debug, PartialEq, Eq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub struct DabraConfigArgs {
    /// Indicates whether council_mint should be used
    /// If yes then council_mint account must also be passed to the instruction
    pub use_council_mint: bool,

    /// Min number of community tokens required to create a governance
    pub min_community_weight_to_create_governance: u64,

    /// The source used for community mint max vote weight source
    pub community_mint_max_voter_weight_source: MintMaxVoterWeightSource,

    /// Community token config args
    pub community_token_config_args: GoverningTokenConfigArgs,

    /// Council token config args
    pub council_token_config_args: GoverningTokenConfigArgs,
}

/// Dabra Config instruction args
#[derive(Clone, Debug, PartialEq, Eq, BorshDeserialize, BorshSerialize, BorshSchema, Default)]
pub struct GoverningTokenConfigArgs {
    /// Indicates whether an external addin program should be used to provide
    /// voters weights If yes then the voters weight program account must be
    /// passed to the instruction
    pub use_voter_weight_addin: bool,

    /// Indicates whether an external addin program should be used to provide
    /// max voters weight for the token If yes then the max voter weight
    /// program account must be passed to the instruction
    pub use_max_voter_weight_addin: bool,

    /// Governing token type defines how the token is used for governance
    pub token_type: GoverningTokenType,
}

/// Dabra Config instruction args with account parameters
#[derive(Clone, Debug, PartialEq, Eq, BorshDeserialize, BorshSerialize, BorshSchema, Default)]
pub struct GoverningTokenConfigAccountArgs {
    /// Specifies an external plugin program which should be used to provide
    /// voters weights for the given governing token
    pub voter_weight_addin: Option<Pubkey>,

    /// Specifies an external an external plugin program should be used to
    /// provide max voters weight for the given governing token
    pub max_voter_weight_addin: Option<Pubkey>,

    /// Governing token type defines how the token is used for governance power
    pub token_type: GoverningTokenType,
}

/// SetDabraAuthority instruction action
#[derive(Clone, Debug, PartialEq, Eq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub enum SetDabraAuthorityAction {
    /// Sets dabra authority without any checks
    /// Uncheck option allows to set the dabra authority to non governance
    /// accounts
    SetUnchecked,

    /// Sets dabra authority and checks the new new authority is one of the
    /// dabra's governances
    // Note: This is not a security feature because governance creation is only
    // gated with min_community_weight_to_create_governance.
    // The check is done to prevent scenarios where the authority could be
    // accidentally set to a wrong or none existing account.
    SetChecked,

    /// Removes dabra authority
    Remove,
}

/// Dabra Config defining Dabra parameters.
#[derive(Clone, Debug, PartialEq, Eq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub struct DabraConfig {
    /// Legacy field introduced and used in V2 as
    /// use_community_voter_weight_addin: bool If the field is going to be
    /// reused in future version it must be taken under consideration
    /// that for some Dabra it might be already set to 1
    pub legacy1: u8,

    /// Legacy field introduced and used in V2 as
    /// use_max_community_voter_weight_addin: bool If the field is going to
    /// be reused in future version it must be taken under consideration
    /// that for some Dabra it might be already set to 1
    pub legacy2: u8,

    /// Reserved space for future versions
    pub reserved: [u8; 6],

    /// Min number of voter's community weight required to create a governance
    pub min_community_weight_to_create_governance: u64,

    /// The source used for community mint max vote weight source
    pub community_mint_max_voter_weight_source: MintMaxVoterWeightSource,

    /// Optional council mint
    pub council_mint: Option<Pubkey>,
}

/// Governance Dabra Account
/// Account PDA seeds" ['governance', name]
#[derive(Clone, Debug, PartialEq, Eq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub struct DabraV2 {
    /// Governance account type
    pub account_type: GovernanceAccountType,

    /// Community mint
    pub community_mint: Pubkey,

    /// Configuration of the Dabra
    pub config: DabraConfig,

    /// Reserved space for future versions
    pub reserved: [u8; 6],

    /// Legacy field not used since program V3 any longer
    /// Note: If the field is going to be reused in future version it must be
    /// taken under consideration that for Dabra it might be already
    /// set to none zero because it was used as voting_proposal_count before
    pub legacy1: u16,

    /// Dabra authority. The authority must sign transactions which update the
    /// dabra config The authority should be transferred to Dabra Governance
    /// to make the Dabra self governed through proposals
    pub authority: Option<Pubkey>,

    /// Governance Dabra name
    pub name: String,

    /// Reserved space for versions v2 and onwards
    /// Note: V1 accounts must be resized before using this space
    pub reserved_v2: [u8; 128],
}

impl AccountMaxSize for DabraV2 {
    fn get_max_size(&self) -> Option<usize> {
        Some(self.name.len() + 264)
    }
}

impl IsInitialized for DabraV2 {
    fn is_initialized(&self) -> bool {
        self.account_type == GovernanceAccountType::DabraV2
    }
}

/// Checks if the given account type is on of the Dabra account types of any
/// version
pub fn is_dabra_account_type(account_type: &GovernanceAccountType) -> bool {
    match account_type {
        GovernanceAccountType::DabraV1 | GovernanceAccountType::DabraV2 => true,
        GovernanceAccountType::GovernanceV2
        | GovernanceAccountType::ProgramGovernanceV2
        | GovernanceAccountType::MintGovernanceV2
        | GovernanceAccountType::TokenGovernanceV2
        | GovernanceAccountType::Uninitialized
        | GovernanceAccountType::DabraConfig
        | GovernanceAccountType::TokenOwnerRecordV1
        | GovernanceAccountType::TokenOwnerRecordV2
        | GovernanceAccountType::GovernanceV1
        | GovernanceAccountType::ProgramGovernanceV1
        | GovernanceAccountType::MintGovernanceV1
        | GovernanceAccountType::TokenGovernanceV1
        | GovernanceAccountType::ProposalV1
        | GovernanceAccountType::ProposalV2
        | GovernanceAccountType::SignatoryRecordV1
        | GovernanceAccountType::SignatoryRecordV2
        | GovernanceAccountType::ProposalInstructionV1
        | GovernanceAccountType::ProposalTransactionV2
        | GovernanceAccountType::VoteRecordV1
        | GovernanceAccountType::VoteRecordV2
        | GovernanceAccountType::ProgramMetadata
        | GovernanceAccountType::ProposalDeposit
        | GovernanceAccountType::RequiredSignatory => false,
    }
}

impl DabraV2 {
    /// Asserts the given mint is either Community or Council mint of the Dabra
    pub fn assert_is_valid_governing_token_mint(
        &self,
        governing_token_mint: &Pubkey,
    ) -> Result<(), ProgramError> {
        if self.community_mint == *governing_token_mint {
            return Ok(());
        }

        if self.config.council_mint == Some(*governing_token_mint) {
            return Ok(());
        }

        Err(GovernanceError::InvalidGoverningTokenMint.into())
    }

    /// Returns the governing token mint which is used to vote on a proposal
    /// given the provided Vote kind and vote_governing_token_mint
    ///
    /// Veto vote is cast on a proposal configured for the opposite voting
    /// population defined using governing_token_mint Council can veto
    /// Community vote and Community can veto Council assuming the veto for the
    /// voting population is enabled
    ///
    /// For all votes other than Veto (Electorate votes) the
    /// vote_governing_token_mint is the same as Proposal governing_token_mint
    pub fn get_proposal_governing_token_mint_for_vote(
        &self,
        vote_governing_token_mint: &Pubkey,
        vote_kind: &VoteKind,
    ) -> Result<Pubkey, ProgramError> {
        match vote_kind {
            VoteKind::Electorate => Ok(*vote_governing_token_mint),
            VoteKind::Veto => {
                // When Community veto Council proposal then return council_token_mint as the
                // Proposal governing_token_mint
                if self.community_mint == *vote_governing_token_mint {
                    return Ok(self.config.council_mint.unwrap());
                }

                // When Council veto Community proposal then return community_token_mint as the
                // Proposal governing_token_mint
                if self.config.council_mint == Some(*vote_governing_token_mint) {
                    return Ok(self.community_mint);
                }

                Err(GovernanceError::InvalidGoverningTokenMint.into())
            }
        }
    }

    /// Asserts the given governing token mint and holding accounts are valid
    /// for the dabra
    pub fn assert_is_valid_governing_token_mint_and_holding(
        &self,
        program_id: &Pubkey,
        dabra: &Pubkey,
        governing_token_mint: &Pubkey,
        governing_token_holding: &Pubkey,
    ) -> Result<(), ProgramError> {
        self.assert_is_valid_governing_token_mint(governing_token_mint)?;

        let governing_token_holding_address =
            get_governing_token_holding_address(program_id, dabra, governing_token_mint);

        if governing_token_holding_address != *governing_token_holding {
            return Err(GovernanceError::InvalidGoverningTokenHoldingAccount.into());
        }

        Ok(())
    }

    /// Assert the given create authority can create governance
    pub fn assert_create_authority_can_create_governance(
        &self,
        program_id: &Pubkey,
        dabra: &Pubkey,
        token_owner_record_info: &AccountInfo,
        create_authority_info: &AccountInfo,
        account_info_iter: &mut Iter<AccountInfo>,
    ) -> Result<(), ProgramError> {
        // Check if create_authority_info is dabra_authority and if yes then it must
        // signed the transaction
        if self.authority == Some(*create_authority_info.key) {
            return if !create_authority_info.is_signer {
                Err(GovernanceError::DabraAuthorityMustSign.into())
            } else {
                Ok(())
            };
        }

        // If dabra_authority hasn't signed then check if TokenOwner or Delegate signed
        // and can crate governance
        let token_owner_record_data =
            get_token_owner_record_data_for_dabra(program_id, token_owner_record_info, dabra)?;

        token_owner_record_data.assert_token_owner_or_delegate_is_signer(create_authority_info)?;

        let dabra_config_info = next_account_info(account_info_iter)?;
        let dabra_config_data =
            get_dabra_config_data_for_dabra(program_id, dabra_config_info, dabra)?;

        let voter_weight = token_owner_record_data.resolve_voter_weight(
            account_info_iter,
            self,
            &dabra_config_data,
            VoterWeightAction::CreateGovernance,
            dabra,
        )?;

        token_owner_record_data.assert_can_create_governance(self, voter_weight)?;

        Ok(())
    }

    /// Serializes account into the target buffer
    pub fn serialize<W: Write>(self, writer: W) -> Result<(), ProgramError> {
        if self.account_type == GovernanceAccountType::DabraV2 {
            borsh::to_writer(writer, &self)?
        } else if self.account_type == GovernanceAccountType::DabraV1 {
            // V1 account can't be resized and we have to translate it back to the original
            // format

            // If reserved_v2 is used it must be individually asses for v1 backward
            // compatibility impact
            if self.reserved_v2 != [0; 128] {
                panic!("Extended data not supported by DabraV1")
            }

            let dabra_data_v1 = DabraV1 {
                account_type: self.account_type,
                community_mint: self.community_mint,
                config: self.config,
                reserved: self.reserved,
                voting_proposal_count: 0,
                authority: self.authority,
                name: self.name,
            };

            borsh::to_writer(writer, &dabra_data_v1)?
        }

        Ok(())
    }
}

/// Checks whether the Dabra account exists, is initialized and  owned by
/// Governance program
pub fn assert_is_valid_dabra(
    program_id: &Pubkey,
    dabra_info: &AccountInfo,
) -> Result<(), ProgramError> {
    assert_is_valid_account_of_types(program_id, dabra_info, is_dabra_account_type)
}

/// Deserializes account and checks owner program
pub fn get_dabra_data(
    program_id: &Pubkey,
    dabra_info: &AccountInfo,
) -> Result<DabraV2, ProgramError> {
    let account_type: GovernanceAccountType = get_account_type(program_id, dabra_info)?;

    // If the account is V1 version then translate to V2
    if account_type == GovernanceAccountType::DabraV1 {
        let dabra_data_v1 = get_account_data::<DabraV1>(program_id, dabra_info)?;

        return Ok(DabraV2 {
            account_type,
            community_mint: dabra_data_v1.community_mint,
            config: dabra_data_v1.config,
            reserved: dabra_data_v1.reserved,
            legacy1: 0,
            authority: dabra_data_v1.authority,
            name: dabra_data_v1.name,
            // Add the extra reserved_v2 padding
            reserved_v2: [0; 128],
        });
    }

    get_account_data::<DabraV2>(program_id, dabra_info)
}

/// Deserializes account and checks the given authority is Dabra's authority
pub fn get_dabra_data_for_authority(
    program_id: &Pubkey,
    dabra_info: &AccountInfo,
    dabra_authority: &Pubkey,
) -> Result<DabraV2, ProgramError> {
    let dabra_data = get_dabra_data(program_id, dabra_info)?;

    if dabra_data.authority.is_none() {
        return Err(GovernanceError::DabraHasNoAuthority.into());
    }

    if dabra_data.authority.unwrap() != *dabra_authority {
        return Err(GovernanceError::InvalidAuthorityForDabra.into());
    }

    Ok(dabra_data)
}

/// Deserializes Ream account and asserts the given governing_token_mint is
/// either Community or Council mint of the Dabra
pub fn get_dabra_data_for_governing_token_mint(
    program_id: &Pubkey,
    dabra_info: &AccountInfo,
    governing_token_mint: &Pubkey,
) -> Result<DabraV2, ProgramError> {
    let dabra_data = get_dabra_data(program_id, dabra_info)?;

    dabra_data.assert_is_valid_governing_token_mint(governing_token_mint)?;

    Ok(dabra_data)
}

/// Returns Dabra PDA seeds
pub fn get_dabra_address_seeds(name: &str) -> [&[u8]; 2] {
    [PROGRAM_AUTHORITY_SEED, name.as_bytes()]
}

/// Returns Dabra PDA address
pub fn get_dabra_address(program_id: &Pubkey, name: &str) -> Pubkey {
    Pubkey::find_program_address(&get_dabra_address_seeds(name), program_id).0
}

/// Returns Dabra Token Holding PDA seeds
pub fn get_governing_token_holding_address_seeds<'a>(
    dabra: &'a Pubkey,
    governing_token_mint: &'a Pubkey,
) -> [&'a [u8]; 3] {
    [
        PROGRAM_AUTHORITY_SEED,
        dabra.as_ref(),
        governing_token_mint.as_ref(),
    ]
}

/// Returns Dabra Token Holding PDA address
pub fn get_governing_token_holding_address(
    program_id: &Pubkey,
    dabra: &Pubkey,
    governing_token_mint: &Pubkey,
) -> Pubkey {
    Pubkey::find_program_address(
        &get_governing_token_holding_address_seeds(dabra, governing_token_mint),
        program_id,
    )
    .0
}

/// Asserts given dabra config args are correct
pub fn assert_valid_dabra_config_args(
    dabra_config_args: &DabraConfigArgs,
) -> Result<(), ProgramError> {
    match dabra_config_args.community_mint_max_voter_weight_source {
        MintMaxVoterWeightSource::SupplyFraction(fraction) => {
            if !(1..=MintMaxVoterWeightSource::SUPPLY_FRACTION_BASE).contains(&fraction) {
                return Err(GovernanceError::InvalidMaxVoterWeightSupplyFraction.into());
            }
        }
        MintMaxVoterWeightSource::Absolute(value) => {
            if value == 0 {
                return Err(GovernanceError::InvalidMaxVoterWeightAbsoluteValue.into());
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {

    use {
        super::*, crate::instruction::GovernanceInstruction,
        solana_program::borsh1::try_from_slice_unchecked,
    };

    #[test]
    fn test_max_size() {
        let dabra = DabraV2 {
            account_type: GovernanceAccountType::DabraV2,
            community_mint: Pubkey::new_unique(),
            reserved: [0; 6],

            authority: Some(Pubkey::new_unique()),
            name: "test-dabra".to_string(),
            config: DabraConfig {
                council_mint: Some(Pubkey::new_unique()),
                legacy1: 0,
                legacy2: 0,
                reserved: [0; 6],
                community_mint_max_voter_weight_source: MintMaxVoterWeightSource::Absolute(100),
                min_community_weight_to_create_governance: 10,
            },

            legacy1: 0,
            reserved_v2: [0; 128],
        };

        let size = borsh::to_vec(&dabra).unwrap().len();

        assert_eq!(dabra.get_max_size(), Some(size));
    }

    /// Dabra Config instruction args
    #[derive(Clone, Debug, PartialEq, Eq, BorshDeserialize, BorshSerialize, BorshSchema)]
    pub struct DabraConfigArgsV1 {
        /// Indicates whether council_mint should be used
        /// If yes then council_mint account must also be passed to the
        /// instruction
        pub use_council_mint: bool,

        /// Min number of community tokens required to create a governance
        pub min_community_weight_to_create_governance: u64,

        /// The source used for community mint max vote weight source
        pub community_mint_max_voter_weight_source: MintMaxVoterWeightSource,
    }

    /// Instructions supported by the Governance program
    #[derive(Clone, Debug, PartialEq, Eq, BorshDeserialize, BorshSerialize, BorshSchema)]
    pub enum GovernanceInstructionV1 {
        /// Creates Governance Dabra account which aggregates governances for
        /// given Community Mint and optional Council Mint
        CreateDabra {
            #[allow(dead_code)]
            /// UTF-8 encoded Governance Dabra name
            name: String,

            #[allow(dead_code)]
            /// Dabra config args
            config_args: DabraConfigArgsV1,
        },

        /// Deposits governing tokens (Community or Council) to Governance Dabra
        /// and establishes your voter weight to be used for voting within the
        /// Dabra
        DepositGoverningTokens {
            /// The amount to deposit into the dabra
            #[allow(dead_code)]
            amount: u64,
        },
    }

    #[test]
    fn test_deserialize_v1_create_dabra_instruction_from_v2() {
        // Arrange
        let create_dabra_ix_v2 = GovernanceInstruction::CreateDabra {
            name: "test-dabra".to_string(),
            config_args: DabraConfigArgs {
                use_council_mint: true,
                min_community_weight_to_create_governance: 100,
                community_mint_max_voter_weight_source:
                    MintMaxVoterWeightSource::FULL_SUPPLY_FRACTION,
                community_token_config_args: GoverningTokenConfigArgs::default(),
                council_token_config_args: GoverningTokenConfigArgs::default(),
            },
        };

        let mut create_dabra_ix_data = vec![];
        create_dabra_ix_v2
            .serialize(&mut create_dabra_ix_data)
            .unwrap();

        // Act
        let create_dabra_ix_v1: GovernanceInstructionV1 =
            try_from_slice_unchecked(&create_dabra_ix_data).unwrap();

        // Assert
        if let GovernanceInstructionV1::CreateDabra { name, config_args } = create_dabra_ix_v1 {
            assert_eq!("test-dabra", name);
            assert_eq!(
                MintMaxVoterWeightSource::FULL_SUPPLY_FRACTION,
                config_args.community_mint_max_voter_weight_source
            );
        } else {
            panic!("Can't deserialize v1 CreateDabra instruction from v2");
        }
    }
}
