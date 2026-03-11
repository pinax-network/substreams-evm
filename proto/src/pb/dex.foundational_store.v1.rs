// @generated
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Pool {
    #[prost(bytes = "vec", repeated, tag = "1")]
    pub tokens: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
    #[prost(bytes = "vec", tag = "2")]
    pub factory: ::prost::alloc::vec::Vec<u8>,
}
impl ::prost::Name for Pool {
    const NAME: &'static str = "Pool";
    const PACKAGE: &'static str = "dex.foundational_store.v1";

    fn type_url() -> ::prost::alloc::string::String {
        "type.googleapis.com/dex.foundational_store.v1.Pool".into()
    }
}
