use crate::ty::{ColumnId, Fragment, FragmentRef};
use hyena_common::collections::HashMap;

pub mod append;

pub use self::append::Append;

// @todo: we have a trait with the same name in block::BlockData
pub type BlockData = HashMap<ColumnId, Fragment>;
pub type BlockRefData<'frag> = HashMap<ColumnId, FragmentRef<'frag>>;
