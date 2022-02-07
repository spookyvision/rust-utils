#![no_std]
#![feature(slice_as_chunks)]
// for ziggy bytes
//#![feature(generic_const_exprs)]

pub mod const_parse;
pub mod led;
pub mod math;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        // cool
    }
}
