use std::{
    cmp::Reverse,
    convert::Infallible,
    fmt::{Debug, Display},
    iter::Sum,
    ops::{Add, Neg, Sub, SubAssign},
    str::FromStr,
};

use num::{One, Zero};
use ordered_float::NotNan;
use radix_heap::Radix;
use rand_distr::uniform::SampleUniform;

/// Generic definition of a weight (typically either `f64` or `i64`)
pub trait Weight:
    Sized
    + Copy
    + Zero
    + One
    + PartialOrd
    + PartialEq
    + SampleUniform
    + Add<Output = Self>
    + Sub<Output = Self>
    + SubAssign
    + Neg<Output = Self>
    + Display
    + Debug
    + Sum
{
    /// `RadixHeapMap` requires values that implement the following. Additionally we need a MinHeap
    /// while `RadixHeapMap` is a MaxHeap
    type RadixWeight: Radix + Ord + Copy;

    /// Threshold near `0`
    const ZERO_THRESHOLD: Self;

    /// Maximum positive value, i.e. `INFINITY` for `f64` and `2^64 - 1` for `i64`
    const MAX: Self;

    // Float Conversions are explicitly implemented here since `f64` do not implement
    // `From<i64>` and so on

    /// Convert from an `f64` to `Self`
    fn from_f64(val: f64) -> Self;

    /// Convert `Self` to `f64`
    fn to_f64(self) -> f64;

    /// Convert the weight into its `RadixWeight` form
    fn to_radix(self) -> Self::RadixWeight;

    /// Convert the weight back from its `RadixWeight` form
    fn from_radix(radix: Self::RadixWeight) -> Self;

    /// Rounds `self` up to `value` if they are close
    ///
    /// Note that this is mainly supposed to be used to correct floating point errors thus for `f32` and `f64` implementing this trait.
    /// Non-float types should leave this method empty.
    fn round_up(&mut self, value: Self) {
        if value > *self {
            *self = value;
        }
    }
}

macro_rules! weight_impl_float {
    ($($t:ty),*) => {
        $(
            impl Weight for $t {
                type RadixWeight = Reverse<NotNan<$t>>;
                const ZERO_THRESHOLD: Self = 1e-8 as $t;
                const MAX: Self = <$t>::INFINITY;

                #[inline]
                fn from_f64(val: f64) -> Self {
                    val as $t
                }

                #[inline]
                fn to_f64(self) -> f64 {
                    self as f64
                }

                #[inline]
                fn to_radix(self) -> Self::RadixWeight {
                    Reverse(NotNan::new(self).expect("A NaN value was encountered"))
                }

                #[inline]
                fn from_radix(radix: Self::RadixWeight) -> Self {
                    radix.0.into_inner()
                }
            }
        )*
    };
}

macro_rules! weight_impl_int {
    ($($t:ty),*) => {
        $(
            impl Weight for $t {
                type RadixWeight = Reverse<$t>;
                const ZERO_THRESHOLD: Self = 0 as $t;
                const MAX: Self = <$t>::MAX;

                #[inline]
                fn from_f64(val: f64) -> Self {
                    val as $t
                }

                #[inline]
                fn to_f64(self) -> f64 {
                    self as f64
                }

                #[inline]
                fn to_radix(self) -> Self::RadixWeight {
                    Reverse(self)
                }

                #[inline]
                fn from_radix(radix: Self::RadixWeight) -> Self {
                    radix.0
                }

                /// We should never need to round integer types
                fn round_up(&mut self, _: Self) {}
            }
        )*
    };
}

weight_impl_float!(f32, f64);
weight_impl_int!(i8, i16, i32, i64, i128);

/// Types for which `Weight` has been implemented
///
/// This enum is only used as a helper in `main`
#[derive(Debug, Clone, Copy)]
pub enum WeightType {
    F32,
    F64,
    I8,
    I16,
    I32,
    I64,
}

impl FromStr for WeightType {
    // We should always use a Weight-Type - so default to `f64`
    type Err = Infallible;

    fn from_str(t: &str) -> Result<Self, Self::Err> {
        if t.starts_with('i') {
            if t.contains('8') {
                Ok(WeightType::I8)
            } else if t.contains('1') || t.contains('6') {
                Ok(WeightType::I16)
            } else if t.contains('3') || t.contains('2') {
                Ok(WeightType::I32)
            } else {
                Ok(WeightType::I64)
            }
        } else if t.contains('3') || t.contains('2') {
            Ok(WeightType::F32)
        } else {
            Ok(WeightType::F64)
        }
    }
}
