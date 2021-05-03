#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    pub use frame_support::pallet_prelude::*;
    pub use frame_system::pallet_prelude::*;
    
    use sp_std::prelude::*;
    use sp_std::collections::btree_map::BTreeMap;
    use sp_std::marker::PhantomData;
    
    use frame_support::Callable;
    
    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }
    
    #[pallet::pallet]
    #[pallet::generate_store(pub trait Store)]
    pub struct Pallet<T>(_);
    
    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}
    
    #[pallet::event]
    #[pallet::metadata(u32 = "SpecialU32")]
    pub enum Event<T: Config> {
        Proposed(u32, T::AccountId),
    }
    
    #[pallet::genesis_config]
	#[derive(Default)]
	pub struct GenesisConfig {
		// _myfield: u32,
	}
    
    #[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {}
	}
    
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10_000)]
        fn propose(
            origin: OriginFor<T>,
            batch: Vec<BatchItemX<T::AccountId, Vec<u8>>>,
        )
            -> DispatchResultWithPostInfo
        {
            let _ = origin;
            let _ = batch;
            unimplemented!();
        }
        
        #[pallet::weight(10_000)]
        fn approve(
            origin: OriginFor<T>,
        )
            -> DispatchResultWithPostInfo
        {
            let _ = origin;
            unimplemented!();
        }
        
        #[pallet::weight(10_000)]
        fn explore(
            origin: OriginFor<T>,
        )
            -> DispatchResultWithPostInfo
        {
            let _ = origin;
            frame_support::debug::RuntimeLogger::init();
            frame_support::debug::debug!("call_functions: {:?}", <Pallet<T>>::call_functions());
            // unimplemented!();
            Result::Ok(frame_support::dispatch::PostDispatchInfo { actual_weight: None, pays_fee: Default::default() })
        }
    }
    
    /// Logic:

    pub struct DeipProposalBuilder<T: Config> {
        _m: (PhantomData<T>, ),
        batch: Vec<BatchItemX<T::AccountId, Vec<u8>>>,
    }

    impl<T: Config> DeipProposalBuilder<T> {
        fn map_callable<'a, C: Callable<T> + 'a>(&self, c: &'a BTreeMap<Vec<u8>, C>) -> Vec<&'a C> {
            let _ = c;
            unimplemented!();
        }
    }

    pub struct DeipProposal<T: Config> {
        _m: (PhantomData<T>, ),
    }

    impl<T: Config> DeipProposal<T> {
        pub fn builder(batch: Vec<BatchItemX<T::AccountId, Vec<u8>>>) -> DeipProposalBuilder<T> {
            DeipProposalBuilder { _m: Default::default(), batch }
        }
    }

    #[derive(Debug, Clone, Eq, PartialEq, Encode, Decode)]
    pub struct BatchItemX<AccountId, Extrinsic> {
        account: AccountId,
        extrinsic: Extrinsic,
    }
    // 
    // pub struct BatchItem<AccountId> where Self: BatchItemT {
    //     account: AccountId,
    //     extrinsic: <Self as BatchItemT>::Extrinsic
    // }
    // pub trait BatchItemT {
    //     type Extrinsic;
    // }
    // impl<T: Config> BatchItemT for BatchItem<T::AccountId> {
    //     type Extrinsic = ();
    // }
    // 
    // pub struct Batch<T: Config, Item: BatchItemT> {
    //     items: Vec<BatchItem<T::AccountId>>
    // }
    // 
    // pub trait BatchT<Item: BatchItemT> {
    //     fn add_item(&mut self, item: Item);
    // }
    // 
    // impl<T: Config, Item: BatchItemT> BatchT<Item> for Batch<T, Item> {
    //     fn add_item(&mut self, item: Item) {
    //         self.items.push(item);
    //     }
    // }

}
