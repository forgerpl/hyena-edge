use super::{Catalog, Column, PartitionGroup, PartitionMeta};
use crate::block::SparseIndex;
use crate::error::*;
use crate::mutator::append::Append;
use crate::params::{SourceId, CATALOG_METADATA, PARTITION_GROUP_METADATA};
use crate::scanner::{Scan, ScanResult};
use crate::storage::manager::{PartitionGroupManager, PartitionManager};
use crate::ty::BlockStorage;
use hyena_common::collections::HashMap;
use hyena_common::ty::Timestamp;
use std::default::Default;
use std::sync::RwLock;

use crate::params::BLOCK_SIZE;
use crate::storage::manager::RootManager;
use hyena_test::random::timestamp::RandomTimestampGen;
use static_assertions::assert_eq_size;

// until const fn stabilizes we have to use this hack
// see https://github.com/rust-lang/rust/issues/24111

// make sure that size_of::<Timestamp>() == 8
assert_eq_size!(timestamp_size_check; u64, Timestamp);

const TIMESTAMP_SIZE: usize = ::std::mem::size_of::<Timestamp>();
const MAX_RECORDS: usize = BLOCK_SIZE / TIMESTAMP_SIZE;

#[macro_use]
mod append;
mod scan;

pub(super) fn create_random_partitions(pg: &mut PartitionGroup, im_count: usize, mut_count: usize) {
    let pts = RandomTimestampGen::pairs::<u64>(im_count + mut_count);

    let (imparts, mutparts): (Vec<_>, Vec<_>) = pts
        .iter()
        .map(|&(ref lo, ref hi)| {
            let mut part = pg
                .create_partition(*lo)
                .with_context(|_| "Unable to create partition")
                .unwrap();

            part.set_ts(None, Some(*hi))
                .with_context(|_| "Failed to set timestamp on partition")
                .unwrap();

            part
        })
        .enumerate()
        .partition(|&(idx, _)| idx < im_count);

    pg.immutable_partitions = imparts
        .into_iter()
        .map(|(_, part)| (PartitionMeta::from(&part), part))
        .collect();
    pg.mutable_partitions = locked!(rw mutparts.into_iter().map(|(_, part)| part).collect());
}
