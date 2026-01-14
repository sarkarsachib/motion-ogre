#![deny(missing_docs)]
#![forbid(unsafe_code)]

//! Keyframe animation primitives for Motion Ogre.
//!
//! This crate provides keyframe tracks that can be attached to clip/layer properties.
//! Sampling is deterministic and uses [`motion_ogre_time::RationalTime`] for time.

use core::fmt;
use std::collections::BTreeMap;

use motion_ogre_time::RationalTime;
use serde::{Deserialize, Serialize};

/// A 2D vector.
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct Vec2 {
    /// X component.
    pub x: f32,
    /// Y component.
    pub y: f32,
}

impl Vec2 {
    /// Linearly interpolates between two vectors.
    pub fn lerp(a: Self, b: Self, t: f32) -> Self {
        Self {
            x: a.x + (b.x - a.x) * t,
            y: a.y + (b.y - a.y) * t,
        }
    }
}

/// A value that can be animated.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PropertyValue {
    /// A floating point scalar.
    Float(f32),
    /// A 2D vector.
    Vec2(Vec2),
}

impl PropertyValue {
    /// Linearly interpolates between two values.
    ///
    /// Returns `None` if the two values are not of the same variant.
    pub fn lerp(a: &Self, b: &Self, t: f32) -> Option<Self> {
        let t = t.clamp(0.0, 1.0);
        match (a, b) {
            (PropertyValue::Float(a), PropertyValue::Float(b)) => {
                Some(PropertyValue::Float(a + (b - a) * t))
            }
            (PropertyValue::Vec2(a), PropertyValue::Vec2(b)) => {
                Some(PropertyValue::Vec2(Vec2::lerp(*a, *b, t)))
            }
            _ => None,
        }
    }
}

/// An identifier for an animatable property.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PropertyId(String);

impl PropertyId {
    /// Creates a new property id.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the property id as a string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PropertyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<&str> for PropertyId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

/// A cubic bezier easing curve.
///
/// This matches the behavior of CSS timing functions:
/// - `t` is treated as the x value
/// - the output is the y value
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct CubicBezier {
    /// Control point 1 x.
    pub x1: f32,
    /// Control point 1 y.
    pub y1: f32,
    /// Control point 2 x.
    pub x2: f32,
    /// Control point 2 y.
    pub y2: f32,
}

impl CubicBezier {
    /// Creates a cubic bezier curve.
    pub fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self { x1, y1, x2, y2 }
    }

    fn sample_curve(a1: f32, a2: f32, t: f32) -> f32 {
        // (1-t)^3*0 + 3(1-t)^2 t*a1 + 3(1-t)t^2*a2 + t^3*1
        let inv = 1.0 - t;
        3.0 * inv * inv * t * a1 + 3.0 * inv * t * t * a2 + t * t * t
    }

    fn sample_curve_derivative(a1: f32, a2: f32, t: f32) -> f32 {
        // derivative of sample_curve for x(t) or y(t)
        let inv = 1.0 - t;
        3.0 * inv * inv * a1 + 6.0 * inv * t * (a2 - a1) + 3.0 * t * t * (1.0 - a2)
    }

    /// Applies the easing curve.
    pub fn apply(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);

        // Invert x(u) = t to find u, then evaluate y(u).
        let mut u = t;
        for _ in 0..8 {
            let x = Self::sample_curve(self.x1, self.x2, u) - t;
            let dx = Self::sample_curve_derivative(self.x1, self.x2, u);
            if dx.abs() < 1e-6 {
                break;
            }
            let next = u - x / dx;
            if (next - u).abs() < 1e-6 {
                u = next;
                break;
            }
            u = next.clamp(0.0, 1.0);
        }

        // Fallback to bisection if needed.
        let mut lo = 0.0;
        let mut hi = 1.0;
        for _ in 0..20 {
            let x = Self::sample_curve(self.x1, self.x2, u);
            if (x - t).abs() < 1e-6 {
                break;
            }
            if x < t {
                lo = u;
            } else {
                hi = u;
            }
            u = (lo + hi) * 0.5;
        }

        Self::sample_curve(self.y1, self.y2, u).clamp(0.0, 1.0)
    }
}

/// The interpolation method between two keyframes.
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum KeyframeInterpolation {
    /// Linear interpolation.
    Linear,
    /// Ease-in interpolation (accelerating).
    EaseIn,
    /// Ease-out interpolation (decelerating).
    EaseOut,
    /// A custom cubic bezier easing curve.
    Bezier(CubicBezier),
}

impl KeyframeInterpolation {
    /// Applies interpolation to a linear `t` in `[0, 1]`.
    pub fn apply(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            KeyframeInterpolation::Linear => t,
            KeyframeInterpolation::EaseIn => t * t,
            KeyframeInterpolation::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
            KeyframeInterpolation::Bezier(b) => b.apply(t),
        }
    }
}

/// A single keyframe on a track.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Keyframe {
    time: RationalTime,
    value: PropertyValue,
    interpolation: KeyframeInterpolation,
}

