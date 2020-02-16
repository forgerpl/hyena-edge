#![cfg_attr(feature = "nightly", feature(test))]

#[cfg(all(feature = "nightly", test))]
extern crate test;

#[macro_use]
extern crate log;
#[macro_use]
extern crate cfg_if;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate hyena_common;

#[cfg(test)]
#[macro_use]
extern crate hyena_test;
#[macro_use]
extern crate failure;

#[macro_use]
extern crate strum_macros;

#[cfg(feature = "debug")]
extern crate prettytable;
#[cfg(feature = "debug")]
extern crate term;

#[cfg(feature = "debug")]
pub mod debug;

pub(crate) mod params;

mod error;

mod fs;
mod storage;
#[macro_use]
mod block;

#[macro_use]
mod ty;
pub mod datastore;
mod mutator;
mod scanner;

pub use self::block::{BlockType, ColumnIndexType, SparseIndex};
pub use self::datastore::{Catalog, Column, ColumnMap};
pub use self::error::{Error, Result};
pub use self::params::SourceId;
pub use self::scanner::{
    Regex, Scan, ScanData, ScanFilter, ScanFilterApply, ScanFilterOp, ScanFilters, ScanResult,
    ScanTsRange, StreamConfig, StreamState,
};
pub use self::ty::block::memory::Block as MemoryBlock;
pub use self::ty::block::mmap::Block as MemmapBlock;
pub use self::ty::fragment::{Fragment, FragmentIter, TimestampFragment};
pub use self::ty::{BlockStorage, ColumnId, ColumnIndexStorage, RowId};
pub use hyena_common::ty::Value;
pub use hyena_common::ty::{Timestamp, MAX_TIMESTAMP_VALUE, MIN_TIMESTAMP_VALUE};

pub use self::mutator::{Append, BlockData};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
