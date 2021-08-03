// MIT/Apache2 License

use num_traits::{AsPrimitive, Bounded};
use std::{fmt, ops};
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

/// Hides a type in order to make #[derive(Debug)] usable.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub(crate) struct DebugContainer<T>(pub T);

impl<T> DebugContainer<T> {
    #[inline]
    pub(crate) fn new(t: T) -> Self {
        Self(t)
    }

    #[inline]
    pub(crate) fn into_inner(self) -> T {
        self.0
    }
}

impl<T> ops::Deref for DebugContainer<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> ops::DerefMut for DebugContainer<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T> fmt::Debug for DebugContainer<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("..")
    }
}

impl<T> From<T> for DebugContainer<T> {
    #[inline]
    fn from(t: T) -> Self {
        Self(t)
    }
}
