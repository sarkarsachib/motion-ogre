#![deny(missing_docs)]
#![forbid(unsafe_code)]

//! Timeline data structures for Motion Ogre.
//!
//! The timeline model is designed to be:
//! - Frame-accurate (all time is rational)
//! - Serializable (via `serde`)
//! - Modular and suitable for nested compositions

use core::fmt;

use motion_ogre_time::{FrameRate, RationalTime, TimeRange};
use serde::{Deserialize, Serialize};

/// An identifier for a timeline layer.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LayerId(u64);

/// An identifier for a track.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TrackId(u64);

/// An identifier for a clip.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ClipId(u64);

/// Errors that can occur when manipulating a [`Timeline`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimelineError {
    /// The layer was not found.
    LayerNotFound(LayerId),
    /// The track was not found.
    TrackNotFound(TrackId),
    /// The clip was not found.
    ClipNotFound(ClipId),
}

impl fmt::Display for TimelineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TimelineError::LayerNotFound(id) => write!(f, "layer not found: {id:?}"),
            TimelineError::TrackNotFound(id) => write!(f, "track not found: {id:?}"),
            TimelineError::ClipNotFound(id) => write!(f, "clip not found: {id:?}"),
        }
    }
}

impl std::error::Error for TimelineError {}

/// The kind of a layer.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LayerKind {
    /// A video layer.
    Video,
    /// An audio layer.
    Audio,
    /// An adjustment layer.
    Adjustment,
    /// A VFX layer.
    Vfx,
}

/// The source content for a clip.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ClipSource {
    /// A placeholder reference to a media asset.
    ///
    /// Media decoding/reading will be implemented later.
    PlaceholderMedia { media_id: String },

    /// A nested composition.
    Composition { timeline: Box<Timeline> },
}

/// A clip on a track.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Clip {
    id: ClipId,
    name: String,
    range: TimeRange,
    enabled: bool,
    source: ClipSource,
}

impl Clip {
    /// Creates a new clip.
    pub fn new(id: ClipId, name: impl Into<String>, range: TimeRange, source: ClipSource) -> Self {
        Self {
            id,
            name: name.into(),
            range,
            enabled: true,
            source,
        }
    }

    /// Returns the clip id.
    pub fn id(&self) -> ClipId {
        self.id
    }

    /// Returns the clip name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the clip's range in timeline time.
    pub fn range(&self) -> TimeRange {
        self.range
    }

    /// Returns true if this clip is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Sets whether this clip is enabled.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Returns the clip source.
    pub fn source(&self) -> &ClipSource {
        &self.source
    }

    /// Returns the nested timeline if this clip is a composition.
    pub fn nested_timeline(&self) -> Option<&Timeline> {
        match &self.source {
            ClipSource::Composition { timeline } => Some(timeline),
            _ => None,
        }
    }

    /// Returns true if the given time is within this clip range.
    pub fn contains_time(&self, time: RationalTime) -> bool {
        self.enabled && self.range.contains(time)
    }
}

/// A track which contains an ordered list of clips.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Track {
    id: TrackId,
    name: String,
    clips: Vec<Clip>,
}

impl Track {
    /// Creates a new empty track.
    pub fn new(id: TrackId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            clips: Vec::new(),
        }
    }

    /// Returns the track id.
    pub fn id(&self) -> TrackId {
        self.id
    }

    /// Returns the track name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns all clips on this track.
    pub fn clips(&self) -> &[Clip] {
        &self.clips
    }

    /// Returns a mutable slice of all clips.
    ///
    /// After mutating clip timing, call [`Track::sort_clips`] to preserve ordering.
    pub fn clips_mut(&mut self) -> &mut [Clip] {
        &mut self.clips
    }

    /// Sorts clips by start time.
    pub fn sort_clips(&mut self) {
        self.clips
            .sort_by(|a, b| a.range().start().cmp(&b.range().start()));
    }

    /// Inserts a clip, maintaining sorted order by start time.
    pub fn insert_clip(&mut self, clip: Clip) {
        let start = clip.range().start();
        let idx = self
            .clips
            .binary_search_by(|c| c.range().start().cmp(&start))
            .unwrap_or_else(|i| i);
        self.clips.insert(idx, clip);
    }

    /// Removes a clip by id.
    pub fn remove_clip(&mut self, clip_id: ClipId) -> Option<Clip> {
        let idx = self.clips.iter().position(|c| c.id() == clip_id)?;
        Some(self.clips.remove(idx))
    }

    /// Returns the clips active at `time`.
    pub fn clips_at_time(&self, time: RationalTime) -> Vec<&Clip> {
        let idx = self
            .clips
            .partition_point(|c| c.range().start() <= time);

        let mut out = Vec::new();
        for clip in self.clips[..idx].iter().rev() {
            if clip.contains_time(time) {
                out.push(clip);
            }
        }

        out
    }

    /// Returns the given clip.
    pub fn clip(&self, clip_id: ClipId) -> Option<&Clip> {
        self.clips.iter().find(|c| c.id() == clip_id)
    }

    /// Returns the given clip mutably.
    pub fn clip_mut(&mut self, clip_id: ClipId) -> Option<&mut Clip> {
        self.clips.iter_mut().find(|c| c.id() == clip_id)
    }
}

