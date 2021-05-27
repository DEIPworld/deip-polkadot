### DEIP protocol operations

------

###### CREATE_PROPOSAL

The `CREATE_PROPOSAL` DEIP protocol operation is Implemented as `propose(batch)` extrinsic from the `deipProposal` pallet (runtime module).  

In the [Polkadot JS App](https://polkadot.js.org/apps) click on "[Developer -> Extrinsics](https://polkadot.js.org/apps/#/extrinsics)" at menu bar and select the target extrinsic as a Pallet->Call pair:

| deipProposal | propose(batch) |
| :----------- | :------------- |

Then click the "+ Add item" button to add the proposal *batch* *items*.

> Total number of batch items is NOT LIMITED yet but we are going to perform some kinds of benchmarking to extract the optimal batch size constraints soon.

> Proposal may have other CREATE_PROPOSAL operations as a batch items thus we got some constraints on the  ***nested proposals***:
>
> - Now the *depth* of nested proposals is set to **max 2**
> - We must to check that proposal batch has no UPDATE_PROPOPSAL operations that refers to the parent proposal via `proposal_id` call arg.
>   Because of proposal ID is a hash of [BlockNumber;ExtrinsicId] pair (where the ExtrinsicId is an ID of the currently executed CREATE_PROPOSAL operation on a Block) then it may be predicted in some cases (for example: if we have no transactions on the network in the current time then we can predict the next BlockNumber and suggest that ExtrinsicID will be "1", then we can potentially create a self-referential proposal).



###### UPDATE_PROPOSAL

The `UPDATE_PROPOSAL` DEIP protocol operation is implemented as `decide(proposal_id, decision)` extrinsic from the `deipProposal` pallet.  

In the [Polkadot JS App](https://polkadot.js.org/apps) click on "[Developer -> Extrinsics](https://polkadot.js.org/apps/#/extrinsics)" at menu bar and select the target extrinsic as a Pallet->Call pair:

| deipProposal | decide(proposal_id, decision) |
| ------------ | ----------------------------- |

> To obtain a `proposal_id` of the *pending* *proposal* you should perform some Storage API queries (see "Storage API" section).
> Also `CREATE_PROPOSAL` emits a **Proposed(AccountId, ProposalId)** event where `AccountId` is a proposal author account ID.

Fill up fields and submit transaction. If you make "Approve" decision then state of a proposal member decision updates from  "Pending" to "Approved" state in the proposal object.  When the all members of proposal make "Approve" decision the batch will be executed as a single transaction and proposal state will updates from "Pending" to "Done" in the case of the successful batch execution or "Fail" in the case of batch execution error. If only one member make "Decline" decision then proposal state will be immediately updated from "Pending" to "Rejected" state.



###### DELETE_PROPOSAL

*Not implemented*



### Storage API

------

###### PENDING_PROPOSALS

The `PENDING_PROPOSALS` storage query is implemented as `pendingProposals(AccountId)` query from the `deipProposal` pallet.

In the [Polkadot JS App](https://polkadot.js.org/apps) click on "[Developer -> Chain state -> Storage](https://polkadot.js.org/apps/#/chainstate)" at menu bar and select the target storage query as  Pallet->Query pair:

| deipProposal | pendingProposals(AccountId): PendingProposalsMap |
| ------------ | ------------------------------------------------ |

The `pendingProposals` query accept an AccountId and returns a hash-map where keys of hash-map is an IDs of *pending proposals* with corresponding values which are *proposal* *author* AccountId.

```json
{
  "0x2e3e498716c2ad1e9544fd77e4e98aa41513e9dda58494af1b8160d11c5bb704":"5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
}
```



###### PROPOSAL_STORAGE

In the [Polkadot JS App](https://polkadot.js.org/apps) click on "[Developer -> Chain state -> Storage](https://polkadot.js.org/apps/#/chainstate)" at menu bar.

Select the `proposalStorage` storage query from the `deipProposal` pallet to explore the proposal *state*:

| deipProposal | proposalStorage(AccountId, ProposalId): Option\<DeipProposal\> |
| ------------ | ------------------------------------------------------------ |

The `proposalStorage` accept an AccountId which is a proposal author's account ID and ProposalId as proposal ID and returns a proposal object:

```json
{
  "id": "0x2e3e498716c2ad1e9544fd77e4e98aa41513e9dda58494af1b8160d11c5bb704",
  "batch": [
    {
      "account": "5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y",
      "call": {
        "args": [
          "somedomainsomedomain"
        ],
        "method": "addDomain",
        "section": "deip"
      }
    }
  ],
  "decisions": {
    "5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y": "Pending"
  },
  "state": "Pending",
  "author": "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"
}
```

