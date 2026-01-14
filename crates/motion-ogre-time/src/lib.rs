#![deny(missing_docs)]
#![forbid(unsafe_code)]

//! Frame-accurate time primitives for Motion Ogre.
//!
//! The editing engine uses rational numbers for all timeline time values to ensure
//! deterministic, frame-accurate results across platforms.

use core::cmp::Ordering;
use core::fmt;
use core::ops::{Add, Div, Mul, Neg, Sub};

use serde::{Deserialize, Serialize};

/// Errors that can occur when constructing time primitives.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimeError {
    /// The denominator is zero.
    ZeroDenominator,
    /// The value must be greater than zero (used for frame rates).
    NonPositiveValue,
}

impl fmt::Display for TimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TimeError::ZeroDenominator => write!(f, "denominator must be non-zero"),
            TimeError::NonPositiveValue => write!(f, "value must be greater than zero"),
        }
    }
}

impl std::error::Error for TimeError {}

fn gcd_i64(mut a: i64, mut b: i64) -> i64 {
    a = a.abs();
    b = b.abs();

    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }

    if a == 0 { 1 } else { a }
}

/// A reduced rational number representing a time value.
///
/// The value is stored as `numerator / denominator` where `denominator > 0` and the fraction is
/// always reduced by its greatest common divisor.
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct RationalTime {
    numerator: i64,
    denominator: i64,
}

impl RationalTime {
    /// A constant representing `0`.
    pub const ZERO: Self = Self {
        numerator: 0,
        denominator: 1,
    };

    /// A constant representing `1`.
    pub const ONE: Self = Self {
        numerator: 1,
        denominator: 1,
    };

    /// Creates a new reduced rational value.
    pub fn new(numerator: i64, denominator: i64) -> Result<Self, TimeError> {
        if denominator == 0 {
            return Err(TimeError::ZeroDenominator);
        }

        let mut n = numerator;
        let mut d = denominator;
        if d < 0 {
            n = -n;
            d = -d;
        }

        let g = gcd_i64(n, d);
        Ok(Self {
            numerator: n / g,
            denominator: d / g,
        })
    }

    /// Creates an integer time value.
    pub fn from_i64(value: i64) -> Self {
        Self {
            numerator: value,
            denominator: 1,
        }
    }

    /// Returns the numerator.
    pub fn numerator(&self) -> i64 {
        self.numerator
    }

    /// Returns the denominator.
    pub fn denominator(&self) -> i64 {
        self.denominator
    }

    /// Returns the value in seconds as `f64`.
    pub fn as_f64(&self) -> f64 {
        self.numerator as f64 / self.denominator as f64
    }

    /// Returns `self` as `f32`.
    pub fn as_f32(&self) -> f32 {
        self.numerator as f32 / self.denominator as f32
    }

    /// Returns the floor of this rational value.
    ///
    /// This follows mathematical floor semantics (towards negative infinity).
    pub fn floor_i64(&self) -> i64 {
        let q = self.numerator / self.denominator;
        let r = self.numerator % self.denominator;
        if r == 0 {
            q
        } else if self.numerator < 0 {
            q - 1
        } else {
            q
        }
    }

    /// Returns the ceil of this rational value.
    ///
    /// This follows mathematical ceil semantics (towards positive infinity).
    pub fn ceil_i64(&self) -> i64 {
        let q = self.numerator / self.denominator;
        let r = self.numerator % self.denominator;
        if r == 0 {
            q
        } else if self.numerator > 0 {
            q + 1
        } else {
            q
        }
    }

    /// Returns an absolute value.
    pub fn abs(&self) -> Self {
        Self {
            numerator: self.numerator.abs(),
            denominator: self.denominator,
        }
    }

    /// Returns true if this value is exactly zero.
    pub fn is_zero(&self) -> bool {
        self.numerator == 0
    }
}

impl PartialEq for RationalTime {
    fn eq(&self, other: &Self) -> bool {
        self.numerator == other.numerator && self.denominator == other.denominator
    }
}

impl Eq for RationalTime {}

impl PartialOrd for RationalTime {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RationalTime {
    fn cmp(&self, other: &Self) -> Ordering {
        let a = self.numerator as i128 * other.denominator as i128;
        let b = other.numerator as i128 * self.denominator as i128;
        a.cmp(&b)
    }
}

impl Add for RationalTime {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let n = self.numerator as i128 * rhs.denominator as i128
            + rhs.numerator as i128 * self.denominator as i128;
        let d = self.denominator as i128 * rhs.denominator as i128;
        let n = i64::try_from(n).expect("RationalTime add overflow");
        let d = i64::try_from(d).expect("RationalTime add overflow");
        Self::new(n, d).expect("non-zero denominator")
    }
}

impl Sub for RationalTime {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self + (-rhs)
    }
}

impl Neg for RationalTime {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            numerator: -self.numerator,
            denominator: self.denominator,
        }
    }
}

