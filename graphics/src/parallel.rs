#[cfg(feature = "rayon")]
pub use rayon::prelude::*;

#[cfg(not(feature = "rayon"))]
pub trait OptionalParIter: IntoIterator {
    fn into_par_iter(self) -> Self::IntoIter
    where
        Self: Sized,
    {
        self.into_iter()
    }

    fn par_bridge(self) -> Self
    where
        Self: Sized,
    {
        self
    }
}

#[cfg(not(feature = "rayon"))]
impl<I: IntoIterator> OptionalParIter for I {}
