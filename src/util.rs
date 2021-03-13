// MIT/Apache2 License

use num_traits::{AsPrimitive, Bounded};
use std::ops;
#[cfg(feature = "async")]
use std::{future::Future, pin::Pin};

#[cfg(feature = "async")]
pub type GenericResult<'future, T = ()> =
    Pin<Box<dyn Future<Output = crate::Result<T>> + Send + 'future>>;

pub(crate) fn clamp<N: Bounded + Copy + ops::Sub + 'static>(i: f32) -> N
where
    f32: AsPrimitive<N> + From<N::Output>,
{
    let bounds: f32 = (N::max_value() - N::min_value()).into();
    (bounds * i).as_()
}
