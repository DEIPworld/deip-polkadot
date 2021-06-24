use sp_std::prelude::*;

use codec::Codec;

use super::org::{OrgName, Org};


pub type GetResult<AccountId> = Option<Org<AccountId, OrgName>>;
pub type GetMultiResult<AccountId> = Vec<Option<Org<AccountId, OrgName>>>;
pub type ListResult<AccountId> = Vec<Org<AccountId, OrgName>>;

sp_api::decl_runtime_apis! {
    pub trait DeipOrgRuntimeApi<AccountId>
        where AccountId: Codec
    {
        fn get(name: OrgName) -> GetResult<AccountId>;
        fn get_multi(names: Vec<OrgName>) -> GetMultiResult<AccountId>;
        fn list() -> ListResult<AccountId>;
    }
}

use super::{Pallet, Config, OrgRepository};

impl<T: Config> Pallet<T> {
    pub fn rpc_get(name: OrgName) -> GetResult<T::AccountId> {
        OrgRepository::<T>::try_get(name).ok()
    }
    pub fn rpc_get_multi(names: Vec<OrgName>) -> GetMultiResult<T::AccountId> {
        names.into_iter().map(|x| OrgRepository::<T>::try_get(x).ok()).collect()
    }
    pub fn rpc_list() -> ListResult<T::AccountId> {
        OrgRepository::<T>::iter_values().collect()
    }
}
