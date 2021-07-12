//! # DEIP OrgProposal Module
//! A module that adapts Proposals interface in context of DAO
//! 
//! - [`Config`](./trait.Config.html)
//! - [`Call`](./enum.Call.html)
//!
//! ## Overview
//! A module that adapts Proposals interface by extending proposal members id variants
//! with org-name alongside native accounts
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! * `propose` - Create proposal.
//!
//! [`Call`]: ./enum.Call.html
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
    
    use pallet_deip_proposal::{ProposalId, BatchItem};
    use pallet_deip_proposal::entrypoint as imp;
    use pallet_deip_org::org::OrgName;

    /// Configuration trait
    #[pallet::config]
    pub trait Config: frame_system::Config
                        + pallet_deip_proposal::Config
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
    pub enum OrgProposalAccount<AccountId> {
        Native(AccountId),
        Org(OrgName)
    }
    
    #[allow(type_alias_bounds)]
    pub type OrgProposalBatchItem<T: Config> = BatchItem<
        OrgProposalAccount<T::AccountId>,
        <T as pallet_deip_proposal::Config>::Call
    >;

    pub fn into_native<T: Config>(item: OrgProposalBatchItem<T>)
        -> pallet_deip_proposal::ProposalBatchItemOf<T>
    {
        let BatchItem { account, call } = item;
        BatchItem {
            account: match account {
                OrgProposalAccount::Native(native) => { native },
                OrgProposalAccount::Org(name) => {
                    pallet_deip_org::Pallet::<T>::org_key(&name)
                },
            },
            call,
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        pub fn propose(
            origin: OriginFor<T>,
            batch: Vec<OrgProposalBatchItem<T>>,
            external_id: Option<ProposalId>
        )
            -> DispatchResultWithPostInfo
        {
            let author = ensure_signed(origin)?;
            
            // frame_support::debug::RuntimeLogger::init();
            
            imp::propose::<T>(
                author,
                batch.into_iter().map(into_native::<T>).collect(),
                external_id
            )?;
            
            Ok(Some(0).into())
        }
    }
}
