use crate::error::*;
use bincode;

pub use self::de::{deserialize, deserialize_from};
pub use self::ser::{serialize, serialize_into};

pub(crate) mod ser {
    use super::*;
    use serde::Serialize;
    use std::io::Write;

    pub fn serialize<T>(value: &T) -> Result<Vec<u8>>
    where
        T: ?Sized + Serialize,
    {
        bincode::serialize(value, bincode::Infinite)
            .with_context(|_| "Serialization failed")
            .map_err(|e| e.into())
    }

    pub fn serialize_into<T, W>(value: &T, writer: &mut W) -> Result<()>
    where
        T: ?Sized + Serialize,
        W: ?Sized + Write,
    {
        bincode::serialize_into(writer, value, bincode::Infinite)
            .with_context(|_| "Serialization failed")
            .map_err(|e| e.into())
    }
}

pub(crate) mod de {
    use super::*;
    use serde::Deserialize;
    use std::io::Read;

    pub fn deserialize<'de, T>(data: &'de [u8]) -> Result<T>
    where
        T: Deserialize<'de>,
    {
        bincode::deserialize(data)
            .with_context(|_| "Deserialization failed")
            .map_err(|e| e.into())
    }

    pub fn deserialize_from<T, R>(reader: &mut R) -> Result<T>
    where
        for<'de> T: Deserialize<'de>,
        R: Read,
    {
        bincode::deserialize_from(reader, bincode::Infinite)
            .with_context(|_| "Deserialization failed")
            .map_err(|e| e.into())
    }
}

#[macro_export]
macro_rules! deserialize {
    (file $name: expr, $T: ty) => {{
        use std::fs::File;
        use $crate::serde_utils::deserialize_from;

        let mut file = File::open($name)
            .with_context(|_| format!("Failed to open file {:?} for deserialization", $name))?;

        deserialize_from::<$T, _>(&mut file)
    }};

    (file $name: expr) => {{
        use std::fs::File;
        use $crate::serde_utils::deserialize_from;

        let mut file = File::open($name)
            .with_context(|_| format!("Failed to open file {:?} for deserialization", $name))?;

        deserialize_from(&mut file)
    }};

    (buf $buf: expr, $T: ty) => {{
        use $crate::serde_utils::deserialize;

        let mut buf = $buf;

        deserialize::<$T, _>(&mut buf)
    }};

    (buf $buf: expr) => {{
        use $crate::serde_utils::deserialize;

        let mut buf = $buf;

        deserialize(&mut buf)
    }};
}

#[macro_export]
macro_rules! serialize {
    (file $name: expr, $value: expr) => {{
        use std::fs::File;
        use $crate::serde_utils::serialize_into;

        let mut file = File::create($name)
            .with_context(|_| format!("Failed to open file {:?} for serialization", $name))?;

        let value = $value;

        serialize_into(&value, &mut file)
    }};

    (buf $value: expr) => {{
        use $crate::serde_utils::serialize;

        let value = $value;

        serialize(&value)
    }};
}
