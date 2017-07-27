use error::*;
use uuid::Uuid;

use block::BlockData;
use ty::{Block, BlockMap, BlockType, BlockTypeMap, Timestamp};
use std::path::{Path, PathBuf};
use std::cmp::{max, min};
use ty::block::memory::BlockType as MemoryBlockType;
#[cfg(feature = "mmap")]
use ty::block::mmap::BlockType as MmapBlockType;
use params::PARTITION_METADATA;


pub(crate) type PartitionId = Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Partition<'part> {
    id: PartitionId,
    // TODO: do we really need ts_* here?
    ts_min: Timestamp,
    ts_max: Timestamp,

    #[serde(skip)]
    blocks: BlockMap<'part>,
    #[serde(skip)]
    data_root: PathBuf,
}

impl<'part> Partition<'part> {
    pub fn new<P: AsRef<Path>, TS: Into<Timestamp>>(root: P, ts: TS) -> Result<Partition<'part>> {
        let root = root.as_ref().to_path_buf();

        let meta = root.join(PARTITION_METADATA);

        if meta.exists() {
            bail!("Partition metadata already exists {:?}", meta);
        }

        let ts = ts.into();

        Ok(Partition {
            id: Partition::gen_id(),
            ts_min: ts,
            ts_max: ts,
            blocks: Default::default(),
            data_root: root,
        })
    }

    pub fn with_data<P: AsRef<Path>>(root: P) -> Result<Partition<'part>> {
        let root = root.as_ref().to_path_buf();

        let meta = root.join(PARTITION_METADATA);

        Partition::deserialize(&meta, &root)
    }

    pub(crate) fn scan_ts(&self) -> Result<(Timestamp, Timestamp)> {
        let ts_block = self.blocks
            .get(&0)
            .ok_or_else(|| "Failed to get timestamp block")?;

        block_apply!(expect physical U64Dense, ts_block, block, pb, {
            let (ts_min, ts_max) = pb.as_slice().iter().fold((None, None), |mut acc, &x| {
                acc.0 = Some(if let Some(cur) = acc.0 {
                    min(cur, x)
                } else {
                    x
                });

                acc.1 = Some(if let Some(cur) = acc.1 {
                    max(cur, x)
                } else {
                    x
                });

                acc
            });

            Ok((ts_min.ok_or_else(|| "Failed to get min timestamp")?.into(),
                ts_max.ok_or_else(|| "Failed to get max timestamp")?.into()))
        })
    }

    pub(crate) fn get_ts(&self) -> (Timestamp, Timestamp) {
        (self.ts_min, self.ts_max)
    }

    #[must_use]
    pub(crate) fn set_ts<TS>(&mut self, ts_min: Option<TS>, ts_max: Option<TS>) -> Result<()>
    where
        Timestamp: From<TS>,
    {

        let cmin = ts_min.map(Timestamp::from).unwrap_or(self.ts_min);
        let cmax = ts_max.map(Timestamp::from).unwrap_or(self.ts_max);

        if !(cmin <= cmax) {
            bail!("ts_min ({:?}) should be <= ts_max ({:?})", cmin, cmax)
        } else {
            self.ts_min = cmin;
            self.ts_max = cmax;

            Ok(())
        }
    }

    pub(crate) fn update_meta(&mut self) -> Result<()> {
        // TODO: handle ts_min changes -> partition path

        let (ts_min, ts_max) = self.scan_ts()
            .chain_err(|| "Unable to perform ts scan for metadata update")?;

        if ts_min != self.ts_min {
            warn!(
                "Partition min ts changed {} -> {} [{}]",
                self.ts_min,
                ts_min,
                self.id
            );
        }

        self.ts_min = ts_min;
        self.ts_max = ts_max;

        Ok(())
    }

    pub fn get_blocks(&self) -> &BlockMap<'part> {
        &self.blocks
    }

    pub fn mut_blocks(&mut self) -> &mut BlockMap<'part> {
        &mut self.blocks
    }

    pub fn ensure_blocks(&mut self, type_map: &BlockTypeMap) -> Result<()> {
        let blocks = Partition::prepare_blocks(&self.data_root, type_map)
            .chain_err(|| "Unable to create block map")?;

        self.blocks.extend(blocks);

        Ok(())
    }

    #[inline]
    pub fn gen_id() -> PartitionId {
        Uuid::new_v4()
    }

    fn prepare_blocks<P: AsRef<Path>>(root: P, type_map: &BlockTypeMap) -> Result<BlockMap<'part>> {
        type_map
            .iter()
            .map(|(block_id, block_type)| match *block_type {
                BlockType::Memory(bty) => {
                    use ty::block::memory::Block;

                    Block::create(bty)
                        .map(|block| (*block_id, block.into()))
                        .chain_err(|| "Unable to create in-memory block")
                }
                BlockType::Memmap(bty) => {
                    use ty::block::mmap::Block;

                    Block::create(&root, bty, *block_id)
                        .map(|block| (*block_id, block.into()))
                        .chain_err(|| "Unable to create mmap block")
                }
            })
            .collect()
    }

    fn flush(&self) -> Result<()> {
        // TODO: add dirty flag
        let meta = self.data_root.join(PARTITION_METADATA);

        Partition::serialize(self, &meta)
    }

    fn serialize<P: AsRef<Path>>(partition: &Partition<'part>, meta: P) -> Result<()> {
        let meta = meta.as_ref();

        let blocks: BlockTypeMap = (&partition.blocks).into();

        let data = (partition, blocks);

        serialize!(file meta, &data).chain_err(|| "Failed to serialize partition metadata")
    }

    fn deserialize<P: AsRef<Path>, R: AsRef<Path>>(meta: P, root: R) -> Result<Partition<'part>> {
        let meta = meta.as_ref();

        if !meta.exists() {
            bail!("Cannot find partition metadata {:?}", meta);
        }

        let (mut partition, blocks): (Partition, BlockTypeMap) = deserialize!(file meta)
            .chain_err(|| "Failed to read partition metadata")?;

        partition.blocks = Partition::prepare_blocks(&root, &blocks)
            .chain_err(|| "Failed to read block data")?;

        partition.data_root = root.as_ref().to_path_buf();

        Ok(partition)
    }
}


