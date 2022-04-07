#![cfg_attr(not(feature = "std"), no_std)]
#![feature(slice_as_chunks)]
#![feature(generic_const_exprs)]

pub mod led;
pub mod math;
pub mod peripherals;
pub mod str;

#[cfg(feature = "serde")]
pub mod serde;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        // cool
    }
}
