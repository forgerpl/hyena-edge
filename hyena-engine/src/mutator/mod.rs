use error::*;
use std::collections::HashMap;
use ty::{ColumnId, Fragment, FragmentRef};

pub mod append;

// @todo: we have a trait with the same name in block::BlockData
pub type BlockData = HashMap<ColumnId, Fragment>;
pub type BlockRefData<'frag> = HashMap<ColumnId, FragmentRef<'frag>>;
