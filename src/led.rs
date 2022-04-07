use crate::str::const_parse::*;
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
                itertools::Either::Right(boi)
            } else {
                itertools::Either::Left(boi.rev())
            }
        })
        .flatten();

    ziggy
}

#[inline]
fn mk_segmented_iterator(
    num_segments: usize,
    px_per_segment: usize,
    segment_width: usize,
) -> impl Iterator<Item = usize> {
    let num_cols = px_per_segment / segment_width;
    let inner = 0..num_segments;
    let outer = 0..num_cols;

    outer
        .into_iter()
        .map(move |col| {
            inner
                .clone()
                .into_iter()
                .map(move |row| col + row * num_cols)
        })
        .flatten()
}

#[test]
fn test_mk_segmented_iterator() {
    fn c(it: impl Iterator<Item = usize>) -> Vec<usize> {
        it.collect()
    }
    assert_eq!(vec![0, 2, 1, 3], c(mk_segmented_iterator(2, 6, 3)));
    assert_eq!(vec![0, 2, 4, 1, 3, 5], c(mk_segmented_iterator(3, 4, 2)));
}

fn array_chunks_reordered<const C: usize, T, I>(
    src: &[T],
    order: I,
) -> impl Iterator<Item = &[T; C]>
where
    I: IntoIterator<Item = usize>,
{
    dbg!(&C, src.len());
    order.into_iter().map(|chunk_idx| {
        let start = chunk_idx.checked_mul(C).unwrap();
        src[start..start + C].try_into().unwrap()
    })
}

#[test]
fn test_segmented() {
    let data = [10u8, 20, 30, 40, 50, 60, 11, 22, 33, 44, 55, 66];
    let it = segmented::<_, 2, 6, 2>(&data, false);
    assert_eq!(
        it.cloned().collect::<Vec<_>>(),
        vec![10u8, 20, 11, 22, 30, 40, 33, 44, 50, 60, 55, 66] // segs: 2, num_rows: 3, seg_w: 2
    );
}
pub fn segmented<
    T,
    const NUM_SEGMENTS: usize,
    const PX_PER_SEGMENT: usize,
    const SEG_WIDTH: usize,
>(
    data: &[T],
    flatten: bool,
) -> impl Iterator<Item = &T> {
    let it = mk_segmented_iterator(NUM_SEGMENTS, PX_PER_SEGMENT, SEG_WIDTH);
    let res = array_chunks_reordered::<SEG_WIDTH, _, _>(data, it).flatten();
    res

    // a b c  d e f
    // g h i  j k l

    // 1 2 3  7 8 9  4 5 6  10 11 12
    // idx: 0,2,1,4

    // a b c  d e f  g h i
    // j k l  m n o  p q r
    // 1,4,2,5,3,6
    // n, n+cols, n+1, n+1+cols, n+2, n+2+cols

    // a b  c d  e f
    // g h  i j  k l

    // 1 2  7 8  3 4  9 10  5 6  11 12
    // I    II   I    II    I    II

    // a b  c d
    // e f  g h
    // i j  k l

    // 1 2  5 6  9 10!  3 4  7 8  11 12
    // I    II   III    I    II   III
    // chunk idx:
    // 1    3    5      2    4    6
    // n+0*cols, n+1*cols, n+2*cols, n+1+0*cols, n+1+1*cols, n+1+2*cols
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
