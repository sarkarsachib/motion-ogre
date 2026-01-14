#![deny(missing_docs)]
#![forbid(unsafe_code)]

//! Motion Ogre editing engine foundations.
//!
//! This crate is a convenience umbrella that re-exports the core building blocks:
//! - [`motion_ogre_time`] for rational time and frame rate
//! - [`motion_ogre_timeline`] for timeline structures
//! - [`motion_ogre_animation`] for keyframe animation
//! - [`motion_ogre_playback`] for playhead and composition resolution

/// Frame-accurate time primitives.
pub mod time {
    pub use motion_ogre_time::*;
}

/// Timeline model types.
pub mod timeline {
    pub use motion_ogre_timeline::*;
}

/// Keyframe animation primitives.
pub mod animation {
    pub use motion_ogre_animation::*;
}

/// Playback primitives.
pub mod playback {
    pub use motion_ogre_playback::*;
}
