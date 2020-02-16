#[macro_use]
pub(super) mod block;

#[macro_use]
pub mod fragment;
pub(crate) mod index;
pub(crate) mod sparse_iter;

pub(crate) use self::block::{BlockHeadMap, BlockHeads, BlockMap, BlockStorageMap};
pub use self::block::{BlockStorage, BlockStorageType};
pub use self::fragment::{Fragment, FragmentRef, TimestampFragment};
pub(crate) use self::index::ColumnIndexMap;
pub use self::index::{ColumnIndexStorage, ColumnIndexStorageMap};
pub(crate) use self::sparse_iter::{SparseIter, SparseIterator};

pub type ColumnId = usize;
pub type RowId = usize;
pub use crate::params::SourceId;