impl Mul for RationalTime {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let n = self.numerator as i128 * rhs.numerator as i128;
        let d = self.denominator as i128 * rhs.denominator as i128;
        let n = i64::try_from(n).expect("RationalTime mul overflow");
        let d = i64::try_from(d).expect("RationalTime mul overflow");
        Self::new(n, d).expect("non-zero denominator")
    }
}

impl Div for RationalTime {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        let n = self.numerator as i128 * rhs.denominator as i128;
        let d = self.denominator as i128 * rhs.numerator as i128;
        let n = i64::try_from(n).expect("RationalTime div overflow");
        let d = i64::try_from(d).expect("RationalTime div overflow");
        Self::new(n, d).expect("non-zero denominator")
    }
}

/// A start time and duration pair.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeRange {
    start: RationalTime,
    duration: RationalTime,
}

impl TimeRange {
    /// Creates a new time range.
    pub fn new(start: RationalTime, duration: RationalTime) -> Self {
        Self { start, duration }
    }

    /// Returns the start time.
    pub fn start(&self) -> RationalTime {
        self.start
    }

    /// Returns the duration.
    pub fn duration(&self) -> RationalTime {
        self.duration
    }

    /// Returns the exclusive end time (`start + duration`).
    pub fn end(&self) -> RationalTime {
        self.start + self.duration
    }

    /// Returns true if `time` is contained within the range.
    ///
    /// The end of the range is treated as exclusive.
    pub fn contains(&self, time: RationalTime) -> bool {
        time >= self.start && time < self.end()
    }
}

/// A rational frame rate expressed as frames per second.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrameRate {
    fps: RationalTime,
}

impl FrameRate {
    /// Creates a new frame rate.
    ///
    /// For example, `FrameRate::new(24, 1)` represents 24 fps and `FrameRate::new(30000, 1001)`
    /// represents ~29.97 fps.
    pub fn new(numerator: i64, denominator: i64) -> Result<Self, TimeError> {
        let fps = RationalTime::new(numerator, denominator)?;
        if fps <= RationalTime::ZERO {
            return Err(TimeError::NonPositiveValue);
        }
        Ok(Self { fps })
    }

    /// Returns frames-per-second as a rational.
    pub fn as_rational(&self) -> RationalTime {
        self.fps
    }

    /// Converts timeline time to an exact rational frame number.
    pub fn time_to_frames_exact(&self, time: RationalTime) -> RationalTime {
        time * self.fps
    }

    /// Converts an integer frame number to timeline time.
    pub fn frames_to_time(&self, frames: i64) -> RationalTime {
        RationalTime::from_i64(frames) / self.fps
    }

    /// Converts a time to a frame index and sub-frame offset.
    pub fn time_to_frame_position(&self, time: RationalTime) -> FramePosition {
        let frames = self.time_to_frames_exact(time);
        let frame = frames.floor_i64();
        let sub = frames - RationalTime::from_i64(frame);
        FramePosition {
            frame,
            subframe: sub,
        }
    }
}

/// A frame number and sub-frame offset.
///
/// `subframe` is a rational value in the range `[0, 1)`.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct FramePosition {
    /// The integer frame index.
    pub frame: i64,
    /// The sub-frame offset.
    pub subframe: RationalTime,
}

impl FramePosition {
    /// Returns a floating point frame position.
    pub fn as_f64(&self) -> f64 {
        self.frame as f64 + self.subframe.as_f64()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rational_reduces_and_normalizes_sign() {
        let t = RationalTime::new(2, 4).unwrap();
        assert_eq!(t, RationalTime::new(1, 2).unwrap());

        let t = RationalTime::new(1, -2).unwrap();
        assert_eq!(t, RationalTime::new(-1, 2).unwrap());
    }

    #[test]
    fn rational_floor_ceil_positive() {
        let t = RationalTime::new(3, 2).unwrap();
        assert_eq!(t.floor_i64(), 1);
        assert_eq!(t.ceil_i64(), 2);
    }

    #[test]
    fn rational_floor_ceil_negative() {
        let t = RationalTime::new(-3, 2).unwrap();
        assert_eq!(t.floor_i64(), -2);
        assert_eq!(t.ceil_i64(), -1);
    }

    #[test]
    fn time_range_contains_end_is_exclusive() {
        let start = RationalTime::from_i64(10);
        let r = TimeRange::new(start, RationalTime::from_i64(5));
        assert!(r.contains(RationalTime::from_i64(10)));
        assert!(r.contains(RationalTime::from_i64(14)));
        assert!(!r.contains(RationalTime::from_i64(15)));
    }

    #[test]
    fn frame_rate_time_to_frames_subframe() {
        let fr = FrameRate::new(24, 1).unwrap();
        let t = RationalTime::new(1, 48).unwrap();
        let fp = fr.time_to_frame_position(t);
        assert_eq!(fp.frame, 0);
        assert_eq!(fp.subframe, RationalTime::new(1, 2).unwrap());
    }

    #[test]
    fn serde_round_trip_rational_time() {
        let t = RationalTime::new(30000, 1001).unwrap();
        let s = serde_json::to_string(&t).unwrap();
        let back: RationalTime = serde_json::from_str(&s).unwrap();
        assert_eq!(t, back);
    }
}
