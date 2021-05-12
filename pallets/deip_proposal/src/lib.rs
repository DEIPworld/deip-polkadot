#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_system::pallet_prelude::*;
    use frame_system::RawOrigin;
    
    use frame_support::pallet_prelude::*;
    use frame_support::{Callable, Hashable};
    use frame_support::weights::{PostDispatchInfo, GetDispatchInfo};
    
    use frame_support::traits::UnfilteredDispatchable;
    
    use sp_std::prelude::*;
    use sp_std::collections::btree_map::BTreeMap;
    use sp_std::marker::PhantomData;
    use sp_std::fmt::Debug;
    
    use sp_runtime::traits::Dispatchable;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Call: Parameter +
             Dispatchable<Origin = Self::Origin, PostInfo = PostDispatchInfo> +
             GetDispatchInfo +
             From<frame_system::pallet::Call<Self>> +
             UnfilteredDispatchable<Origin = Self::Origin> +
             frame_support::dispatch::Codec;
    }
    
    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
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
        #[pallet::weight(10)]
        #[frame_support::transactional]
        fn propose(
            origin: OriginFor<T>,
            batch: Vec<ProposalBatchItemOf<T>>,
        )
            -> DispatchResultWithPostInfo
        {
            let author = ensure_signed(origin)?;
            
            let proposal = DeipProposal::<T>::new(batch, author.clone());
            
            let proposal_id = proposal.id();
            
            ensure!(!ProposalStorage::<T>::contains_key(author.clone(), proposal_id), "exists");
            
            for member in proposal.batch.iter()
                .map(|m| &m.account)
            {
                PendingProposals::<T>::mutate(member, |x| {
                    x.insert(proposal_id, author.clone());
                });
            }
            ProposalStorage::<T>::insert(author, proposal_id, proposal);
            Ok(Some(0).into())
        }
        
        #[pallet::weight(10)]
        #[frame_support::transactional]
        fn decide(
            origin: OriginFor<T>,
            proposal_id: ProposalId,
            decision: ProposalMemberDecision,
        )
            -> DispatchResultWithPostInfo
        {
            let member = ensure_signed(origin)?;
            let pending = PendingProposals::<T>::get(&member);
            let author = pending.get(&proposal_id).ok_or("not found")?;
            let mut proposal = ProposalStorage::<T>::get(author, &proposal_id).ok_or("not_found")?;
            proposal.decide(&member, decision, |prop| {
                frame_support::debug::RuntimeLogger::init();
                for x in prop.batch.iter() {
                    let ProposalBatchItemOf::<T> { account, call } = x;
                    frame_support::debug::debug!("{:?}; {:?}", &account, &call);
                    call.dispatch(RawOrigin::Signed(account).into())?;
                }
                None
            })?;
            Ok(Some(0).into())
        }
        
        #[pallet::weight(10)]
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
    
    // ==== Storage ====:
    
    #[pallet::storage]
    pub(super) type ProposalStorage<T: Config> = StorageDoubleMap<_,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        ProposalId,
        DeipProposal<T>,
        OptionQuery
    >;
    
    #[pallet::storage]
    #[pallet::getter(fn pending_proposals)]
    pub(super) type PendingProposals<T: Config> = StorageMap<_,
        Blake2_128Concat,
        T::AccountId,
        PendingProposalsMap<T>,
        ValueQuery,
        PendingProposalsMapDefault<T>
    >;
    
    #[pallet::type_value]
    pub(super) fn PendingProposalsMapDefault<T: Config>() -> PendingProposalsMap<T> { Default::default() }
    
    // ==== Logic ====:

    pub type ProposalId = [u8; 32];
    #[allow(type_alias_bounds)]
    pub type PendingProposalsMap<T: Config> = BTreeMap<ProposalId, T::AccountId>;
    
    #[allow(type_alias_bounds)]
    pub type ProposalBatchItemOf<T: Config> = BatchItemX<
        <T as frame_system::Config>::AccountId,
        <T as Config>::Call
    >;
    
    pub struct DeipProposalBuilder<T: Config> {
        _m: (PhantomData<T>, ),
        batch: Vec<ProposalBatchItemOf<T>>,
    }

    impl<T: Config> DeipProposalBuilder<T> {
        fn map_callable<'a, C: Callable<T> + 'a>(&self, c: &'a BTreeMap<Vec<u8>, C>) -> Vec<&'a C> {
            let _ = c;
            unimplemented!();
        }
    }

    #[derive(Debug, Encode, Decode)]
    pub struct DeipProposal<T: Config> {
        _m: (PhantomData<T>, ),
        batch: Vec<ProposalBatchItemOf<T>>,
        decisions: Vec<ProposalMemberDecisionState>,
        state: ProposalState,
        author: T::AccountId
    }
    
    #[derive(Debug, Clone, Copy, Eq, PartialEq, Encode, Decode)]
    pub enum ProposalState {
        Pending,
        Rejected,
        Done
    }
    
    impl<T: Config> DeipProposal<T> {
        fn id(&self) -> ProposalId {
            let author_hash = self.author.twox_256();
            let batch_hash = self.batch.encode().twox_256();
            let mut proposal_id_source = Vec::<u8>::with_capacity(64);
            proposal_id_source.extend(author_hash.iter());
            proposal_id_source.extend(batch_hash.iter());
            proposal_id_source.twox_256()
        }
        
        pub fn builder(batch: Vec<ProposalBatchItemOf<T>>) -> DeipProposalBuilder<T> {
            DeipProposalBuilder { _m: Default::default(), batch }
        }
        
        pub fn new(batch: Vec<ProposalBatchItemOf<T>>, author: T::AccountId) -> Self {
            let mut decisions = Vec::with_capacity(batch.len());
            decisions.extend(sp_std::iter::repeat(ProposalMemberDecisionState::Pending).take(batch.len()));
            Self {
                _m: (Default::default()),
                batch,
                decisions,
                state: ProposalState::Pending,
                author
            }
        }
        
        #[frame_support::require_transactional]
        pub fn decide<'a, R, F: FnMut(&'a mut Self) -> Option<R>>(
            &'a mut self,
            member: &T::AccountId,
            decision: ProposalMemberDecision,
            on_all_approved: F
        )
            -> Result<Option<R>, &'static str>
        {
            let (item, state) = self.batch.iter_mut()
                .zip(self.decisions.iter_mut())
                .find(|x| &x.0.account == member)
                .ok_or("not a member")?;
            if !matches!(self.state, ProposalState::Pending) {
                return Err("can't decide on resolved proposal")
            }
            let state = state.decide(decision).or(Err("member already made decision"))?;
            match state {
                ProposalMemberDecisionState::Declined => {
                    self.state = ProposalState::Rejected;
                    Ok(None)
                },
                ProposalMemberDecisionState::Approved => {
                    let all_approved = self.decisions.iter()
                        .all(|x: &ProposalMemberDecisionState| matches!(x, ProposalMemberDecisionState::Approved));
                    if all_approved {
                        self.state = ProposalState::Done;
                        Ok(on_all_approved(self))
                    } else {
                        Ok(None)
                    }
                },
                _ => Ok(None),
            }
        }
    }
    
    #[derive(Debug, Clone, Eq, PartialEq, Encode, Decode)]
    pub struct BatchItemX<Account, CallT> {
        account: Account,
        call: CallT,
    }
    
    #[derive(Debug, Clone, Copy, Eq, PartialEq, Encode, Decode)]
    pub enum ProposalMemberDecisionState {
        Pending,
        Approved,
        Declined
    }
    impl ProposalMemberDecisionState {
        fn decide(&mut self, decision: ProposalMemberDecision) -> Result<Self, Self> {
            match self {
                Self::Pending => {
                    *self = match decision {
                        ProposalMemberDecision::Approve => Self::Approved,
                        ProposalMemberDecision::Decline => Self::Declined,
                    };
                    Ok(*self)
                },
                other => Err(*other),
            }
        }
    }
    
    #[derive(Debug, Clone, Copy, Eq, PartialEq, Encode, Decode)]
    pub enum ProposalMemberDecision {
        Approve,
        Decline
    }
    
    pub enum ProposalStatus {
        Pending,
        ReadyToExec,
        Done
    }
}
