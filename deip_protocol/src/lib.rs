// #[cfg(test)]
// mod tests {
//     #[test]
//     fn it_works() {
//         assert_eq!(2 + 2, 4);
//     }
//     
//     fn protocol() {
//         
//     }
// }
// 
// pub struct ProtocolObj {
//     
// }
// impl Protocol for ProtocolObj {
//     type AccountId = ();
//     type Account = ();
// }
// 
// pub struct AccountObj<P: Protocol> where Self: Account<P> {
//     id: P::AccountId
// }
// impl Account<ProtocolObj> for AccountObj<ProtocolObj> {
//     fn id(&self) -> <ProtocolObj as Protocol>::AccountId {
//         self.id
//     }
// }
// 
// pub use traits::*;
// mod traits {
//     pub trait Protocol {
//         type AccountId;
//         type Account: Account<Self>;
//     }
// 
//     pub trait Account<P: Protocol> {
//         fn id(&self) -> P::AccountId;
//     }
// }