/// A layer which contains tracks.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Layer {
    id: LayerId,
    name: String,
    kind: LayerKind,
    z_order: i32,
    tracks: Vec<Track>,
}

impl Layer {
    /// Creates a new layer.
    pub fn new(id: LayerId, kind: LayerKind, z_order: i32, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            kind,
            z_order,
            tracks: Vec::new(),
        }
    }

    /// Returns the layer id.
    pub fn id(&self) -> LayerId {
        self.id
    }

    /// Returns the layer name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the layer kind.
    pub fn kind(&self) -> LayerKind {
        self.kind
    }

    /// Returns the z-order (higher z-order is composited above lower z-order).
    pub fn z_order(&self) -> i32 {
        self.z_order
    }

    /// Sets the z-order.
    pub fn set_z_order(&mut self, z_order: i32) {
        self.z_order = z_order;
    }

    /// Returns the tracks.
    pub fn tracks(&self) -> &[Track] {
        &self.tracks
    }

    /// Returns the tracks mutably.
    pub fn tracks_mut(&mut self) -> &mut [Track] {
        &mut self.tracks
    }

    /// Adds a track.
    pub fn add_track(&mut self, track: Track) {
        self.tracks.push(track);
    }

    /// Returns the track.
    pub fn track(&self, track_id: TrackId) -> Option<&Track> {
        self.tracks.iter().find(|t| t.id() == track_id)
    }

    /// Returns the track mutably.
    pub fn track_mut(&mut self, track_id: TrackId) -> Option<&mut Track> {
        self.tracks.iter_mut().find(|t| t.id() == track_id)
    }
}

/// The root timeline object.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Timeline {
    frame_rate: FrameRate,
    layers: Vec<Layer>,
    next_id: u64,
}

impl Timeline {
    /// Creates a new empty timeline.
    pub fn new(frame_rate: FrameRate) -> Self {
        Self {
            frame_rate,
            layers: Vec::new(),
            next_id: 1,
        }
    }

    /// Returns the timeline frame rate.
    pub fn frame_rate(&self) -> FrameRate {
        self.frame_rate
    }

    /// Returns all layers in insertion order.
    pub fn layers(&self) -> &[Layer] {
        &self.layers
    }

    /// Returns all layers mutably.
    pub fn layers_mut(&mut self) -> &mut [Layer] {
        &mut self.layers
    }

