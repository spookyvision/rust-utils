use itertools::Either;

use crate::str::const_parse::*;
pub struct LedConfig {
    pub w: usize,
    pub h: usize,
    pub zigzag: bool,
    pub segments: usize,
    pub right_to_left: bool,
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
            segments: parse_unwrap(env_or_default!("LEDS_SEGMENTS", "1")),
            w: parse_unwrap(env_or_default!("LEDS_W", "4")),
            h: parse_unwrap(env_or_default!("LEDS_H", "4")),
            zigzag: match option_env!("LEDS_NO_ZIGZAG") {
                Some(_) => false,
                None => true,
            },
            right_to_left: match option_env!("LEDS_RTL") {
                Some(_) => true,
                None => false,
            },
        }
    }

    pub const fn num_leds(&self) -> usize {
        self.w * self.h
    }
}

pub fn ziggy_components<T, const ROW_LEN: usize>(data: &[T]) -> impl Iterator<Item = &T>
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
                itertools::Either::Left(boi)
            } else {
                itertools::Either::Right(boi.rev())
            }
        })
        .flatten();

    ziggy
}

#[inline]
fn mk_segmented_iterator(
    num_segments: usize,
    segment_height: usize,
    right_to_left: bool,
) -> impl Iterator<Item = usize> {
    let inner = 0..segment_height;
    let outer = 0..num_segments;

    outer
        .into_iter()
        .map(move |segment| {
            let segment_offset = if right_to_left {
                num_segments - 1 - segment
            } else {
                segment
            };
            inner
                .clone()
                .into_iter()
                .map(move |row| row * num_segments + segment_offset)
        })
        .flatten()
}

#[test]
fn test_mk_segmented_iterator() {
    fn c(it: impl Iterator<Item = usize>) -> Vec<usize> {
        it.collect()
    }

    // 0 1 2  3 4 5
    // 6 7 8  9 a b
    assert_eq!(vec![1, 3, 0, 2], c(mk_segmented_iterator(2, 2, true)));

    // 0 1 2  3 4 5
    // 6 7 8  9 a b
    assert_eq!(vec![0, 2, 1, 3], c(mk_segmented_iterator(2, 2, false)));

    // 0 1  2 3  4 5
    // 6 7  8 9  a b
    assert_eq!(
        vec![0, 3, 1, 4, 2, 5],
        c(mk_segmented_iterator(3, 2, false))
    );

    // 0 1
    // 2 3
    // 4 5
    // ??????

    // assert_eq!(
    //     vec![0, 2, 4, 1, 3, 5],
    //     mk_segmented_iterator(3, 4, 2, true).collect::<Vec<_>>()
    // );
}

#[inline]
// thanks to H2CO3 - https://users.rust-lang.org/t/reorder-iteration-without-heap-allocation/73859/3
fn array_chunks_reordered<const C: usize, T, I>(
    src: &[T],
    order: I,
) -> impl Iterator<Item = &[T; C]>
where
    I: IntoIterator<Item = usize>,
{
    order.into_iter().map(|chunk_idx| {
        let start = chunk_idx.checked_mul(C).unwrap();
        let res = src[start..start + C].try_into().unwrap();
        res
    })
}

#[test]
fn test_segmented() {
    // segs: 2, seg_h: 2, seg_w: 3
    let data = [10u8, 20, 30, 40, 50, 60, 11, 22, 33, 44, 55, 66];

    let it = segmented::<_, 2, 6, 3>(&data, false, false);
    assert_eq!(
        vec![10u8, 20, 30, 11, 22, 33, 40, 50, 60, 44, 55, 66],
        it.cloned().collect::<Vec<_>>()
    );

    let it = segmented::<_, 2, 6, 3>(&data, true, false);
    assert_eq!(
        vec![10u8, 20, 30, 33, 22, 11, 40, 50, 60, 66, 55, 44],
        it.cloned().collect::<Vec<_>>()
    );

    let data = [1, 2, 3, 4, 5, 6, 11, 22, 33, 44, 55, 66];
    let it = segmented::<_, 2, 6, 2>(&data, true, false);
    assert_eq!(
        vec![1, 2, 6, 5, 33, 44, 3, 4, 22, 11, 55, 66],
        it.cloned().collect::<Vec<_>>()
    );
}

/// builds an iterator over several adjacent display segments whose rows may be zig zag ordered
#[inline]
pub fn segmented<
    T,
    const NUM_SEGMENTS: usize,
    const PX_PER_SEGMENT: usize,
    const SEG_WIDTH: usize,
>(
    data: &[T],
    zigzag: bool,
    right_to_left: bool,
    rev_odd: bool,
) -> impl Iterator<Item = &T> {
    let row_cmp = if rev_odd { 1 } else { 0 };
    let it = mk_segmented_iterator(NUM_SEGMENTS, PX_PER_SEGMENT / SEG_WIDTH, right_to_left);
    let res = array_chunks_reordered::<SEG_WIDTH, _, _>(data, it)
        .enumerate()
        .map(move |(idx, seg)| {
            let rev = zigzag && ((idx % (PX_PER_SEGMENT / SEG_WIDTH)) % 2 == row_cmp);
            if rev {
                Either::Left(seg.into_iter().rev())
            } else {
                Either::Right(seg.into_iter())
            }
        })
        .flatten();
    res
}

#[cfg(test)]
#[test]
fn it_ziggys() -> Result<(), Box<dyn std::error::Error>> {
    let arr = [1u8, 2, 3, 4, 5, 6, 7, 8, 9];
    let zigged: Vec<_> = ziggy_components::<_, 3>(&arr).cloned().collect();
    assert_eq!(&[1u8, 2, 3, 6, 5, 4, 7, 8, 9], zigged.as_slice());
    Ok(())
}

#[cfg(test)]
#[test]
fn it_ziggys_segments() -> Result<(), Box<dyn std::error::Error>> {
    let arr = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
    // let zigged: Vec<_> = ziggy_components::<_, 3, 2>(&arr).cloned().collect();

    // desired display
    // 1 2 3   4  5  6
    // 7 8 9  10 11 12

    // pixels:
    // 1 2 3   7  8  9
    // 6 5 4  10 11 12

    // assert_eq!(&[1u8, 2, 3, 6, 5, 4, 7, 8, 9], zigged.as_slice());
    Ok(())
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
