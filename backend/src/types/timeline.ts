/**
 * Timeline data structures matching Rust crate definitions.
 */

export type LayerKind = "Video" | "Audio" | "Adjustment" | "Vfx";

export interface RationalTime {
  numerator: number;
  denominator: number;
}

export interface TimeRange {
  start: RationalTime;
  duration: RationalTime;
}

export type ClipSource =
  | { PlaceholderMedia: { media_id: string } }
  | { Composition: { timeline: Timeline } };

export interface Clip {
  id: { 0: number };
  name: string;
  range: TimeRange;
  enabled: boolean;
  source: ClipSource;
}

export interface Track {
  id: { 0: number };
  name: string;
  clips: Clip[];
}

export interface Layer {
  id: { 0: number };
  name: string;
  kind: LayerKind;
  z_order: number;
  tracks: Track[];
}

export interface Timeline {
  frame_rate: { fps: RationalTime };
  layers: Layer[];
  next_id: number;
}

export interface ActiveClip {
  layer_id: { 0: number };
  track_id: { 0: number };
  clip_id: { 0: number };
}

/**
 * Timeline mutations
 */
export interface CreateLayerCommand {
  kind: LayerKind;
  z_order: number;
  name: string;
}

export interface CreateTrackCommand {
  layer_id: { 0: number };
  name: string;
}

export interface CreateClipCommand {
  layer_id: { 0: number };
  track_id: { 0: number };
  name: string;
  range: TimeRange;
  source: ClipSource;
}

export interface UpdateClipCommand {
  layer_id: { 0: number };
  track_id: { 0: number };
  clip_id: { 0: number };
  range: TimeRange;
}

export interface SetClipEnabledCommand {
  layer_id: { 0: number };
  track_id: { 0: number };
  clip_id: { 0: number };
  enabled: boolean;
}