    fn alloc(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Creates a new layer and returns its id.
    pub fn create_layer(
        &mut self,
        kind: LayerKind,
        z_order: i32,
        name: impl Into<String>,
    ) -> LayerId {
        let id = LayerId(self.alloc());
        self.layers.push(Layer::new(id, kind, z_order, name));
        id
    }

    /// Removes a layer.
    pub fn remove_layer(&mut self, layer_id: LayerId) -> Option<Layer> {
        let idx = self.layers.iter().position(|l| l.id() == layer_id)?;
        Some(self.layers.remove(idx))
    }

    /// Returns a layer by id.
    pub fn layer(&self, layer_id: LayerId) -> Option<&Layer> {
        self.layers.iter().find(|l| l.id() == layer_id)
    }

    /// Returns a layer by id mutably.
    pub fn layer_mut(&mut self, layer_id: LayerId) -> Option<&mut Layer> {
        self.layers.iter_mut().find(|l| l.id() == layer_id)
    }

    /// Creates a new track on the specified layer.
    pub fn create_track(&mut self, layer_id: LayerId, name: impl Into<String>) -> Result<TrackId, TimelineError> {
        let track_id = TrackId(self.alloc());
        let layer = self
            .layer_mut(layer_id)
            .ok_or(TimelineError::LayerNotFound(layer_id))?;
        layer.add_track(Track::new(track_id, name));
        Ok(track_id)
    }

    /// Inserts a clip on a specific track.
    pub fn create_clip(
        &mut self,
        layer_id: LayerId,
        track_id: TrackId,
        name: impl Into<String>,
        range: TimeRange,
        source: ClipSource,
    ) -> Result<ClipId, TimelineError> {
        let clip_id = ClipId(self.alloc());
        let layer = self
            .layer_mut(layer_id)
            .ok_or(TimelineError::LayerNotFound(layer_id))?;
        let track = layer
            .track_mut(track_id)
            .ok_or(TimelineError::TrackNotFound(track_id))?;
        track.insert_clip(Clip::new(clip_id, name, range, source));
        Ok(clip_id)
    }

    /// Updates a clip's range and keeps the track sorted.
    pub fn set_clip_range(
        &mut self,
        layer_id: LayerId,
        track_id: TrackId,
        clip_id: ClipId,
        range: TimeRange,
    ) -> Result<(), TimelineError> {
        let layer = self
            .layer_mut(layer_id)
            .ok_or(TimelineError::LayerNotFound(layer_id))?;
        let track = layer
            .track_mut(track_id)
            .ok_or(TimelineError::TrackNotFound(track_id))?;
        let clip = track
            .clip_mut(clip_id)
            .ok_or(TimelineError::ClipNotFound(clip_id))?;
        clip.range = range;
        track.sort_clips();
        Ok(())
    }

    /// Returns layers in composition order.
    ///
    /// Lower `z_order` layers are returned first. If two layers have the same z-order, insertion
    /// order is preserved.
    pub fn layers_in_composition_order(&self) -> Vec<&Layer> {
        let mut layers: Vec<(usize, &Layer)> = self.layers.iter().enumerate().collect();
        layers.sort_by(|(ia, a), (ib, b)| a.z_order().cmp(&b.z_order()).then_with(|| ia.cmp(ib)));
        layers.into_iter().map(|(_, l)| l).collect()
    }

    /// Returns all active clips at the given time in composition order.
    pub fn active_clips_at_time(&self, time: RationalTime) -> Vec<ActiveClip> {
        let mut out = Vec::new();
        for layer in self.layers_in_composition_order() {
            for track in layer.tracks() {
                for clip in track.clips_at_time(time) {
                    out.push(ActiveClip {
                        layer_id: layer.id(),
                        track_id: track.id(),
                        clip_id: clip.id(),
                    });
                }
            }
        }
        out
    }
}

/// A reference to an active clip resolved at a particular time.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ActiveClip {
    /// The layer id.
    pub layer_id: LayerId,
    /// The track id.
    pub track_id: TrackId,
    /// The clip id.
    pub clip_id: ClipId,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(n: i64, d: i64) -> RationalTime {
        RationalTime::new(n, d).unwrap()
    }

    #[test]
    fn composition_order_sorts_by_z_order() {
        let fr = FrameRate::new(24, 1).unwrap();
        let mut tl = Timeline::new(fr);
        let l1 = tl.create_layer(LayerKind::Video, 10, "top");
        let l2 = tl.create_layer(LayerKind::Video, 0, "bottom");
        let l3 = tl.create_layer(LayerKind::Vfx, 5, "mid");

        let ids: Vec<LayerId> = tl
            .layers_in_composition_order()
            .into_iter()
            .map(|l| l.id())
            .collect();
        assert_eq!(ids, vec![l2, l3, l1]);
    }

