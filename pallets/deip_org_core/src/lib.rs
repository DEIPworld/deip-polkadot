//! # DEIP Org Core Module
//! A module that adapts Deip Core interface in context of DAO
//! 
//! - [`Config`](./trait.Config.html)
//!
//! ## Overview
//! A module that adapts Proposals interface by extending proposal members id variants
//! with org-name alongside native accounts
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! * `create_project` - Create project.
//!
//! [`Config`]: ./trait.Config.html

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

#[doc(inline)]
pub use pallet::*;

#[frame_support::pallet]
#[doc(hidden)]
pub mod pallet {
    use sp_std::prelude::*;
    use frame_system::pallet_prelude::*;
    use frame_support::pallet_prelude::*;
    use frame_support::dispatch::DispatchResult;

    use pallet_deip_org::org::OrgName;
    use pallet_deip::{ProjectId, DomainId};

    /// Configuration trait
    #[pallet::config]
    pub trait Config: frame_system::Config
                        + pallet_deip::Config
                        + pallet_deip_org::Config
    {}
    
    #[doc(hidden)]
    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);
    
    #[doc(hidden)]
    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}
    
    #[pallet::error]
    pub enum Error<T> {}
    
    #[doc(hidden)]
    #[pallet::genesis_config]
    #[derive(Default)]
    pub struct GenesisConfig {}

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig {
        fn build(&self) {}
    }

    #[derive(Debug, Clone, Eq, PartialEq, Encode, Decode)]
    pub enum OrgProjectAccount<AccountId> {
        Native(AccountId),
        OrgName(OrgName)
    }

    /// Twin to deip::Project
    #[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq)]
    pub struct Project<Hash, AccountId> {
        /// Determine visible project or not 
        is_private: bool,
        /// Reference for external world and uniques control 
        external_id: ProjectId,
        /// Reference to the Team 
        team_id: OrgProjectAccount<AccountId>,
        /// Hash of Project description
        description: Hash,
        /// List of Domains aka tags Project matches
        domains: Vec<DomainId>,
    }

    pub type ProjectOf<T> = Project<<T as frame_system::Config>::Hash, <T as frame_system::Config>::AccountId>;

    pub fn into_native<T: Config>(project: ProjectOf<T>)
        -> pallet_deip::ProjectOf<T>
    {
        pallet_deip::Project {
            is_private: project.is_private,
            external_id: project.external_id,
            team_id: match project.team_id {
                OrgProjectAccount::Native(account_id) => { account_id },
                OrgProjectAccount::OrgName(name) => {
                    pallet_deip_org::Pallet::<T>::org_key(&name)
                }
            },
            description: project.description,
            domains: project.domains,
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub fn create_project(
            origin: OriginFor<T>,
            project: ProjectOf<T>,
        )
            -> DispatchResultWithPostInfo
        {
            // let author = ensure_signed(origin)?;

            pallet_deip::Module::<T>::create_project(origin, into_native::<T>(project))?;

            Ok(Some(0).into())
        }
    }

    // #[pallet::call]
    // impl<T: Config> Pallet<T> {
    //     #[pallet::weight(10_000)]
    //     pub fn propose(
    //         origin: OriginFor<T>,
    //         batch: Vec<OrgProposalBatchItem<T>>,
    //         external_id: Option<ProposalId>
    //     )
    //         -> DispatchResultWithPostInfo
    //     {
    //         let author = ensure_signed(origin)?;
            
    //         // frame_support::debug::RuntimeLogger::init();
            
    //         imp::propose::<T>(
    //             author,
    //             batch.into_iter().map(into_native::<T>).collect(),
    //             external_id
    //         )?;
            
    //         Ok(Some(0).into())
    //     }
    // }
}
