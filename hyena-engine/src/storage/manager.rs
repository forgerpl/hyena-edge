use error::*;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::cell::RefCell;
use ty::Timestamp;
use fs::ensure_dir;
use chrono::prelude::*;


#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RootManager {
    data: PathBuf,
}

impl RootManager {
    fn new<P: AsRef<Path>>(data_root: P) -> Result<RootManager> {
        let data = ensure_dir(data_root)
            .chain_err(|| "Failed to manage root directory")?;

        Ok(RootManager { data })
    }
}

impl AsRef<Path> for RootManager {
    fn as_ref(&self) -> &Path {
        self.data.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PartitionManager {
    partition_root: PathBuf,
}

impl PartitionManager {
    fn new<P: AsRef<Path>, T: ToString, TS: Into<Timestamp>>(
        data_root: P,
        id: T,
        ts: TS,
    ) -> Result<PartitionManager> {

        let root = ensure_dir(data_root)
            .chain_err(|| "Failed to manage partition root directory")?;

        let ts: DateTime<Utc> = ts.into().into();

        let mut partition_root = root.to_path_buf();

        let day = ts.day();

        partition_root.push(format!("{}-{}", ts.year(), ts.month() / 4));
        partition_root.push(format!("{}", day / 7));
        partition_root.push(format!("{:02}", day));
        partition_root.push(id.to_string());

        let partition_root = ensure_dir(partition_root)
            .chain_err(|| "Failed to ensure partition root directory")?;

        Ok(PartitionManager {
            partition_root,
        })
    }
}

impl AsRef<Path> for PartitionManager {
    fn as_ref(&self) -> &Path {
        self.partition_root.as_ref()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;


    #[test]
    fn nonexistent() {
        use std::fs::remove_dir;

        let root = { tempdir!().as_ref().to_path_buf() };

        assert!(!root.exists());

        let manager = RootManager::new(&root)
            .chain_err(|| "Failed to create manager")
            .unwrap();

        assert!(root.exists());
        assert!(root.is_dir());

        remove_dir(root)
            .chain_err(|| "Failed to clean up temporary directory")
            .unwrap();
    }

    #[test]
    fn existing() {
        let root = tempdir!();

        let manager = RootManager::new(&root)
            .chain_err(|| "Failed to create manager")
            .unwrap();

        let root_path = root.as_ref();

        assert!(root_path.exists());
        assert!(root_path.is_dir());
    }

    #[test]
    fn partition() {
        let root = tempdir!();

        let id = Uuid::new_v4();
        let ts = <Timestamp as Default>::default();

        let pman = PartitionManager::new(&root, &id, *ts)
            .chain_err(|| "Failed to create manager")
            .unwrap();

        let t = pman.as_ref().strip_prefix(root.as_ref())
            .chain_err(|| "Produced path is not a subdirectory of root")
            .unwrap()
            .components()
            .count();

        assert_eq!(t, 4);
    }
}
