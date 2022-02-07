use crate::const_parse::*;
pub struct LedConfig {
    pub w: usize,
    pub h: usize,
    zigzag: bool,
}

macro_rules! env_or_default {
    ($env_var: literal, $default: literal) => {{
        match option_env!($env_var) {
            Some(s) => s,
            None => $default,
        }
    }};
}

impl LedConfig {
    pub const fn new() -> Self {
        Self {
            w: parse_unwrap(env_or_default!("LEDS_W", "4")),
            h: parse_unwrap(env_or_default!("LEDS_H", "4")),
            zigzag: match option_env!("LEDS_NO_ZIGZAG") {
                Some(_) => false,
                None => true,
            },
        }
    }

    pub const fn num_leds(&self) -> usize {
        self.w * self.h
    }
}

pub fn ziggy_components<const ROW_LEN: usize, T>(data: &[T]) -> impl Iterator<Item = &T>
where
    T: Sized,
{
    let (chunks, _remainder) = data.as_chunks::<{ ROW_LEN }>();

    let ziggy = chunks
        .into_iter()
        .enumerate()
        .map(|(idx, chunky_boi)| {
            let boi = chunky_boi.iter();

            if idx % 2 == 0 {
                itertools::Either::Left(boi.rev())
            } else {
                itertools::Either::Right(boi)
            }
        })
        .flatten();

    ziggy
}

// needs #![feature(generic_const_exprs)]
//
// pub fn ziggy_bytes<const ROW_LEN: usize>(data: &[u8]) -> impl Iterator<Item = &[u8]>
// where
//     [(); ROW_LEN * 3]:,
// {
//     let (chunks, _remainder) = data.as_chunks::<{ ROW_LEN * 3 }>();

//     let ziggy = chunks
//         .into_iter()
//         .enumerate()
//         .map(|(idx, chunky_boi)| {
//             let boi = chunky_boi.chunks_exact(3);

//             if idx % 2 == 0 {
//                 itertools::Either::Left(boi.rev())
//             } else {
//                 itertools::Either::Right(boi)
//             }
//         })
//         .flatten();

//     ziggy
// }
