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
    
    #[pallet::error]
    pub enum Error<T> {
        /// Proposal not found
        NotFound,
        /// Proposal already exist
        AlreadyExist,
        /// Current origin is not a member of Proposal
        NotAMember,
        /// Proposal already resolved
        AlreadyResolved,
        /// Member already made decision on Proposal
        AlreadyDecide,
    }
    
    #[pallet::event]
    #[pallet::metadata(u32 = "SpecialU32")]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T: Config> {
        Proposed(T::AccountId),
        Rejected(T::AccountId),
        Approved(T::AccountId),
        Done,
        Failed(DispatchError)
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
            
            ensure!(!ProposalStorage::<T>::contains_key(author.clone(), proposal_id), Error::<T>::AlreadyExist);
            
            for member in proposal.batch.iter()
                .map(|m| &m.account)
            {
                PendingProposals::<T>::mutate(member, |x| {
                    x.insert(proposal_id, author.clone());
                });
            }
            ProposalStorage::<T>::insert(author.clone(), proposal_id, proposal);
            Self::deposit_event(Event::<T>::Proposed(author));
            Ok(Some(0).into())
        }

        #[pallet::weight(10)]
        fn decide(
            origin: OriginFor<T>,
            proposal_id: ProposalId,
            decision: ProposalMemberDecision,
        )
            -> DispatchResultWithPostInfo
        {
            let member = ensure_signed(origin)?;
            let pending = PendingProposals::<T>::get(&member);
            let author = pending.get(&proposal_id).ok_or(Error::<T>::NotFound)?;
            let proposal = ProposalStorage::<T>::get(author, &proposal_id).ok_or(Error::<T>::NotFound)?;
            let result: (Transaction<T>, Option<DispatchResultWithPostInfo>) = proposal.decide(
                &member,
                decision,
                Self::exec_batch
            )?;
            let (transaction, maybe_batch_exec_result) = result;
            transaction.commit();
            if let Some(batch_exec_result) = maybe_batch_exec_result {
                let _batch_exec_ok = batch_exec_result?;
            }
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

    impl<T: Config> Pallet<T> {
        #[frame_support::transactional]
        fn exec_batch(batch: ProposalBatch<T>) -> DispatchResultWithPostInfo
        {
            frame_support::debug::RuntimeLogger::init();
            for x in batch {
                let ProposalBatchItemOf::<T> { account, call } = x;
                frame_support::debug::debug!("{:?}; {:?}", &account, &call);
                call.dispatch(RawOrigin::Signed(account).into())?;
            }
            Ok(Some(0).into())
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
    
    #[allow(type_alias_bounds)]
    pub type ProposalBatch<T: Config> = Vec<ProposalBatchItemOf<T>>;
    
    pub struct DeipProposalBuilder<T: Config> {
        _m: (PhantomData<T>, ),
        #[allow(dead_code)]
        batch: Vec<ProposalBatchItemOf<T>>,
    }

    impl<T: Config> DeipProposalBuilder<T> {
        #[allow(dead_code)]
        fn map_callable<'a, C: Callable<T> + 'a>(&self, c: &'a BTreeMap<Vec<u8>, C>) -> Vec<&'a C> {
            let _ = c;
            unimplemented!();
        }
    }

    #[derive(Debug, Encode, Decode)]
    pub struct DeipProposal<T: Config> {
        _m: (PhantomData<T>, ),
        batch: ProposalBatch<T>,
        decisions: Vec<ProposalMemberDecisionState>,
        state: ProposalState,
        author: T::AccountId
    }
    
    #[derive(Debug, Clone, Copy, Eq, PartialEq, Encode, Decode)]
    pub enum ProposalState {
        Pending,
        Rejected,
        Done,
        Failed(DispatchError)
    }
    
    enum TransactionOps<T: Config> {
        Persist(DeipProposal<T>),
        PutEvent(Event<T>)
    }
    pub struct Transaction<T: Config>(Vec<TransactionOps<T>>);
    impl<T: Config> Transaction<T> {
        fn new() -> Self { Self(Vec::new()) }
        fn push_op(&mut self, op: TransactionOps<T>) { self.0.push(op); }
        fn commit(self) {
            for op in self.0 {
                match op {
                    TransactionOps::Persist(proposal) => {
                       <ProposalStorage<T>>::insert(proposal.author.clone(), proposal.id(), proposal); 
                    },
                    TransactionOps::PutEvent(event) => {
                        <Pallet<T>>::deposit_event(event);
                    }
                }
            }
        }
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
        
        fn new(batch: Vec<ProposalBatchItemOf<T>>, author: T::AccountId) -> Self {
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
        
        fn decide<BatchExec>(
            mut self,
            member: &T::AccountId,
            decision: ProposalMemberDecision,
            batch_exec: BatchExec,
        )
            -> Result<(Transaction<T>, Option<BatchExec::Output>), Error<T>>
            where
                BatchExec: FnOnce(ProposalBatch<T>) -> DispatchResultWithPostInfo
        {
            let mut transaction = <Transaction<T>>::new();
            
            let member_decision_state = self.batch.iter()
                .zip(self.decisions.iter_mut())
                .find(|x| &x.0.account == member)
                .map(|(_, x)| x)
                .ok_or(Error::<T>::NotAMember)?;
            
            ensure!(matches!(self.state, ProposalState::Pending), Error::<T>::AlreadyResolved);
            
            let batch_exec_result = match member_decision_state.decide(decision) {
                Err(_) | Ok(ProposalMemberDecisionState::Pending) => Err(Error::<T>::AlreadyDecide)?,
                Ok(ProposalMemberDecisionState::Declined) => {
                    self.state = ProposalState::Rejected;
                    transaction.push_op(TransactionOps::PutEvent(Event::<T>::Rejected(member.clone())));
                    None
                },
                Ok(ProposalMemberDecisionState::Approved) => {
                    transaction.push_op(TransactionOps::PutEvent(Event::<T>::Approved(member.clone())));
                    if self.ready_to_exec() {
                        let batch_exec_result = batch_exec(self.batch.clone());
                        self.state = if let Err(ref err) = batch_exec_result {
                            transaction.push_op(TransactionOps::PutEvent(Event::<T>::Failed(err.error.clone())));
                            ProposalState::Failed(err.error.clone())
                        } else {
                            transaction.push_op(TransactionOps::PutEvent(Event::<T>::Done));
                            ProposalState::Done
                        };
                        Some(batch_exec_result)
                    } else { None }
                },
            };
            
            transaction.push_op(TransactionOps::Persist(self));
            
            Ok((transaction, batch_exec_result))
        }
        
        fn ready_to_exec(&self) -> bool {
            let approved = self.decisions.iter()
                .all(|x: &ProposalMemberDecisionState| {
                    matches!(x, ProposalMemberDecisionState::Approved)
                });
            approved && matches!(self.state, ProposalState::Pending)
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
}