impl<'part> Drop for Partition<'part> {
    fn drop(&mut self) {
        self.flush()
            .chain_err(|| "Failed to flush data during drop")
            .unwrap();
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::HashMap;
    use ty::BlockId;
    use std::fmt::Debug;
    use helpers::random::timestamp::{RandomTimestamp, RandomTimestampGen};
    use block::BlockData;


    macro_rules! block_test_impl {
        ($base: ident, $($variant: ident),* $(,)*) => {{
            let mut idx = 0;

            hashmap! {
                $(
                    {idx += 1; idx} => $base::$variant,
                )*
            }
        }};

        ($base: ident) => {
            map_block_type_variants!(block_test_impl, $base)
        };
    }

    fn ts_init<'part, P, T>(
        root: P,
        ts_ty: T,
        count: usize,
    ) -> (Partition<'part>, Timestamp, Timestamp)
    where
        T: Debug + Clone,
        BlockType: From<T>,
        BlockTypeMap: From<HashMap<BlockId, T>>,
        P: AsRef<Path>,
    {
        let ts = RandomTimestampGen::iter().take(count).collect::<Vec<_>>();

        let ts_min = ts.iter()
            .min()
            .ok_or_else(|| "Failed to get minimal timestamp")
            .unwrap();

        let ts_max = ts.iter()
            .max()
            .ok_or_else(|| "Failed to get maximal timestamp")
            .unwrap();

        let mut part = Partition::new(&root, *ts_min)
            .chain_err(|| "Failed to create partition")
            .unwrap();

        part.ensure_blocks(&(hashmap! { 0 => ts_ty }).into())
            .chain_err(|| "Failed to create blocks")
            .unwrap();

        {
            let ts_block = part.mut_blocks()
                .get_mut(&0)
                .ok_or_else(|| "Failed to get ts block")
                .unwrap();

            block_apply!(mut expect physical U64Dense, ts_block, block, pb, {
                {
                    let buf = pb.as_mut_slice_append();
                    assert!(buf.len() >= ts.len());

                    &mut buf[..count].copy_from_slice(&ts[..]);

                    assert_eq!(&buf[..count], &ts[..]);
                }

                pb.set_written(count)
                    .chain_err(|| "Unable to move head ptr")
                    .unwrap();
            });
        }

        (part, (*ts_min).into(), (*ts_max).into())
    }

    fn scan_ts<T>(ts_ty: T)
    where
        T: Debug + Clone + Copy,
        BlockType: From<T>,
        BlockTypeMap: From<HashMap<BlockId, T>>,
    {

        let root = tempdir!();

        let (part, ts_min, ts_max) = ts_init(&root, ts_ty, 100);

        let (ret_ts_min, ret_ts_max) = part.scan_ts()
            .chain_err(|| "Failed to scan timestamps")
            .unwrap();

        assert_eq!(ret_ts_min, ts_min);
        assert_eq!(ret_ts_max, ts_max);
    }

    fn update_meta<T>(ts_ty: T)
    where
        T: Debug + Clone + Copy,
        BlockType: From<T>,
        BlockTypeMap: From<HashMap<BlockId, T>>,
    {

        let root = tempdir!();

        let (mut part, ts_min, ts_max) = ts_init(&root, ts_ty, 100);

        part.update_meta()
            .chain_err(|| "Unable to update metadata")
            .unwrap();

        assert_eq!(part.ts_min, ts_min);
        assert_eq!(part.ts_max, ts_max);
    }

    #[test]
    fn get_ts() {
        let root = tempdir!();

        let &(ts_min, ts_max) = &RandomTimestampGen::pairs(1)[..][0];

        let mut part = Partition::new(&root, ts_min)
            .chain_err(|| "Failed to create partition")
            .unwrap();

        part.ts_max = ts_max;

        let part = part;

        let (ret_ts_min, ret_ts_max) = part.get_ts();

        assert_eq!(ret_ts_min, ts_min);
        assert_eq!(ret_ts_max, ts_max);
    }

    #[test]
    fn set_ts() {
        let root = tempdir!();

        let &(ts_min, ts_max) = &RandomTimestampGen::pairs(1)[..][0];
        let start_ts = RandomTimestampGen::random_from(ts_max);

        assert_ne!(ts_min, start_ts);

        let mut part = Partition::new(&root, start_ts)
            .chain_err(|| "Failed to create partition")
            .unwrap();

        part.set_ts(Some(ts_min), Some(ts_max))
            .chain_err(|| "Unable to set timestamps")
            .unwrap();

        assert_eq!(part.ts_min, ts_min);
        assert_eq!(part.ts_max, ts_max);
    }

    mod serialize {
        use super::*;


        pub(super) fn block<'block, T>(type_map: HashMap<BlockId, T>)
        where
            T: Debug + Clone,
            Block<'block>: PartialEq<T>,
            BlockType: From<T>,
            BlockTypeMap: From<HashMap<BlockId, T>>,
        {
            let root = tempdir!();
            let ts = <Timestamp as Default>::default();

            let mut part = Partition::new(&root, *ts)
                .chain_err(|| "Failed to create partition")
                .unwrap();

            part.ensure_blocks(&(type_map.clone().into()))
                .chain_err(|| "Failed to create blocks")
                .unwrap();

            assert_eq!(part.ts_min, ts);
            assert_eq!(part.ts_max, ts);
            assert_eq!(part.blocks.len(), type_map.len());
            for (block_id, block_type) in &type_map {
                assert_eq!(part.blocks[block_id], *block_type);
            }
        }

        #[test]
        fn meta() {
            let root = tempdir!();
            let ts = <Timestamp as Default>::default();

            {
                Partition::new(&root, *ts)
                    .chain_err(|| "Failed to create partition")
                    .unwrap();
            }

            let meta = root.as_ref().join(PARTITION_METADATA);

            assert!(meta.exists());
        }
    }

    mod deserialize {
        use super::*;


        pub(super) fn block<'block, T>(type_map: HashMap<BlockId, T>)
        where
            T: Debug + Clone,
            Block<'block>: PartialEq<T>,
            BlockType: From<T>,
            BlockTypeMap: From<HashMap<BlockId, T>>,
        {
            let root = tempdir!();
            let ts = <Timestamp as Default>::default();

            {
                let mut part = Partition::new(&root, *ts)
                    .chain_err(|| "Failed to create partition")
                    .unwrap();

                part.ensure_blocks(&(type_map.clone().into()))
                    .chain_err(|| "Failed to create blocks")
                    .unwrap();
            }

            let part = Partition::with_data(&root)
                .chain_err(|| "Failed to read partition data")
                .unwrap();

            assert_eq!(part.blocks.len(), type_map.len());
            for (block_id, block_type) in &type_map {
                assert_eq!(part.blocks[block_id], *block_type);
            }
        }

        #[test]
        fn meta() {
            let root = tempdir!();
            let ts = <Timestamp as Default>::default();

            let id = {
                let part = Partition::new(&root, *ts)
                    .chain_err(|| "Failed to create partition")
                    .unwrap();

                part.id
            };

            let part = Partition::with_data(&root)
                .chain_err(|| "Failed to read partition data")
                .unwrap();

            assert_eq!(part.id, id);
            assert_eq!(part.ts_min, ts);
            assert_eq!(part.ts_max, ts);
        }
    }

    mod memory {
        use super::*;

        #[test]
        fn scan_ts() {
            super::scan_ts(MemoryBlockType::U64Dense);
        }

        #[test]
        fn update_meta() {
            super::update_meta(MemoryBlockType::U64Dense);
        }

        mod serialize {
            use super::*;

            #[test]
            fn block() {
                super::super::serialize::block(block_test_impl!(MemoryBlockType));
            }
        }

        mod deserialize {
            use super::*;

            #[test]
            fn block() {
                super::super::deserialize::block(block_test_impl!(MemoryBlockType));
            }
        }
    }


    #[cfg(feature = "mmap")]
    mod mmap {
        use super::*;

        #[test]
        fn scan_ts() {
            super::scan_ts(MmapBlockType::U64Dense);
        }

        #[test]
        fn update_meta() {
            super::update_meta(MmapBlockType::U64Dense);
        }

        mod serialize {
            use super::*;

            #[test]
            fn block() {
                super::super::serialize::block(block_test_impl!(MmapBlockType));
            }
        }

        mod deserialize {
            use super::*;

            #[test]
            fn block() {
                super::super::deserialize::block(block_test_impl!(MmapBlockType));
            }
        }
    }
}