impl Keyframe {
    /// Creates a new keyframe.
    pub fn new(time: RationalTime, value: PropertyValue, interpolation: KeyframeInterpolation) -> Self {
        Self {
            time,
            value,
            interpolation,
        }
    }

    /// Returns the keyframe time.
    pub fn time(&self) -> RationalTime {
        self.time
    }

    /// Returns the keyframe value.
    pub fn value(&self) -> &PropertyValue {
        &self.value
    }

    /// Returns the interpolation used from this keyframe to the next.
    pub fn interpolation(&self) -> KeyframeInterpolation {
        self.interpolation
    }
}

/// A keyframe track for a single property.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PropertyTrack {
    keyframes: Vec<Keyframe>,
}

impl PropertyTrack {
    /// Creates an empty track.
    pub fn new() -> Self {
        Self {
            keyframes: Vec::new(),
        }
    }

    /// Returns all keyframes.
    pub fn keyframes(&self) -> &[Keyframe] {
        &self.keyframes
    }

    /// Adds a keyframe.
    ///
    /// If a keyframe already exists at the same time, it is replaced.
    pub fn add_keyframe(&mut self, keyframe: Keyframe) {
        match self
            .keyframes
            .binary_search_by(|k| k.time().cmp(&keyframe.time()))
        {
            Ok(idx) => self.keyframes[idx] = keyframe,
            Err(idx) => self.keyframes.insert(idx, keyframe),
        }
    }

    /// Removes a keyframe at an exact time.
    pub fn remove_keyframe_at(&mut self, time: RationalTime) -> Option<Keyframe> {
        let idx = self.keyframes.binary_search_by(|k| k.time().cmp(&time)).ok()?;
        Some(self.keyframes.remove(idx))
    }

    /// Samples the track at a specific time.
    pub fn sample(&self, time: RationalTime) -> Option<PropertyValue> {
        let len = self.keyframes.len();
        if len == 0 {
            return None;
        }

        match self
            .keyframes
            .binary_search_by(|k| k.time().cmp(&time))
        {
            Ok(i) => Some(self.keyframes[i].value.clone()),
            Err(0) => Some(self.keyframes[0].value.clone()),
            Err(i) if i >= len => Some(self.keyframes[len - 1].value.clone()),
            Err(i) => {
                let left = &self.keyframes[i - 1];
                let right = &self.keyframes[i];
                let span = (right.time - left.time).as_f64();
                if span == 0.0 {
                    return Some(left.value.clone());
                }
                let t = ((time - left.time).as_f64() / span) as f32;
                let eased = left.interpolation.apply(t);
                PropertyValue::lerp(&left.value, &right.value, eased)
            }
        }
    }
}

impl Default for PropertyTrack {
    fn default() -> Self {
        Self::new()
    }
}

/// A collection of property tracks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PropertyAnimations {
    tracks: BTreeMap<PropertyId, PropertyTrack>,
}

impl PropertyAnimations {
    /// Creates an empty collection.
    pub fn new() -> Self {
        Self {
            tracks: BTreeMap::new(),
        }
    }

    /// Returns all tracks.
    pub fn tracks(&self) -> &BTreeMap<PropertyId, PropertyTrack> {
        &self.tracks
    }

    /// Returns a track for a property.
    pub fn track(&self, property: &PropertyId) -> Option<&PropertyTrack> {
        self.tracks.get(property)
    }

    /// Returns a track for a property mutably.
    pub fn track_mut(&mut self, property: PropertyId) -> &mut PropertyTrack {
        self.tracks.entry(property).or_default()
    }

    /// Samples a property at the given time.
    pub fn sample(&self, property: &PropertyId, time: RationalTime) -> Option<PropertyValue> {
        self.track(property)?.sample(time)
    }
}

impl Default for PropertyAnimations {
    fn default() -> Self {
        Self::new()
    }
}

/// Common property identifiers used by the engine.
///
/// These are helpers; custom properties can be created with [`PropertyId`].
pub mod properties {
    use super::PropertyId;

    /// 2D position in pixels.
    pub fn position() -> PropertyId {
        PropertyId::new("transform.position")
    }

    /// 2D scale multiplier (1.0 is identity).
    pub fn scale() -> PropertyId {
        PropertyId::new("transform.scale")
    }

    /// Rotation in degrees.
    pub fn rotation() -> PropertyId {
        PropertyId::new("transform.rotation")
    }

