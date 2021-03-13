// MIT/Apache2 License

#[cfg(feature = "async")]
use std::{future::Future, pin::Pin};

#[cfg(feature = "async")]
pub type GenericResult<'future, T = ()> =
    Pin<Box<dyn Future<Output = crate::Result<T>> + Send + 'future>>;
