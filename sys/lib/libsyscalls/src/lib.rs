#![no_std]
pub mod heap;
pub mod syscalls;

pub use ::syscalls::errors;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
