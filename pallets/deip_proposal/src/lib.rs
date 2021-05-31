//! # Proposal Module
//! A module for doing multi-account batch-transaction.
//! 
//! - [`Config`](./trait.Config.html)
//! - [`Call`](./enum.Call.html)
//!
//! ## Overview
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! * `propose` - Create proposal. Corresponds to `CREATE_PROPOSAL` operation of DEIP protocol
//! * `decide` - Make decision on proposal being a member of it. Corresponds to `UPDATE_PROPOSAL` operation of DEIP protocol
//! * `delete` - Corresponds to `DELETE_PROPOSAL` operation of DEIP protocol. (Not implemented)
//!
//! [`Call`]: ./enum.Call.html
//! [`Config`]: ./trait.Config.html

#![cfg_attr(not(feature = "std"), no_std)]

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
    
    use pallet_multisig;

    /// Configuration trait
    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_multisig::Config {
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
        /// Proposal not found
        NotFound,
        /// Proposal already exist
        AlreadyExist,
        /// Current origin is not a member of Proposal
        NotAMember,
        /// Proposal already resolved (done, failed or rejected)
        AlreadyResolved,
        /// Reach depth limit of nested proposals
        ReachDepthLimit,
        /// Self-referential proposal
        SelfReferential
    }
    
    #[pallet::event]
    #[pallet::metadata(u32 = "SpecialU32")]
    #[pallet::generate_deposit(fn deposit_event)]
    pub enum Event<T: Config> {
        /// Emits when proposal created
        Proposed {
            author: T::AccountId,
            batch: ProposalBatch<T>,
            proposal_id: ProposalId
        },
        /// Emits when proposal approved by it's member
        Approved {
            member: T::AccountId,
            proposal_id: ProposalId
        },
        /// Emits when member revokes his approval
        RevokedApproval {
            member: T::AccountId,
            proposal_id: ProposalId
        },
        /// Emits when proposal resolved (rejected / done / failed)
        Resolved {
            member: T::AccountId,
            proposal_id: ProposalId
        }
    }
    
    #[doc(hidden)]
    #[pallet::genesis_config]
	#[derive(Default)]
	pub struct GenesisConfig {}
    
    #[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {}
	}
    
    pub use proposal_assertions::*;
    /// Module contains some proposal assertions
    mod proposal_assertions {
        use super::{
            Config, DeipProposal, BatchTreeNode, StopTraverse,
            BatchItemKind, ProposalBatchItemOf, traverse_batch_tree
        };
        /// Proposal assertions enumeration
        pub enum ProposalAssertions {
            /// Reached depth limit of nested proposals
            DepthLimit,
            /// Proposal has self-references
            SelfReference
        }
        /// Perform some assertions on proposal object
        pub fn assert_proposal<T: Config>(
            proposal: &DeipProposal<T>,
            depth_limit: usize,
        )
            -> Option<ProposalAssertions>
        {
            let mut res = None;
            traverse_batch_tree::<T, _>(&proposal.batch, |node: BatchTreeNode<&ProposalBatchItemOf<T>>| {
                if node.depth > depth_limit {
                    res = Some(ProposalAssertions::DepthLimit);
                    return Some(StopTraverse)
                }
                if let BatchItemKind::Decide(proposal_id) = BatchItemKind::<T>::kind(node.data) {
                    if proposal_id == &proposal.id {
                        res = Some(ProposalAssertions::SelfReference);
                        return Some(StopTraverse)
                    }
                }
                None
            });
            res
        }
    }
    
    use batch_item_kind::*;
    #[doc(no_inline)]
    /// Module contains classification of the proposal batch item
    pub mod batch_item_kind {
        use frame_support::traits::IsSubType;
        use super::{Config, Call, ProposalBatch, ProposalId, ProposalBatchItemOf};
        
        /// Batch item kinds
        pub enum BatchItemKind<'a, T: Config> {
            /// Batch item contains `propose` dispatchable
            Propose(&'a ProposalBatch<T>),
            /// Batch item contains `decide` dispatchable
            Decide(&'a ProposalId),
            Other
        }
        impl<'a, T: Config> BatchItemKind<'a, T> {
            /// Classify proposal batch item
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
    }
    
    use batch_tree::*;
    /// Module contains operations on nested proposals
    #[doc(no_inline)]
    pub mod batch_tree {
        use sp_std::collections::vec_deque::VecDeque;
        use sp_std::prelude::*;
        use sp_std::iter::{Peekable, Iterator};
        use super::{Config, ProposalBatch, ProposalBatchItemOf, BatchItemKind};
        
        /// Visited tree node abstraction
        pub struct BatchTreeNode<Data> {
            /// Nested level
            pub depth: usize,
            pub data: Data,
        }
        
        /// Marker-type used for traverse operation flow control 
        pub struct StopTraverse;
        
        /// Batch tree traverse operation.
        /// Invokes `visit_node` callback on each node and accepts flow-control commands from it
        pub fn traverse_batch_tree<'a, T: Config, V>(
            root: &'a ProposalBatch<T>,
            mut visit_node: V
        )
            where V: FnMut(BatchTreeNode<&'a ProposalBatchItemOf<T>>) -> Option<StopTraverse>
        {
            let mut stack = VecDeque::<Peekable<Box<dyn Iterator<Item=&ProposalBatchItemOf<T>>>>>::new();
            let boxed: Box<dyn Iterator<Item=&ProposalBatchItemOf<T>>> = Box::new(root.iter());
            stack.push_front(boxed.peekable());
            while !stack.is_empty() {
                let depth = stack.len();
                while let Some(data) = stack.front_mut().unwrap().next() {
                    if visit_node(BatchTreeNode { depth, data }).is_some() {
                        return
                    }
                    match BatchItemKind::<T>::kind(data) {
                        BatchItemKind::Propose(batch) => {
                            let boxed: Box<dyn Iterator<Item=&ProposalBatchItemOf<T>>> = Box::new(batch.iter());
                            stack.push_front(boxed.peekable());
                            break
                        },
                        _ => ()
                    }
                }
                if stack.front_mut().unwrap().peek().is_none() {
                    stack.pop_front();
                }
            }
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
            
            let proposal = DeipProposal::<T>::create(
                batch,
                author,
                <DeipProposal<T>>::timepoint
            )?;

            StorageOpsTransaction::<T, _>::new()
                .commit(move |ops| {
                    let author = proposal.author.clone();
                    let batch = proposal.batch.clone();
                    let proposal_id = proposal.id;
                    ops.push_op(StorageOps::CreateProposal(proposal));
                    ops.push_op(StorageOps::DepositEvent(Event::<T>::Proposed {
                        author,
                        batch,
                        proposal_id
                    }));
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
        /// Execute batch as atomic transaction
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
    
    /// Specialized version of [`BatchItem`]
    #[allow(type_alias_bounds)]
    pub type ProposalBatchItemOf<T: Config> = BatchItem<
        <T as frame_system::Config>::AccountId,
        <T as Config>::Call
    >;
    
    #[allow(type_alias_bounds)]
    pub type ProposalBatch<T: Config> = Vec<ProposalBatchItemOf<T>>;
    
    /// Proposal object
    #[derive(Debug, Encode, Decode, Clone, Eq, PartialEq)]
    pub struct DeipProposal<T: Config> {
        /// Proposal ID
        id: ProposalId,
        /// Batch-transaction items
        batch: ProposalBatch<T>,
        /// Member decisions mapping
        decisions: BTreeMap<T::AccountId, ProposalMemberDecision>,
        /// Proposal state
        state: ProposalState,
        /// Proposal author
        author: T::AccountId
    }
    
    /// Proposal state
    #[derive(Debug, Clone, Copy, Eq, PartialEq, Encode, Decode)]
    pub enum ProposalState {
        /// Pending proposal
        Pending,
        /// Rejected proposal
        Rejected,
        /// Batch transaction executed successfully
        Done,
        /// Batch transaction execution failed
        Failed(DispatchError)
    }
    
    use storage_ops::*;
    #[doc(no_inline)]
    /// Module contains abstractions over pallet storage operations
    pub mod storage_ops {
        use sp_std::collections::vec_deque::VecDeque;
        use sp_std::marker::PhantomData;
        use sp_std::prelude::*;
        use super::{
            Config, DeipProposal, Event, ProposalId,
            ProposalStorage, Pallet, PendingProposals};
        
        /// Storage operations
        pub enum StorageOps<T: Config> {
            /// Deposit event
            DepositEvent(Event<T>),
            /// Create proposal
            CreateProposal(DeipProposal<T>),
            /// Update proposal
            UpdateProposal(DeipProposal<T>),
            /// Delete proposal
            DeleteProposal(DeipProposal<T>),
        }
        
        /// Fifo-queue for storage operations
        pub struct StorageOpsQueue<T>(VecDeque<T>);
        impl<T> StorageOpsQueue<T> {
            /// Add storage operation
            pub fn push_op(&mut self, op: T) -> &mut Self { self.0.push_back(op); self }
            fn pop_op(&mut self) -> Option<T> { self.0.pop_front() }
        }

        /// Multi-ops storage transaction 
        pub struct StorageOpsTransaction<T: Config, Ops=StorageOps<T>>(StorageOpsQueue<Ops>, PhantomData<T>);
        impl<T: Config> StorageOpsTransaction<T, StorageOps<T>> {
            /// New storage transaction
            pub fn new() -> Self { Self(StorageOpsQueue(VecDeque::new()), Default::default()) }
            /// Execute callable then perform storage operations provided via ops-queue
            pub fn commit<R>(mut self, transactional: impl FnOnce(&mut StorageOpsQueue<StorageOps<T>>) -> R) -> R {
                let result = transactional(&mut self.0);
                while let Some(op) = self.0.pop_op() {
                    match op {
                        StorageOps::DepositEvent(event) => {
                            <Pallet<T>>::deposit_event(event);
                        },
                        StorageOps::CreateProposal(proposal) => {
                            let members = proposal.decisions.keys().cloned();
                            for m in members {
                                PendingProposals::<T>::mutate(m, |x| {
                                    x.insert(proposal.id, proposal.author.clone());
                                });
                            }
                            <ProposalStorage<T>>::insert(proposal.author.clone(), proposal.id, proposal);
                        },
                        StorageOps::UpdateProposal(proposal) => {
                            <ProposalStorage<T>>::insert(proposal.author.clone(), proposal.id, proposal)
                        },
                        StorageOps::DeleteProposal(proposal) => {
                            let DeipProposal::<T> {
                                id: proposal_id,
                                decisions,
                                author,
                                .. 
                            } = proposal;
                            let members = decisions.keys();
                            for m in members {
                                PendingProposals::<T>::mutate(m, |x| {
                                    x.remove(&proposal_id);
                                });
                            }
                            <ProposalStorage<T>>::remove(author, proposal_id);
                        },
                    }
                }
                result
            }
        }
    }
    
    impl<T: Config> DeipProposal<T> {
        /// Generate "Timepoint" aka unique proposal ID.
        /// Implemented as hash-value of Timepoint from `pallet_multisig`   
        fn timepoint() -> ProposalId { pallet_multisig::Module::<T>::timepoint().twox_256() }
        
        /// Create proposal object.
        /// Fail if input arguments violates proposal assertions (See [proposal_assertions](./module.proposal_assertions))
        fn create(
            batch: Vec<ProposalBatchItemOf<T>>,
            author: T::AccountId,
            timepoint: impl FnOnce() -> ProposalId
        )
            -> Result<Self, Error<T>>
        {
            let decisions = BTreeMap::from_iter(
                batch.iter().map(|x| (
                    x.account.clone(),
                    ProposalMemberDecision::Pending
                ))
            );
            let proposal = Self {
                id: timepoint(),
                batch,
                decisions,
                state: ProposalState::Pending,
                author
            };
            match assert_proposal(&proposal, 2) {
                Some(ProposalAssertions::DepthLimit) => {
                    Err(Error::<T>::ReachDepthLimit)
                },
                Some(ProposalAssertions::SelfReference) => {
                    Err(Error::<T>::SelfReferential)
                },
                None => Ok(proposal),
            }
        }
        
        /// 
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
            let member_decision = self.decisions.get_mut(member).ok_or(Error::<T>::NotAMember)?;
            
            ensure!(matches!(self.state, ProposalState::Pending), Error::<T>::AlreadyResolved);

            match member_decision.decide(decision) {
                Err(_) => return Err(Error::<T>::AlreadyResolved),
                Ok(None) => Ok(None),
                Ok(Some(ProposalMemberDecision::Pending)) => {
                    storage_ops.push_op(StorageOps::DepositEvent(Event::<T>::RevokedApproval {
                        member: member.clone(),
                        proposal_id: self.id
                    }));
                    storage_ops.push_op(StorageOps::UpdateProposal(self));
                    Ok(None)
                },
                Ok(Some(ProposalMemberDecision::Reject)) => {
                    self.state = ProposalState::Rejected;
                    storage_ops.push_op(StorageOps::DepositEvent(Event::<T>::Resolved {
                        member: member.clone(),
                        proposal_id: self.id
                    }));
                    storage_ops.push_op(StorageOps::DeleteProposal(self));
                    Ok(None)
                },
                Ok(Some(ProposalMemberDecision::Approve)) => {
                    if self.ready_to_exec() {
                        let batch_exec_result = batch_exec(self.batch.clone());
                        self.state = if let Err(ref err) = batch_exec_result { 
                            ProposalState::Failed(err.error.clone())
                        } else {
                            ProposalState::Done
                        };
                        storage_ops.push_op(StorageOps::DepositEvent(Event::<T>::Resolved {
                            member: member.clone(),
                            proposal_id: self.id,
                        }));
                        storage_ops.push_op(StorageOps::DeleteProposal(self));
                        Ok(Some(batch_exec_result))
                    } else {
                        storage_ops.push_op(StorageOps::DepositEvent(Event::<T>::Approved {
                            member: member.clone(),
                            proposal_id: self.id,
                        }));
                        storage_ops.push_op(StorageOps::UpdateProposal(self));
                        Ok(None)
                    }
                },
            }
        }
        
        fn ready_to_exec(&self) -> bool {
            let approved = self.decisions.values()
                .all(|x: &ProposalMemberDecision| {
                    matches!(x, ProposalMemberDecision::Approve)
                });
            approved && matches!(self.state, ProposalState::Pending)
        }
    }
    
    /// Batch item generic container
    #[derive(Debug, Clone, Eq, PartialEq, Encode, Decode)]
    pub struct BatchItem<Account, CallT> {
        account: Account,
        call: CallT,
    }
    
    /// Proposal member decision
    #[derive(Debug, Clone, Copy, Eq, PartialEq, Encode, Decode)]
    pub enum ProposalMemberDecision {
        /// Pending state
        Pending,
        /// Approved state
        Approve,
        /// Rejected state
        Reject
    }
    impl ProposalMemberDecision {
        /// Make decision state transition.
        /// 
        /// Except of transitions from `Reject` current state all another transitions are allowed.
        /// `Ok(None)` result means transition to the same state.
        /// 
        /// This function must stay private to disallow state transitions from code outsides
        /// of this module.
        /// You should prefer to use [`DeipProposal`] object as a pallet logic's main interface
        /// 
        fn decide(&mut self, decision: Self) -> Result<Option<Self>, Self> {
            let cur = self;
            let new = &decision;
            match (&cur, new) {
                (Self::Reject, _) => Err(*cur),
                _ => {
                    let transition = cur != new;
                    *cur = *new;
                    if transition { Ok(Some(*cur)) } else { Ok(None) }
                },
            }
        }
    }
}
