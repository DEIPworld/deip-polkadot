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
    
    use sp_runtime::{MultiSigner, traits::{Dispatchable, IdentifyAccount}};
    use frame_support::dispatch::DispatchResult;
    
    // use sp_core::crypto::Pair;
    // use sp_core::ed25519;
    
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
        ///
        KeySourceMismatch
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
        use super::{Config, OrgRepository, Error, OrgLookup};
        
        #[cfg(feature = "std")]
        use serde::{Serialize, Deserialize};
        use frame_system::Key;
        use crate::Error::KeySourceMismatch;

        #[allow(type_alias_bounds)]
        pub type OrgOf<T: Config> = Org<T::AccountId, OrgName>;
        pub type OrgName = sp_core::H160;
        
        pub enum KeyType<'a, K> {
            Members(&'a K),
            Own(&'a K)
        }
        impl<'a, K> KeyType<'a, K> {
            pub fn members(k: &'a K) -> Self { Self::Members(k) }
            pub fn own(k: &'a K) -> Self { Self::Own(k) }
        }
        pub trait MatchKey<T: Config> {
            fn match_key(&self, org: &OrgOf<T>) -> bool;
        }
        impl<T: Config> MatchKey<T> for KeyType<'_, T::AccountId> {
            fn match_key(&self, org: &OrgOf<T>) -> bool {
                match self {
                    Self::Members(k) => {
                        *k == org.key()
                    },
                    Self::Own(k) => {
                        *k == org.org_key()
                    },
                }
            }
        }
        
        pub enum LoadBy<'a, AccountId> {
            Name { name: &'a OrgName, who: KeyType<'a, AccountId> },
            OrgKey { org_key: &'a AccountId }
        }
        
        pub fn load_org<T: Config>(
            q: LoadBy<'_, T::AccountId>,
        )
            -> Result<OrgOf<T>, Error<T>>
        {
            let (org, who) = match q {
                LoadBy::Name { name, who } => {
                    let org = OrgRepository::<T>::get(name)
                        .ok_or(Error::<T>::NotFound)?;
                    (org, who)
                },
                LoadBy::OrgKey { org_key } => {
                    let name = OrgLookup::<T>::get(org_key)
                        .ok_or(Error::<T>::NotFound)?;
                    let org = OrgRepository::<T>::get(&name)
                        .ok_or(Error::<T>::NotFound)?;
                    (org, KeyType::Own(org_key))
                },
            };
            ensure!(MatchKey::<T>::match_key(&who, &org), Error::<T>::Forbidden);
            Ok(org)
        }
        
        #[derive(Debug, Clone, Eq, PartialEq, Encode, Decode)]
        #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
        pub struct KeySource<AccountId> {
            signatories: Vec<AccountId>,
            threshold: u16 
        }
        #[derive(Debug, Clone, Eq, PartialEq, Encode, Decode)]
        pub struct InputKeySource<AccountId> {
            pub signatories: Vec<AccountId>,
            pub threshold: u16 
        }
        pub enum KeySourceAssert {
            EmptySignatories,
            /// We expect signatures list with exactly one element for plain account
            PlainAccountExpect,
            ThresholdMismatch,
            OriginMismatch,
        }
        impl<T: Config> From<KeySourceAssert> for Error<T> {
            fn from(source: KeySourceAssert) -> Self {
                Error::<T>::KeySourceMismatch
            }
        }
        pub trait AssertKeySource<T: Config> {
            fn assert(self, origin: &T::AccountId) -> Result<KeySource<T::AccountId>, KeySourceAssert>;
        }
        pub fn multi_account_id<T: Config>(who: &[T::AccountId], threshold: u16) -> T::AccountId {
            let entropy = (b"modlpy/utilisuba", who, threshold).using_encoded(sp_io::hashing::blake2_256);
            T::AccountId::decode(&mut &entropy[..]).unwrap_or_default()
        }
        impl<T: Config> AssertKeySource<T> for InputKeySource<T::AccountId> {
            fn assert(self, origin: &T::AccountId) -> Result<KeySource<T::AccountId>, KeySourceAssert>
            {
                let Self { mut signatories, threshold } = self;
                ensure!(!signatories.is_empty(), KeySourceAssert::EmptySignatories);
                
                // zero threshold adjusts plain non-multisig account
                let key = if threshold == 0 {
                    ensure!(signatories.len() == 1, KeySourceAssert::PlainAccountExpect);
                    signatories.get(0).unwrap().clone()
                } else {
                    ensure!(threshold as usize <= signatories.len(), KeySourceAssert::ThresholdMismatch);
                    signatories.sort();
                    multi_account_id::<T>(signatories.as_slice(), threshold)
                };
                if origin == &key {
                    Ok(KeySource { signatories, threshold })
                } else {
                    Err(KeySourceAssert::OriginMismatch)
                }
            }
        }
        
        #[derive(Debug, Clone, Eq, PartialEq, Encode, Decode)]
        #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
        pub struct Org<AccountId, Name> {
            /// Members aka "control" key. Not fixed, may changes in future
            members_key: AccountId,
            /// Details of control key: multi-sig or plain account
            members_key_source: KeySource<AccountId>,
            /// Unique organisation name aka ID
            name: Name,
            /// Own key of organization for keeping assets,
            /// to signing of an extrinsics calls dispatched on behalf of organization etc ..
            /// Must be generated internally when organisation will be created,
            /// nobody knows private half of this key
            org_key: AccountId
        }
        impl<AccountId, Name> Org<AccountId, Name> {
            pub fn new(
                members_key: AccountId,
                members_key_source: KeySource<AccountId>,
                name: Name,
                org_key: AccountId,
            )
                -> Self
            {
                Self { members_key, members_key_source, name, org_key }
            }
        }
        impl<AccountId, Name> Org<AccountId, Name> {
            pub fn key(&self) -> &AccountId { &self.members_key }
            pub fn key_source(&self) -> &KeySource<AccountId>{ &self.members_key_source }
            pub fn name(&self) -> &Name { &self.name }
            pub fn org_key(&self) -> &AccountId { &self.org_key }
        }
        impl<AccountId, Name> Org<AccountId, Name> {
            pub fn update_members_key(&mut self, members_key: AccountId, source: KeySource<AccountId>) {
                self.members_key = members_key;
                self.members_key_source = source;
            }
        }
    }
    
    impl<T: Config> Pallet<T> {
        pub fn org_key(org_name: &OrgName) -> T::AccountId {
            let entropy = (org_name.as_bytes()).using_encoded(sp_io::hashing::blake2_256);
            T::AccountId::decode(&mut &entropy[..]).unwrap_or_default()
        }
    }
    
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        
        #[pallet::weight(10_000)]
        pub fn create(
            origin: OriginFor<T>,
            name: OrgName,
            key_source: InputKeySource<T::AccountId>,
        )
            -> DispatchResultWithPostInfo
            where KeySourceAssert: Into<Error<T>>,
                  InputKeySource<T::AccountId>: AssertKeySource<T>,
        {
            let who = ensure_signed(origin)?;
            let key_source = key_source.assert(&who)
                .map_err(|x| x.into())?;
            ensure!(!OrgRepository::<T>::contains_key(&name), Error::<T>::Exists);
            let org_key = Self::org_key(&name);
            let org = OrgOf::<T>::new(
                who,
                key_source,
                name,
                org_key
            );
            StorageOpsTransaction::<StorageOps<T>>::new()
                .commit(move |ops| {
                    ops.push_op(StorageOps::CreateOrg(org.clone()));
                    ops.push_op(StorageOps::DepositEvent(Event::<T>::OrgCreate(org)));
                });
            Ok(Some(0).into())
        }
        
        #[pallet::weight(10_000)]
        pub fn transfer_ownership(
            origin: OriginFor<T>,
            transfer_to: T::AccountId,
            key_source: InputKeySource<T::AccountId>,
        )
            -> DispatchResultWithPostInfo
            where InputKeySource<T::AccountId>: AssertKeySource<T>,
                  KeySourceAssert: Into<Error<T>>
        {
            let who = ensure_signed(origin)?;
            let mut org = load_org::<T>(LoadBy::OrgKey { org_key: &who })?;
            let key_source = key_source.assert(&transfer_to)
                .map_err(|x| x.into())?;
            org.update_members_key(transfer_to, key_source);
            StorageOpsTransaction::<StorageOps<T>>::new()
                .commit(move |ops| {
                    ops.push_op(StorageOps::UpdateOrg(org.clone()));
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
            let org = load_org::<T>(LoadBy::Name {
                name: &name,
                who: KeyType::Members(&who)
            })?;
            call.dispatch(RawOrigin::Signed(org.org_key().clone()).into())
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
    
    #[pallet::storage]
    #[pallet::getter(fn lookup_org)]
    pub(super) type OrgLookup<T: Config> = StorageMap<_,
        Blake2_128Concat,
        T::AccountId,
        OrgName,
        OptionQuery
    >;
    
    use storage_ops::*;
    #[doc(no_inline)]
    /// Module contains abstractions over pallet storage operations
    pub mod storage_ops {
        use sp_std::prelude::*;
        use pallet_deip_toolkit::storage_ops::StorageOp;
        use super::{Config, Event, Pallet};
        use super::{OrgOf, OrgRepository, OrgLookup};

        /// Storage operations
        pub enum StorageOps<T: Config> {
            /// Deposit event
            DepositEvent(Event<T>),
            /// Create org
            CreateOrg(OrgOf<T>),
            /// Update org
            UpdateOrg(OrgOf<T>),
            
        }
        impl<T: Config> StorageOp for StorageOps<T> {
            fn exec(self) {
                match self {
                    Self::DepositEvent(e) => {
                        Pallet::<T>::deposit_event(e)
                    },
                    Self::CreateOrg(org) => {
                        OrgLookup::<T>::insert(org.org_key().clone(), org.name().clone());
                        OrgRepository::<T>::insert(*org.name(), org);
                    }
                    Self::UpdateOrg(org) => {
                        OrgRepository::<T>::insert(*org.name(), org);
                    }
                }
            }
        }
    }
}
