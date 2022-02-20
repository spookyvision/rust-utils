pub trait Uuid {
    type T: AsRef<str>;
    type Error;
    fn uuid(&self) -> Result<Self::T, Self::Error>;
}

pub mod const_parse {
    // Based on Gist by DutchGhost
    // https://gist.github.com/DutchGhost/d8604a3c796479777fe9f5e25d855cfd
    // Changes:
    // - Conditional compilation to support either 64 or 32-bit
    // - Added panic error message
    // - Added convenience function parse_unwrap
    // - Added tests

    #![feature(const_if_match)]
    #![feature(const_panic)]
    #![feature(const_fn)]

    #[derive(Clone, Copy, Debug)]
    pub enum ParseIntError {
        InvalidDigit,
        Overflow,
    }

    const fn parse_byte(b: u8) -> Result<usize, ParseIntError> {
        let r = b.wrapping_sub(48);

        if r > 9 {
            Err(ParseIntError::InvalidDigit)
        } else {
            Ok(r as usize)
        }
    }

    #[cfg(target_pointer_width = "64")]
    const POW_LENGTH: usize = 20;

    #[cfg(target_pointer_width = "32")]
    const POW_LENGTH: usize = 10;

    #[cfg(target_pointer_width = "64")]
    pub(crate) const POW10: [usize; POW_LENGTH] = [
        10_000_000_000_000_000_000,
        1_000_000_000_000_000_000,
        100_000_000_000_000_000,
        10_000_000_000_000_000,
        1_000_000_000_000_000,
        100_000_000_000_000,
        10_000_000_000_000,
        1_000_000_000_000,
        100_000_000_000,
        10_000_000_000,
        1_000_000_000,
        100_000_000,
        10_000_000,
        1_000_000,
        100_000,
        10_000,
        1_000,
        100,
        10,
        1,
    ];

    #[cfg(target_pointer_width = "32")]
    pub(crate) const POW10: [usize; POW_LENGTH] = [
        1_000_000_000,
        100_000_000,
        10_000_000,
        1_000_000,
        100_000,
        10_000,
        1_000,
        100,
        10,
        1,
    ];

    macro_rules! try_const {
        ($e:expr) => {
            match $e {
                Ok(ok) => ok,
                Err(e) => return Err(e),
            }
        };
    }

    macro_rules! const_index {
    ($e:expr, else $($or:tt)*) => {
        match $e {
            ConstIndex::Break => $($or)*,
            ConstIndex::Continue(index) => index
        }
    }
}

    #[derive(Debug, Clone, Copy)]
    enum ConstIndex {
        Continue(usize),
        Break,
    }

    impl ConstIndex {
        const fn next(&self) -> Self {
            match self {
                Self::Continue(n) if *n > 0 => ConstIndex::Continue(*n - 1),
                _ => ConstIndex::Break,
            }
        }
    }

    const fn parse_inner(
        bytes: &[u8],
        current_index: ConstIndex,
        index_into_const_table: ConstIndex,
    ) -> Result<usize, ParseIntError> {
        if bytes.len() > POW_LENGTH {
            return Err(ParseIntError::Overflow);
        }

        let index = const_index!(current_index, else return Ok(0));
        let index_const_table = const_index!(index_into_const_table, else return Ok(0));

        let r = try_const!(parse_byte(bytes[index])) * POW10[index_const_table];

        let tail = try_const!(parse_inner(
            bytes,
            current_index.next(),
            index_into_const_table.next()
        ));

        Ok(tail + r)
    }

    pub const fn parse(b: &str) -> Result<usize, ParseIntError> {
        parse_inner(
            b.as_bytes(),
            ConstIndex::Continue(b.len() - 1),
            ConstIndex::Continue(POW_LENGTH - 1),
        )
    }

    pub const fn parse_unwrap(b: &str) -> usize {
        match parse(b) {
            Ok(t) => t,
            _ => panic!("Could not parse string into a usize"),
        }
    }
}
