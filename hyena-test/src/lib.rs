mod helpers;
pub mod perf;
pub mod random;
pub mod seq;
pub mod string;
pub mod tempfile;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
