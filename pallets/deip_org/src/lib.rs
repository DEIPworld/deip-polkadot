//! # DEIP Org Module
//! A module for manage virtual organisations and perform actions on behalf of it
//! 
//! - [`Config`](./trait.Config.html)
//! - [`Call`](./enum.Call.html)
//!
//! ## Overview
//! A module for manage virtual organisations and perform actions on behalf of it
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! * `create` - Create an Org.
//! * `transfer_ownership` - Transfer ownership of an Org to another account.
//! * `on_behalf` - Perform action on behalf of an Org.
//!
//! [`Call`]: ./enum.Call.html
//! [`Config`]: ./trait.Config.html

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

#[cfg(test)]
mod tests;

pub mod api;

#[doc(inline)]
pub use pallet::*;

#[frame_support::pallet]
#[doc(hidden)]
pub mod pallet {
    use frame_system::pallet_prelude::*;
    use frame_system::RawOrigin;
    
    use frame_support::pallet_prelude::*;
    use frame_support::{Hashable};
    use frame_support::weights::{PostDispatchInfo, GetDispatchInfo};
    
    use frame_support::traits::{UnfilteredDispatchable, IsSubType};
    
    use sp_std::prelude::*;
    use sp_std::collections::{btree_map::BTreeMap};
    use sp_std::iter::FromIterator;
    
    use sp_runtime::traits::Dispatchable;
    use frame_support::dispatch::DispatchResult;
    
    use pallet_deip_toolkit::storage_ops::StorageOpsTransaction;

    /// Configuration trait
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Type represents events
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// Type represents particular call from batch-transaction 
        type Call: Parameter +
             Dispatchable<Origin = Self::Origin, PostInfo = PostDispatchInfo> +
             GetDispatchInfo +
             From<frame_system::pallet::Call<Self>> +
             UnfilteredDispatchable<Origin = Self::Origin> +
             frame_support::dispatch::Codec + 
             IsSubType<Call<Self>>;
    }
    
    #[doc(hidden)]
    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);
    
    #[doc(hidden)]
    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}
    
    #[pallet::error]
    pub enum Error<T> {
        /// Already exists (unique by `name`)
        Exists,
        /// Not found
        NotFound,
        /// Access denied
        Forbidden,
    }
    
    #[pallet::event]
    #[pallet::metadata(u32 = "SpecialU32")]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T: Config> {
        /// Emits when organisation created
        OrgCreate(OrgOf<T>),
        /// Emits when organisation ownership transferred
        OrgTransferOwnership(OrgOf<T>),
    }
    
    #[doc(hidden)]
    #[pallet::genesis_config]
	#[derive(Default)]
	pub struct GenesisConfig {}
    
    #[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {}
	}
    
    use org::*;
    pub mod org {
        use sp_std::prelude::*;
        use frame_support::pallet_prelude::*;
        use super::{Config, OrgRepository, Error};
        
        #[cfg(feature = "std")]
        use serde::{Serialize, Deserialize};

        #[allow(type_alias_bounds)]
        pub type OrgOf<T: Config> = Org<T::AccountId, OrgName>;
        pub type OrgName = sp_core::H160;
        
        pub fn load_org<T: Config>(
            name: &OrgName,
            who: &T::AccountId,
        )
            -> Result<OrgOf<T>, Error<T>>
        {
            let org = OrgRepository::<T>::get(name)
                .ok_or(Error::<T>::NotFound)?;
            ensure!(who == org.key(), Error::<T>::Forbidden);
            Ok(org)
        }
        
        #[derive(Debug, Clone, Eq, PartialEq, Encode, Decode)]
        #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
        pub struct Org<AccountId, Name> {
            key: AccountId,
            name: Name
        }
        impl<AccountId, Name> Org<AccountId, Name> {
            pub fn new(key: AccountId, name: Name) -> Self {
                Self { key, name }
            }
        }
        impl<AccountId, Name> Org<AccountId, Name> {
            pub fn key(&self) -> &AccountId { &self.key }
            pub fn name(&self) -> &Name { &self.name }
        }
        impl<AccountId, Name> Org<AccountId, Name> {
            pub fn update_key(&mut self, key: AccountId) {
                self.key = key;
            }
        }
    }
    
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        
        #[pallet::weight(10_000)]
        pub fn create(
            origin: OriginFor<T>,
            name: OrgName,
        ) -> DispatchResultWithPostInfo
        {
            let who = ensure_signed(origin)?;
            ensure!(!OrgRepository::<T>::contains_key(&name), Error::<T>::Exists);
            let org = Org::new(who, name);
            StorageOpsTransaction::<StorageOps<T>>::new()
                .commit(move |ops| {
                    ops.push_op(StorageOps::CreateDao(org.clone()));
                    ops.push_op(StorageOps::DepositEvent(Event::<T>::OrgCreate(org)));
                });
            Ok(Some(0).into())
        }
        
        #[pallet::weight(10_000)]
        pub fn transfer_ownership(
            origin: OriginFor<T>,
            name: OrgName,
            transfer_to: T::AccountId,
        ) -> DispatchResultWithPostInfo
        {
            let who = ensure_signed(origin)?;
            let mut org = load_org::<T>(&name, &who)?;
            org.update_key(transfer_to);
            StorageOpsTransaction::<StorageOps<T>>::new()
                .commit(move |ops| {
                    ops.push_op(StorageOps::UpdateDao(org.clone()));
                    ops.push_op(StorageOps::DepositEvent(Event::<T>::OrgTransferOwnership(org)));
                });
            Ok(Some(0).into())
        }
        
        #[pallet::weight(10_000)]
        pub fn on_behalf(
            origin: OriginFor<T>,
            name: OrgName,
            call: Box<<T as Config>::Call>,
        ) -> DispatchResultWithPostInfo
        {
            let who = ensure_signed(origin)?;
            let _ = load_org::<T>(&name, &who)?;
            call.dispatch(RawOrigin::Signed(who).into())
        }
    }
    
    // ==== Storage ====:
    
    #[pallet::storage]
    #[pallet::getter(fn get_org)]
    pub(super) type OrgRepository<T: Config> = StorageMap<_,
        Blake2_128Concat,
        OrgName,
        OrgOf<T>,
        OptionQuery
    >;
    
    use storage_ops::*;
    #[doc(no_inline)]
    /// Module contains abstractions over pallet storage operations
    pub mod storage_ops {
        use sp_std::prelude::*;
        use pallet_deip_toolkit::storage_ops::StorageOp;
        use super::{Config, Event, Pallet};
        use super::{OrgOf, OrgRepository};

        /// Storage operations
        pub enum StorageOps<T: Config> {
            /// Deposit event
            DepositEvent(Event<T>),
            /// Create proposal
            CreateDao(OrgOf<T>),
            UpdateDao(OrgOf<T>),
            
        }
        impl<T: Config> StorageOp for StorageOps<T> {
            fn exec(self) {
                match self {
                    Self::DepositEvent(e) => {
                        Pallet::<T>::deposit_event(e)
                    },
                    Self::CreateDao(dao) |
                    Self::UpdateDao(dao) => {
                        OrgRepository::<T>::insert(*dao.name(), dao);
                    }
                }
            }
        }
    }
}