    /// Opacity in the range `[0, 1]`.
    pub fn opacity() -> PropertyId {
        PropertyId::new("render.opacity")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    fn t(n: i64, d: i64) -> RationalTime {
        RationalTime::new(n, d).unwrap()
    }

    #[test]
    fn inserts_keyframes_in_sorted_order() {
        let mut track = PropertyTrack::new();
        track.add_keyframe(Keyframe::new(
            t(10, 1),
            PropertyValue::Float(1.0),
            KeyframeInterpolation::Linear,
        ));
        track.add_keyframe(Keyframe::new(
            t(0, 1),
            PropertyValue::Float(0.0),
            KeyframeInterpolation::Linear,
        ));

        assert_eq!(track.keyframes()[0].time(), t(0, 1));
        assert_eq!(track.keyframes()[1].time(), t(10, 1));
    }

    #[test]
    fn sample_clamps_to_endpoints() {
        let mut track = PropertyTrack::new();
        track.add_keyframe(Keyframe::new(
            t(0, 1),
            PropertyValue::Float(0.0),
            KeyframeInterpolation::Linear,
        ));
        track.add_keyframe(Keyframe::new(
            t(1, 1),
            PropertyValue::Float(1.0),
            KeyframeInterpolation::Linear,
        ));

        assert_eq!(track.sample(t(-1, 1)), Some(PropertyValue::Float(0.0)));
        assert_eq!(track.sample(t(2, 1)), Some(PropertyValue::Float(1.0)));
    }

    #[test]
    fn sample_exact_keyframe_returns_exact_value() {
        let mut track = PropertyTrack::new();
        track.add_keyframe(Keyframe::new(
            t(0, 1),
            PropertyValue::Float(0.0),
            KeyframeInterpolation::Linear,
        ));
        assert_eq!(track.sample(t(0, 1)), Some(PropertyValue::Float(0.0)));
    }

    #[test]
    fn linear_interpolation_float_is_precise() {
        let mut track = PropertyTrack::new();
        track.add_keyframe(Keyframe::new(
            t(0, 1),
            PropertyValue::Float(0.0),
            KeyframeInterpolation::Linear,
        ));
        track.add_keyframe(Keyframe::new(
            t(10, 1),
            PropertyValue::Float(10.0),
            KeyframeInterpolation::Linear,
        ));

        match track.sample(t(1, 2)).unwrap() {
            PropertyValue::Float(v) => assert_relative_eq!(v, 0.5, epsilon = 1e-6),
            _ => panic!("wrong type"),
        }
    }

    #[test]
    fn ease_in_is_quadratic() {
        let mut track = PropertyTrack::new();
        track.add_keyframe(Keyframe::new(
            t(0, 1),
            PropertyValue::Float(0.0),
            KeyframeInterpolation::EaseIn,
        ));
        track.add_keyframe(Keyframe::new(
            t(1, 1),
            PropertyValue::Float(1.0),
            KeyframeInterpolation::Linear,
        ));

        match track.sample(t(1, 2)).unwrap() {
            PropertyValue::Float(v) => assert_relative_eq!(v, 0.25, epsilon = 1e-6),
            _ => panic!("wrong type"),
        }
    }

    #[test]
    fn ease_out_is_quadratic() {
        let mut track = PropertyTrack::new();
        track.add_keyframe(Keyframe::new(
            t(0, 1),
            PropertyValue::Float(0.0),
            KeyframeInterpolation::EaseOut,
        ));
        track.add_keyframe(Keyframe::new(
            t(1, 1),
            PropertyValue::Float(1.0),
            KeyframeInterpolation::Linear,
        ));

        match track.sample(t(1, 2)).unwrap() {
            PropertyValue::Float(v) => assert_relative_eq!(v, 0.75, epsilon = 1e-6),
            _ => panic!("wrong type"),
        }
    }

    #[test]
    fn bezier_curve_behaves_like_css_timing_function() {
        // This is CSS's ease curve.
        let bez = CubicBezier::new(0.25, 0.1, 0.25, 1.0);
        let y = bez.apply(0.5);
        assert!(y > 0.5);
        assert!(y < 1.0);
    }

    #[test]
    fn vec2_interpolation_works() {
        let mut track = PropertyTrack::new();
        track.add_keyframe(Keyframe::new(
            t(0, 1),
            PropertyValue::Vec2(Vec2 { x: 0.0, y: 0.0 }),
            KeyframeInterpolation::Linear,
        ));
        track.add_keyframe(Keyframe::new(
            t(1, 1),
            PropertyValue::Vec2(Vec2 { x: 10.0, y: 20.0 }),
            KeyframeInterpolation::Linear,
        ));

        match track.sample(t(1, 2)).unwrap() {
            PropertyValue::Vec2(v) => {
                assert_relative_eq!(v.x, 5.0, epsilon = 1e-6);
                assert_relative_eq!(v.y, 10.0, epsilon = 1e-6);
            }
            _ => panic!("wrong type"),
        }
    }

    #[test]
    fn property_animations_samples_named_track() {
        let mut anims = PropertyAnimations::new();
        let prop = properties::opacity();
        anims.track_mut(prop.clone()).add_keyframe(Keyframe::new(
            t(0, 1),
            PropertyValue::Float(0.0),
            KeyframeInterpolation::Linear,
        ));
        anims.track_mut(prop.clone()).add_keyframe(Keyframe::new(
            t(1, 1),
            PropertyValue::Float(1.0),
            KeyframeInterpolation::Linear,
        ));

        match anims.sample(&prop, t(1, 2)).unwrap() {
            PropertyValue::Float(v) => assert_relative_eq!(v, 0.5, epsilon = 1e-6),
            _ => panic!("wrong type"),
        }
    }
}
