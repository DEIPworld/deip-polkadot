/// Module contains abstractions over pallet storage operations

use sp_std::prelude::*;

pub use pallet_deip_toolkit::storage_ops::*;

use crate::proposal::DeipProposal;

use super::{Config, Event, ProposalStorage, Pallet, PendingProposals};


pub type StorageWrite<T> = StorageOpsTransaction<StorageOps<T>>;
pub type StorageOpsT<T> = StorageOpsQueue<StorageOps<T>>;

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
impl<T: Config> StorageOp for StorageOps<T> {
    fn exec(self) {
        match self {
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
}
