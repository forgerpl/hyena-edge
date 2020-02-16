pub(super) mod dense;
pub(super) mod sparse;

pub(crate) use self::dense::DenseNumericBlock;
pub use self::sparse::SparseIndex;
pub(crate) use self::sparse::SparseIndexedNumericBlock;
