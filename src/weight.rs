use std::{
    convert::Infallible,
    fmt::{Debug, Display},
    iter::Sum,
    ops::{Add, AddAssign, Neg, Sub, SubAssign},
    str::FromStr,
};

use num::{One, Zero};
use rand::Rng;
use rand_distr::uniform::SampleUniform;
use serde_derive::{Deserialize, Serialize};

use crate::utils::Radix;

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
    + Neg<Output = Self>
    + AddAssign
    + SubAssign
    + Display
    + Debug
    + Sum
    + Radix
    + FromStr
    + Send
    + Sync
    + Default
{
    /// Maximum positive value, i.e. `INFINITY` for `f64` and `2^63 - 1` for `i64`
    const MAX: Self;

    /// Minimum value, i.e. `-INFINITY` for `f64` and `-2^63 - 1`
    const MIN: Self;

    // Float Conversions are explicitly implemented here since `f64` does not implement
    // `From<i64>` and so on

    /// Convert from an `f64` to `Self`
    fn from_f64(val: f64) -> Self;

    /// Convert `Self` to `f64`
    fn to_f64(self) -> f64;

    fn from_i64(val: i64) -> Self;

    fn to_i64(self) -> i64;

    /// Rounds `self` up to `value` if `value` is greater
    ///
    /// Note that this is mainly supposed to be used to correct floating point errors thus for `f32` and `f64` implementing this trait.
    /// Non-float types should leave this method empty.
    #[inline]
    fn round_up(&mut self, value: Self) {
        if value > *self {
            *self = value;
        }
    }

    fn as_type() -> WeightType;
}

macro_rules! weight_impl_float {
    ($($t:ty, $e:expr),*) => {
        $(
            impl Weight for $t {
                const MAX: Self = <$t>::INFINITY;
                const MIN: Self = <$t>::NEG_INFINITY;

                #[inline]
                fn from_f64(val: f64) -> Self {
                    val as $t
                }

                #[inline]
                fn to_f64(self) -> f64 {
                    self as f64
                }

                #[inline]
                fn from_i64(val: i64) -> Self {
                    val as $t
                }

                #[inline]
                fn to_i64(self) -> i64 {
                    self as i64
                }

                #[inline]
                fn as_type() -> WeightType {
                    $e
                }
            }
        )*
    };
}

macro_rules! weight_impl_int {
    ($($t:ty, $e:expr),*) => {
        $(
            impl Weight for $t {
                const MAX: Self = <$t>::MAX;
                const MIN: Self = <$t>::MIN;

                #[inline]
                fn from_f64(val: f64) -> Self {
                    val as $t
                }

                #[inline]
                fn to_f64(self) -> f64 {
                    self as f64
                }

                #[inline]
                fn from_i64(val: i64) -> Self {
                    val as $t
                }

                #[inline]
                fn to_i64(self) -> i64 {
                    self as i64
                }

                /// We should never need to round integer types
                fn round_up(&mut self, _: Self) {}

                #[inline]
                fn as_type() -> WeightType {
                    $e
                }
            }
        )*
    };
}

weight_impl_float!(f32, WeightType::F32, f64, WeightType::F64);
weight_impl_int!(
    i8,
    WeightType::I8,
    i16,
    WeightType::I16,
    i32,
    WeightType::I32,
    i64,
    WeightType::I64
);

/// Types for which `Weight` has been implemented
///
/// This enum is only used as a helper in `main`
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub enum WeightType {
    F32,
    F64,
    I8,
    I16,
    I32,
    I64,
}

impl FromStr for WeightType {
    // We should always use a WeightType - so default to `f64`
    type Err = Infallible;

    fn from_str(t: &str) -> Result<Self, Self::Err> {
        if t.starts_with('i') {
            if t.contains('8') {
                Ok(WeightType::I8)
            } else if t.contains('1') {
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

/// Starting weights for edges
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum InitialWeights {
    /// Start with `max_weight`
    Maximum = 0,
    /// Start with `0`
    Zero = 1,
    /// Start with a uniform weight in `[0,max_weight]`
    Uniform = 2,
}

impl InitialWeights {
    /// Used for iteration over all weigh types
    pub const ALL: [InitialWeights; 3] = [
        InitialWeights::Maximum,
        InitialWeights::Zero,
        InitialWeights::Uniform,
    ];

    /// Generate a weight based on `self` and `max_weight`: `rng` needs to be provided for the
    /// `Uniform` case
    #[inline]
    pub fn generate_weight<R: Rng, W: Weight>(&self, rng: &mut R, max_weight: W) -> W {
        match self {
            Self::Maximum => max_weight,
            Self::Zero => W::zero(),
            Self::Uniform => rng.gen_range(W::zero()..=max_weight),
        }
    }

    /// Return a char representing the initial weight type: used for logging in experiments
    #[inline]
    pub fn to_char(&self) -> char {
        match self {
            Self::Maximum => 'm',
            Self::Zero => 'z',
            Self::Uniform => 'u',
        }
    }
}

impl FromStr for InitialWeights {
    // We should always use a weight function - default to `Uniform`
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with('m') {
            Ok(Self::Maximum)
        } else if s.starts_with('z') {
            Ok(Self::Zero)
        } else {
            Ok(Self::Uniform)
        }
    }
}
