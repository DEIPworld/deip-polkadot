#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
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
    // use sp_std::marker::PhantomData;
    use sp_std::fmt::Debug;
    
    use sp_runtime::traits::Dispatchable;
    
    use pallet_multisig;

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_multisig::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Call: Parameter +
             Dispatchable<Origin = Self::Origin, PostInfo = PostDispatchInfo> +
             GetDispatchInfo +
             From<frame_system::pallet::Call<Self>> +
             UnfilteredDispatchable<Origin = Self::Origin> +
             frame_support::dispatch::Codec + 
             IsSubType<Call<Self>>;
        // type NodeRuntimeCall: GetCallMetadata;
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
        /// Reach depth limit of nested proposals
        ReachDepthLimit
    }
    
    #[pallet::event]
    #[pallet::metadata(u32 = "SpecialU32")]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T: Config> {
        Proposed(T::AccountId, ProposalId),
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
    
    use depth_limit::*; 
    mod depth_limit {
        use sp_std::collections::vec_deque::VecDeque;
        use sp_std::prelude::*;
        use sp_std::iter::{Peekable, Iterator};
        use frame_support::traits::IsSubType;
        use super::{Config, Call, ProposalBatch, ProposalBatchItemOf, ProposalId};

        pub enum BatchItemKind<'a, T: Config> {
            Propose(&'a ProposalBatch<T>),
            Decide(&'a ProposalId),
            Other
        }
        impl<'a, T: Config> BatchItemKind<'a, T> {
            pub fn kind(item: &'a ProposalBatchItemOf<T>) -> BatchItemKind<'a, T> {
                match item.call.is_sub_type() {
                    Some(Call::propose(batch)) => {
                        Self::Propose(batch)
                    },
                    Some(Call::decide(proposal_id, _decision)) => {
                        Self::Decide(proposal_id)
                    },
                    _ => Self::Other
                }
            }
        }
        
        pub enum NestedProposalsAssertions {
            DepthLimit,
            SelfReference
        }
        // pub fn assert_nested_proposals<T: Config>(
        //     root: &ProposalBatch<T>,
        //     depth_limit: usize,
        //     root_id: &ProposalId
        // )
        //     -> Option<NestedProposalsAssertions>
        // {
        //     let mut res = None;
        //     let visitor = |node: Node<&ProposalBatchItemOf<T>>| {
        //         if node.depth > depth_limit {
        //             res = Some(NestedProposalsAssertions::DepthLimit);
        //         }
        //         if let BatchItemKind::Decide(ref proposal_id) = BatchItemKind::<T>::kind(node.data) {
        //             if proposal_id == &root_id {
        //                 res = Some(NestedProposalsAssertions::SelfReference);
        //             }
        //         }
        //     };
        //     traverse_nested_proposals(root, visitor);
        //     res
        // }
        
        pub struct Node<Data> {
            depth: usize,
            data: Data,
        }
        
        pub fn traverse_nested_proposals<'a, T: Config>(
            root: &'a ProposalBatch<T>,
            mut visit_node: impl FnMut(Node<&'a ProposalBatchItemOf<T>>)
        )
        {
            let mut stack = VecDeque::<Peekable<Box<dyn Iterator<Item=&ProposalBatchItemOf<T>>>>>::new();
            let boxed: Box<dyn Iterator<Item=&ProposalBatchItemOf<T>>> = Box::new(root.iter());
            stack.push_front(boxed.peekable());
            let mut depth: usize = 1;
            while !stack.is_empty() {
                depth = stack.len();
                let cur = stack.front_mut().unwrap();
                while let Some(nested) = cur.next() {
                    visit_node(Node { depth, data: nested });
                    match BatchItemKind::<T>::kind(nested) {
                        BatchItemKind::Propose(batch) => {
                            let boxed: Box<dyn Iterator<Item=&ProposalBatchItemOf<T>>> = Box::new(batch.iter());
                            stack.push_front(boxed.peekable());
                            break
                        },
                        _ => ()
                    }
                }
                if cur.peek().is_none() {
                    stack.pop_front();
                }
            }
        }

        pub fn nested_proposal_depth<T: Config>(top: &ProposalBatch<T>, depth_limit: Option<usize>) -> Option<usize> {
            // let mut stack = VecDeque::<Box<dyn Iterator<Item=&ProposalBatch<T>>>>::new();
            // // stack.push_front(Box::new(top.iter().filter_map(|x| is_proposal::<T>(&x.call))));
            // stack.push_front(Box::new(top.iter()));
            let mut depth: usize = 1;
            // while !stack.is_empty() {
            //     depth = depth.max(stack.len());
            //     if let Some(ref limit) = depth_limit {
            //         if &depth > limit {
            //             return None;
            //         }
            //     }
            //     let cur = stack.front_mut().unwrap();
            //     if let Some(nested) = cur.next() {
            //         // stack.push_front(Box::new(nested.iter().filter_map(|x| is_proposal::<T>(&x.call))));
            //         stack.push_front(Box::new(nested.iter()));
            //     } else {
            //         stack.pop_front();
            //     }
            // }
            Some(depth)
        }
    }
    
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(10)]
        fn propose(
            origin: OriginFor<T>,
            batch: Vec<ProposalBatchItemOf<T>>,
        )
            -> DispatchResultWithPostInfo
        {
            let author = ensure_signed(origin)?;
            
            frame_support::debug::RuntimeLogger::init();
            
            let depth = nested_proposal_depth::<T>(&batch, Some(2));
            frame_support::debug::debug!("DEPTH: {:?}", depth);
            ensure!(depth.is_some(), Error::<T>::ReachDepthLimit);
            
            let proposal = DeipProposal::<T>::new(
                batch,
                author.clone(),
                <DeipProposal<T>>::timepoint
            );
            
            let members: Vec<T::AccountId> = proposal.batch.iter()
                .map(|m| m.account.clone())
                .collect();

            StorageOpsTransaction::<T, _>::new()
                .commit(move |ops| {
                    let proposal_id = proposal.id;
                    ops.push_op(StorageOps::PersistProposal(proposal));
                    ops.push_op(StorageOps::AddPendingProposal {
                        members,
                        proposal_id,
                        author: author.clone(),
                    });
                    ops.push_op(StorageOps::DepositEvent(Event::<T>::Proposed(author, proposal_id)));
                });
            
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
            let maybe_batch_exec_result: Option<DispatchResultWithPostInfo> =
                StorageOpsTransaction::<T, _>::new()
                    .commit(|ops| {
                        proposal.decide(
                            &member,
                            decision,
                            Self::exec_batch,
                            ops,
                        )
                    })?;
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
    
    #[derive(Debug, Encode, Decode)]
    pub struct DeipProposal<T: Config> {
        id: ProposalId,
        batch: ProposalBatch<T>,
        decisions: BTreeMap<T::AccountId, ProposalMemberDecisionState>,
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
    
    use storage_ops::*;
    mod storage_ops {
        use sp_std::collections::vec_deque::VecDeque;
        use sp_std::marker::PhantomData;
        use sp_std::prelude::*;
        use super::{
            Config, DeipProposal, Event, ProposalId,
            ProposalStorage, Pallet, PendingProposals};
        
        pub enum StorageOps<T: Config> {
            PersistProposal(DeipProposal<T>),
            DepositEvent(Event<T>),
            AddPendingProposal {
                members: Vec<T::AccountId>,
                proposal_id: ProposalId,
                author: T::AccountId,
            },
        }
        
        pub struct StorageOpsQueue<T>(VecDeque<T>);
        impl<T> StorageOpsQueue<T> {
            pub fn push_op(&mut self, op: T) -> &mut Self { self.0.push_back(op); self }
            fn pop_op(&mut self) -> Option<T> { self.0.pop_front() }
        }

        pub struct StorageOpsTransaction<T: Config, Ops=StorageOps<T>>(StorageOpsQueue<Ops>, PhantomData<T>);
        impl<T: Config> StorageOpsTransaction<T, StorageOps<T>> {
            pub fn new() -> Self { Self(StorageOpsQueue(VecDeque::new()), Default::default()) }
            pub fn commit<R>(mut self, transactional: impl FnOnce(&mut StorageOpsQueue<StorageOps<T>>) -> R) -> R {
                let result = transactional(&mut self.0);
                while let Some(op) = self.0.pop_op() {
                    match op {
                        StorageOps::PersistProposal(proposal) => {
                            <ProposalStorage<T>>::insert(proposal.author.clone(), proposal.id, proposal);
                        },
                        StorageOps::DepositEvent(event) => {
                            <Pallet<T>>::deposit_event(event);
                        },
                        StorageOps::AddPendingProposal {
                            members,
                            proposal_id,
                            author
                        } => {
                            for m in members {
                                PendingProposals::<T>::mutate(m, |x| {
                                    x.insert(proposal_id, author.clone());
                                });
                            }
                        }
                    }
                }
                result
            }
        }
    }
    
    impl<T: Config> DeipProposal<T> {
        fn timepoint() -> ProposalId { pallet_multisig::Module::<T>::timepoint().twox_256() }
        
        fn new(
            batch: Vec<ProposalBatchItemOf<T>>,
            author: T::AccountId,
            timepoint: impl FnOnce() -> ProposalId
        )
            -> Self
        {
            let decisions = BTreeMap::from_iter(
                batch.iter().map(|x| (
                    x.account.clone(),
                    ProposalMemberDecisionState::Pending
                ))
            );
            Self {
                id: timepoint(),
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
            storage_ops: &mut StorageOpsQueue<StorageOps<T>>
        )
            -> Result<Option<BatchExec::Output>, Error<T>>
            where
                BatchExec: FnOnce(ProposalBatch<T>) -> DispatchResultWithPostInfo
        {
            let member_decision_state = self.decisions.get_mut(member).ok_or(Error::<T>::NotAMember)?;
            
            ensure!(matches!(self.state, ProposalState::Pending), Error::<T>::AlreadyResolved);
            
            let batch_exec_result = match member_decision_state.decide(decision) {
                Err(_) | Ok(ProposalMemberDecisionState::Pending) => Err(Error::<T>::AlreadyDecide)?,
                Ok(ProposalMemberDecisionState::Declined) => {
                    self.state = ProposalState::Rejected;
                    storage_ops.push_op(StorageOps::DepositEvent(Event::<T>::Rejected(member.clone())));
                    None
                },
                Ok(ProposalMemberDecisionState::Approved) => {
                    storage_ops.push_op(StorageOps::DepositEvent(Event::<T>::Approved(member.clone())));
                    if self.ready_to_exec() {
                        let batch_exec_result = batch_exec(self.batch.clone());
                        self.state = if let Err(ref err) = batch_exec_result {
                            storage_ops.push_op(StorageOps::DepositEvent(Event::<T>::Failed(err.error.clone())));
                            ProposalState::Failed(err.error.clone())
                        } else {
                            storage_ops.push_op(StorageOps::DepositEvent(Event::<T>::Done));
                            ProposalState::Done
                        };
                        Some(batch_exec_result)
                    } else { None }
                },
            };
            
            storage_ops.push_op(StorageOps::PersistProposal(self));
            
            Ok(batch_exec_result)
        }
        
        fn ready_to_exec(&self) -> bool {
            let approved = self.decisions.values()
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
