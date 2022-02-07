use core::marker::PhantomData;

use num_traits::{float::FloatCore, AsPrimitive, NumOps};

pub trait F32Ext {
    fn gl_fract(&self) -> Self;
}

impl F32Ext for f32 {
    fn gl_fract(&self) -> Self {
        self - self.floor()
    }
}

pub trait Construct<T: Default + Copy, SUM, const N: usize> {
    fn construct() -> MovingAverage<T, SUM, N> {
        MovingAverage {
            vals: [T::default(); N],
            idx: 0,
            valid: false,
            _sum: PhantomData,
        }
    }
}

impl<const N: usize> Construct<u8, u16, N> for MovingAverage<u8, u16, N> {}
impl<const N: usize> Construct<f32, f32, N> for MovingAverage<f32, f32, N> {}

impl<T, SUM, const N: usize> Default for MovingAverage<T, SUM, N>
where
    T: Default + Copy,
    MovingAverage<T, SUM, N>: Construct<T, SUM, N>,
{
    fn default() -> Self {
        Self::construct()
    }
}

// fn make<T, const N: usize>() -> MovingAverage<T, _, N> {}

impl<T, SUM, const N: usize> MovingAverage<T, SUM, N>
where
    T: Default + Copy,
{
    pub fn new() -> MovingAverage<T, SUM, N> {
        MovingAverage {
            vals: [T::default(); N],
            idx: 0,
            valid: false,
            _sum: PhantomData,
        }
    }
}

pub struct MovingAverage<T, SUM, const N: usize> {
    vals: [T; N],
    idx: usize,
    valid: bool,
    _sum: PhantomData<SUM>,
}

impl<T, SUM, const N: usize> MovingAverage<T, SUM, N>
where
    T: Default + AsPrimitive<SUM>,
    SUM: NumOps + AsPrimitive<T>,
    usize: AsPrimitive<SUM>,
{
    pub fn avg(&self) -> Option<T> {
        if !self.valid {
            return None;
        }
        let init: SUM = 0usize.as_();
        let sum = self.vals.iter().fold(init, |acc, item| acc + item.as_());
        let div: SUM = N.as_();
        let res: T = (sum / div).as_();
        Some(res)
    }

    pub fn push(&mut self, val: T) {
        self.vals[self.idx] = val;
        self.idx = (self.idx + 1) % N;
        if self.idx == 0 {
            self.valid = true;
        }
    }
}

#[cfg(test)]
mod tests {

    use super::MovingAverage;

    #[test]
    fn empty() {
        let a = MovingAverage::<u8, _, 5>::default();
        let s = a.avg();
        assert_eq!(s, None);
    }

    #[test]
    fn custom_types() {
        let mut a = MovingAverage::<f32, f64, 5>::new();
        a.push(1.);
        for _ in 1..5 {
            a.push(0.);
        }
        let s = a.avg();
        assert_eq!(s, Some(1. / 5.));
    }

    #[test]
    fn neg() {
        let mut a = MovingAverage::<i8, i8, 5>::new();
        a.push(3);
        a.push(-3);
        for _ in 2..5 {
            a.push(0);
        }
        let s = a.avg();
        assert_eq!(s, Some(0));
    }

    #[test]
    fn rollover() {
        let mut a = MovingAverage::<u8, _, 5>::default();
        for i in 0..5 {
            a.push(10);
        }
        a.push(5);
        assert_eq!(a.avg(), Some(45 / 5));

        a.push(5);
        assert_eq!(a.avg(), Some(40 / 5));
    }
}