    #[test]
    fn track_can_insert_and_query_active_clip() {
        let fr = FrameRate::new(24, 1).unwrap();
        let mut tl = Timeline::new(fr);
        let layer_id = tl.create_layer(LayerKind::Video, 0, "video");
        let track_id = tl.create_track(layer_id, "v1").unwrap();

        let clip_id = tl
            .create_clip(
                layer_id,
                track_id,
                "clip",
                TimeRange::new(t(0, 1), t(10, 1)),
                ClipSource::PlaceholderMedia {
                    media_id: "m1".into(),
                },
            )
            .unwrap();

        let active = tl.active_clips_at_time(t(5, 1));
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].clip_id, clip_id);
    }

    #[test]
    fn clip_range_update_re_sorts_track() {
        let fr = FrameRate::new(24, 1).unwrap();
        let mut tl = Timeline::new(fr);
        let layer_id = tl.create_layer(LayerKind::Video, 0, "video");
        let track_id = tl.create_track(layer_id, "v1").unwrap();

        let a = tl
            .create_clip(
                layer_id,
                track_id,
                "a",
                TimeRange::new(t(10, 1), t(1, 1)),
                ClipSource::PlaceholderMedia {
                    media_id: "m".into(),
                },
            )
            .unwrap();
        let b = tl
            .create_clip(
                layer_id,
                track_id,
                "b",
                TimeRange::new(t(0, 1), t(1, 1)),
                ClipSource::PlaceholderMedia {
                    media_id: "m".into(),
                },
            )
            .unwrap();

        // Move `a` earlier than `b`.
        tl.set_clip_range(layer_id, track_id, a, TimeRange::new(t(-1, 1), t(1, 1)))
            .unwrap();

        let layer = tl.layer(layer_id).unwrap();
        let track = layer.track(track_id).unwrap();
        let ids: Vec<ClipId> = track.clips().iter().map(|c| c.id()).collect();
        assert_eq!(ids, vec![a, b]);
    }

    #[test]
    fn nested_composition_is_accessible() {
        let fr = FrameRate::new(24, 1).unwrap();

        let mut nested = Timeline::new(fr);
        let nl = nested.create_layer(LayerKind::Video, 0, "nested");
        let nt = nested.create_track(nl, "t").unwrap();
        nested
            .create_clip(
                nl,
                nt,
                "c",
                TimeRange::new(t(0, 1), t(1, 1)),
                ClipSource::PlaceholderMedia {
                    media_id: "m".into(),
                },
            )
            .unwrap();

        let mut tl = Timeline::new(fr);
        let l = tl.create_layer(LayerKind::Video, 0, "root");
        let tr = tl.create_track(l, "t").unwrap();
        let clip_id = tl
            .create_clip(
                l,
                tr,
                "nested",
                TimeRange::new(t(0, 1), t(10, 1)),
                ClipSource::Composition {
                    timeline: Box::new(nested.clone()),
                },
            )
            .unwrap();

        let root_layer = tl.layer(l).unwrap();
        let root_track = root_layer.track(tr).unwrap();
        let clip = root_track.clip(clip_id).unwrap();
        assert!(clip.nested_timeline().is_some());
        assert_eq!(clip.nested_timeline().unwrap(), &nested);
    }

    #[test]
    fn timeline_handles_many_clips_and_query_is_correct() {
        let fr = FrameRate::new(24, 1).unwrap();
        let mut tl = Timeline::new(fr);
        let layer_id = tl.create_layer(LayerKind::Video, 0, "video");
        let track_id = tl.create_track(layer_id, "v1").unwrap();

        for i in 0..150 {
            let start = t(i, 1);
            tl.create_clip(
                layer_id,
                track_id,
                format!("c{i}"),
                TimeRange::new(start, t(1, 1)),
                ClipSource::PlaceholderMedia {
                    media_id: "m".into(),
                },
            )
            .unwrap();
        }

        assert_eq!(tl.active_clips_at_time(t(42, 1)).len(), 1);
        assert_eq!(tl.active_clips_at_time(t(149, 1)).len(), 1);
        assert_eq!(tl.active_clips_at_time(t(151, 1)).len(), 0);
    }

    #[test]
    fn serde_round_trip_timeline() {
        let fr = FrameRate::new(24, 1).unwrap();
        let mut tl = Timeline::new(fr);
        let layer_id = tl.create_layer(LayerKind::Audio, 0, "audio");
        let track_id = tl.create_track(layer_id, "a1").unwrap();
        tl.create_clip(
            layer_id,
            track_id,
            "c",
            TimeRange::new(t(0, 1), t(5, 1)),
            ClipSource::PlaceholderMedia {
                media_id: "m".into(),
            },
        )
        .unwrap();

        let json = serde_json::to_string(&tl).unwrap();
        let back: Timeline = serde_json::from_str(&json).unwrap();
        assert_eq!(tl, back);
    }
}
