#![cfg_attr(feature = "nightly", feature(test))]
#[cfg(all(feature = "nightly", test))]
extern crate test;

#[macro_use]
extern crate serde_derive;

pub mod collections;
pub(crate) mod error;
pub mod iter;
pub mod libc;
pub mod lock;
pub mod map_type;
pub mod serde_utils;
pub mod ty;

mod macros;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
