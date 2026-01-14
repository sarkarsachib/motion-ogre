#![deny(missing_docs)]
#![forbid(unsafe_code)]

//! Playback primitives for Motion Ogre.
//!
//! This crate does not decode media. It provides time conversion, playhead tracking and
//! composition-order resolution for the timeline model.

use motion_ogre_time::{FramePosition, FrameRate, RationalTime};
use motion_ogre_timeline::{ActiveClip, Layer, Timeline};

/// A playhead that tracks current timeline time and provides frame conversion.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Playhead {
    frame_rate: FrameRate,
    time: RationalTime,
}

impl Playhead {
    /// Creates a new playhead at `t = 0`.
    pub fn new(frame_rate: FrameRate) -> Self {
        Self {
            frame_rate,
            time: RationalTime::ZERO,
        }
    }

    /// Returns the current time.
    pub fn time(&self) -> RationalTime {
        self.time
    }

    /// Returns the playhead's frame rate.
    pub fn frame_rate(&self) -> FrameRate {
        self.frame_rate
    }

    /// Seeks the playhead to a specific time.
    pub fn seek(&mut self, time: RationalTime) {
        self.time = time;
    }

    /// Seeks the playhead to an integer frame.
    pub fn seek_frame(&mut self, frame: i64) {
        self.time = self.frame_rate.frames_to_time(frame);
    }

    /// Steps the playhead by a number of frames.
    pub fn step_frames(&mut self, delta_frames: i64) {
        self.time = self.time + self.frame_rate.frames_to_time(delta_frames);
    }

    /// Returns the current frame position with sub-frame accuracy.
    pub fn frame_position(&self) -> FramePosition {
        self.frame_rate.time_to_frame_position(self.time)
    }

    /// Returns the integer frame index.
    pub fn frame_index(&self) -> i64 {
        self.frame_position().frame
    }

    /// Returns the exact rational frame count for the current time.
    pub fn frames_exact(&self) -> RationalTime {
        self.frame_rate.time_to_frames_exact(self.time)
    }
}

/// A resolved clip instance at a particular timeline time.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ResolvedClip {
    /// The layer z-order.
    pub z_order: i32,
    /// The clip identifier trio.
    pub clip: ActiveClip,
}

/// Resolves active clips at a time, ordered from bottom-to-top (increasing z-order).
pub fn resolve_active_clips(timeline: &Timeline, time: RationalTime) -> Vec<ResolvedClip> {
    let mut out = Vec::new();
    for layer in timeline.layers_in_composition_order() {
        for active in active_clips_in_layer(layer, time) {
            out.push(ResolvedClip {
                z_order: layer.z_order(),
                clip: active,
            });
        }
    }
    out
}

fn active_clips_in_layer(layer: &Layer, time: RationalTime) -> Vec<ActiveClip> {
    let mut out = Vec::new();
    for track in layer.tracks() {
        for clip in track.clips_at_time(time) {
            out.push(ActiveClip {
                layer_id: layer.id(),
                track_id: track.id(),
                clip_id: clip.id(),
            });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use motion_ogre_time::TimeRange;
    use motion_ogre_timeline::{ClipSource, LayerKind};

    fn t(n: i64, d: i64) -> RationalTime {
        RationalTime::new(n, d).unwrap()
    }

    #[test]
    fn playhead_seek_frame_is_exact() {
        let fr = FrameRate::new(24, 1).unwrap();
        let mut ph = Playhead::new(fr);
        ph.seek_frame(12);
        assert_eq!(ph.frame_index(), 12);
        assert_eq!(ph.time(), t(1, 2));
    }

    #[test]
    fn playhead_subframe_precision_is_preserved() {
        let fr = FrameRate::new(24, 1).unwrap();
        let mut ph = Playhead::new(fr);
        ph.seek(t(1, 48));
        let fp = ph.frame_position();
        assert_eq!(fp.frame, 0);
        assert_eq!(fp.subframe, t(1, 2));
        assert!(fp.as_f64() > 0.49 && fp.as_f64() < 0.51);
    }

    #[test]
    fn resolve_active_clips_respects_layer_z_order() {
        let fr = FrameRate::new(24, 1).unwrap();
        let mut tl = Timeline::new(fr);
        let bottom = tl.create_layer(LayerKind::Video, 0, "bottom");
        let top = tl.create_layer(LayerKind::Video, 10, "top");
        let bt = tl.create_track(bottom, "t").unwrap();
        let tt = tl.create_track(top, "t").unwrap();

        let bclip = tl
            .create_clip(
                bottom,
                bt,
                "b",
                TimeRange::new(t(0, 1), t(10, 1)),
                ClipSource::PlaceholderMedia {
                    media_id: "m".into(),
                },
            )
            .unwrap();
        let tclip = tl
            .create_clip(
                top,
                tt,
                "t",
                TimeRange::new(t(0, 1), t(10, 1)),
                ClipSource::PlaceholderMedia {
                    media_id: "m".into(),
                },
            )
            .unwrap();

        let resolved = resolve_active_clips(&tl, t(1, 1));
        assert_eq!(resolved.len(), 2);
        assert_eq!(resolved[0].clip.clip_id, bclip);
        assert_eq!(resolved[1].clip.clip_id, tclip);
    }
}
