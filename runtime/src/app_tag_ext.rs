#![allow(unused_imports)]
use codec::{Encode, Decode};
use frame_system::Config;
use frame_support::weights::DispatchInfo;
use sp_runtime::{
    traits::{SignedExtension, DispatchInfoOf, Dispatchable},
    transaction_validity::TransactionValidityError,
};
use sp_std::marker::PhantomData;
use sp_core::H160;

pub type AppTag = H160;

/// Application tag (tenant) transaction metadata.
///
/// No any validity checks.
#[derive(Encode, Decode, Clone, Eq, PartialEq)]
pub struct TagApp<T: Config>(AppTag, PhantomData<T>);

impl<T: Config> TagApp<T> {
    // utility constructor. Used only in client/factory code.
    pub fn from(tag: AppTag) -> Self {
        Self(tag, Default::default())
    }
}

impl<T: Config> sp_std::fmt::Debug for TagApp<T> {
    #[cfg(feature = "std")]
    fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
        write!(f, "TagApp({:?})", self.0)
    }

    #[cfg(not(feature = "std"))]
    fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
        Ok(())
    }
}

impl<T: Config  + Send + Sync> SignedExtension for TagApp<T> where
    T::Call: Dispatchable<Info=DispatchInfo> + Send + Sync
{
    type AccountId = T::AccountId;
    type Call = T::Call;
    type AdditionalSigned = AppTag;
    type Pre = ();
    const IDENTIFIER: &'static str = "TagApp";

    fn additional_signed(&self) -> sp_std::result::Result<Self::AdditionalSigned, TransactionValidityError>
    {
        Ok(self.0.clone())
    }

    fn pre_dispatch(
        self,
        _who: &Self::AccountId,
        _call: &Self::Call,
        _info: &DispatchInfoOf<Self::Call>,
        _len: usize,
    ) -> Result<(), TransactionValidityError> {
        Ok(())
    }
}
