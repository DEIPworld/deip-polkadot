use jsonrpc_core::{Error as RpcError, ErrorCode as RpcErrorCode, Result as RpcResult};
use jsonrpc_derive::rpc;

use std::vec::Vec;
use std::sync::Arc;

use codec::Codec;

use sp_runtime::{generic::BlockId, traits::Block as BlockT};

use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;

use pallet_deip_org::org::{OrgName};
use pallet_deip_org::api::{GetResult, GetMultiResult, ListResult};
pub use pallet_deip_org::api::{DeipOrgRuntimeApi};


#[rpc]
pub trait DeipOrgRpcApi<BlockHash, AccountId> {
    #[rpc(name = "deipOrg_get")]
    fn get(&self, at: Option<BlockHash>, name: OrgName) -> RpcResult<GetResult<AccountId>>;
    
    #[rpc(name = "deipOrg_getMulti")]
    fn get_multi(&self, at: Option<BlockHash>, names: Vec<OrgName>) -> RpcResult<GetMultiResult<AccountId>>;
    
    #[rpc(name = "deipOrg_list")]
    fn list(&self, at: Option<BlockHash>) -> RpcResult<ListResult<AccountId>>;
}


/// A struct that implements the `DeipOrgRpcApiObj`.
pub struct DeipOrgRpcApiObj<C, Block> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<Block>,
}

impl<C, Block> DeipOrgRpcApiObj<C, Block> {
	/// Create new `DeipOrgRpcApiObj` instance with the given reference to the client.
	pub fn new(client: Arc<C>) -> Self {
		Self {
			client,
			_marker: Default::default(),
		}
	}
}

impl<C, Block, AccountId> DeipOrgRpcApi<<Block as BlockT>::Hash, AccountId>
    for DeipOrgRpcApiObj<C, Block>
        where
            Block: BlockT,
            C: Send + Sync + 'static,
            C: ProvideRuntimeApi<Block>,
            C: HeaderBackend<Block>,
            C::Api: DeipOrgRuntimeApi<Block, AccountId>,
            AccountId: Codec,
{
	fn get(&self, at: Option<<Block as BlockT>::Hash>, name: OrgName) -> RpcResult<GetResult<AccountId>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let runtime_api_result = api.get(&at, name);
		
		runtime_api_result.map_err(|e| RpcError {
			code: RpcErrorCode::ServerError(9876), // No real reason for this value
			message: "Something wrong".into(),
			data: Some(format!("{:?}", e).into()),
		})
	}
	
	fn get_multi(&self, at: Option<<Block as BlockT>::Hash>, names: Vec<OrgName>) -> RpcResult<GetMultiResult<AccountId>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let runtime_api_result = api.get_multi(&at, names);
		
		runtime_api_result.map_err(|e| RpcError {
			code: RpcErrorCode::ServerError(9876), // No real reason for this value
			message: "Something wrong".into(),
			data: Some(format!("{:?}", e).into()),
		})
	}
	
	fn list(&self, at: Option<<Block as BlockT>::Hash>) -> RpcResult<ListResult<AccountId>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let runtime_api_result = api.list(&at);
		
		runtime_api_result.map_err(|e| RpcError {
			code: RpcErrorCode::ServerError(9876), // No real reason for this value
			message: "Something wrong".into(),
			data: Some(format!("{:?}", e).into()),
		})
	}
}
