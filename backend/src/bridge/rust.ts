/**
 * Rust FFI bridge using napi-rs.
 * 
 * This module provides a bridge to the Rust timeline engine.
 * For now, this is a placeholder implementation that will be
 * replaced with actual FFI bindings once the napi-rs crate is set up.
 */

import { Timeline, LayerKind, TimeRange, RationalTime, ClipSource, ActiveClip } from "../types/timeline";

// Placeholder types for FFI bindings
// TODO: Replace with actual napi-rs bindings

class RustEngine {
  private timelines: Map<string, Timeline>;

  constructor() {
    this.timelines = new Map();
  }

  createTimeline(frameRate: { numerator: number; denominator: number }): string {
    const id = `timeline_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
    const timeline: Timeline = {
      frame_rate: { fps: frameRate },
      layers: [],
      next_id: 1,
    };
    this.timelines.set(id, timeline);
    return id;
  }

  getTimeline(id: string): Timeline | null {
    return this.timelines.get(id) || null;
  }

  createLayer(
    timelineId: string,
    kind: LayerKind,
    zOrder: number,
    name: string
  ): { 0: number } | null {
    const timeline = this.timelines.get(timelineId);
    if (!timeline) return null;

    const layerId = timeline.next_id;
    timeline.next_id += 1;

    timeline.layers.push({
      id: { 0: layerId },
      name,
      kind,
      z_order: zOrder,
      tracks: [],
    });

    return { 0: layerId };
  }

  removeLayer(timelineId: string, layerId: { 0: number }): boolean {
    const timeline = this.timelines.get(timelineId);
    if (!timeline) return false;

    const index = timeline.layers.findIndex((l) => l.id[0] === layerId[0]);
    if (index === -1) return false;

    timeline.layers.splice(index, 1);
    return true;
  }

  createTrack(timelineId: string, layerId: { 0: number }, name: string): { 0: number } | null {
    const timeline = this.timelines.get(timelineId);
    if (!timeline) return null;

    const layer = timeline.layers.find((l) => l.id[0] === layerId[0]);
    if (!layer) return null;

    const trackId = timeline.next_id;
    timeline.next_id += 1;

    layer.tracks.push({
      id: { 0: trackId },
      name,
      clips: [],
    });

    return { 0: trackId };
  }

  createClip(
    timelineId: string,
    layerId: { 0: number },
    trackId: { 0: number },
    name: string,
    range: TimeRange,
    source: ClipSource
  ): { 0: number } | null {
    const timeline = this.timelines.get(timelineId);
    if (!timeline) return null;

    const layer = timeline.layers.find((l) => l.id[0] === layerId[0]);
    if (!layer) return null;

    const track = layer.tracks.find((t) => t.id[0] === trackId[0]);
    if (!track) return null;

    const clipId = timeline.next_id;
    timeline.next_id += 1;

    const clip = {
      id: { 0: clipId },
      name,
      range,
      enabled: true,
      source,
    };

    // Insert clip in sorted order by start time
    const startTime = range.start.numerator / range.start.denominator;
    let insertIndex = track.clips.findIndex(
      (c) => c.range.start.numerator / c.range.start.denominator > startTime
    );
    if (insertIndex === -1) {
      insertIndex = track.clips.length;
    }

    track.clips.splice(insertIndex, 0, clip);

    return { 0: clipId };
  }

  updateClipRange(
    timelineId: string,
    layerId: { 0: number },
    trackId: { 0: number },
    clipId: { 0: number },
    range: TimeRange
  ): boolean {
    const timeline = this.timelines.get(timelineId);
    if (!timeline) return false;

    const layer = timeline.layers.find((l) => l.id[0] === layerId[0]);
    if (!layer) return false;

    const track = layer.tracks.find((t) => t.id[0] === trackId[0]);
    if (!track) return false;

    const clip = track.clips.find((c) => c.id[0] === clipId[0]);
    if (!clip) return false;

    clip.range = range;

    // Re-sort clips by start time
    track.clips.sort(
      (a, b) =>
        a.range.start.numerator / a.range.start.denominator -
        b.range.start.numerator / b.range.start.denominator
    );

    return true;
  }

  setClipEnabled(
    timelineId: string,
    layerId: { 0: number },
    trackId: { 0: number },
    clipId: { 0: number },
    enabled: boolean
  ): boolean {
    const timeline = this.timelines.get(timelineId);
    if (!timeline) return false;

    const layer = timeline.layers.find((l) => l.id[0] === layerId[0]);
    if (!layer) return null;

    const track = layer.tracks.find((t) => t.id[0] === trackId[0]);
    if (!track) return null;

    const clip = track.clips.find((c) => c.id[0] === clipId[0]);
    if (!clip) return null;

    clip.enabled = enabled;
    return true;
  }

  removeClip(
    timelineId: string,
    layerId: { 0: number },
    trackId: { 0: number },
    clipId: { 0: number }
  ): boolean {
    const timeline = this.timelines.get(timelineId);
    if (!timeline) return false;

    const layer = timeline.layers.find((l) => l.id[0] === layerId[0]);
    if (!layer) return false;

    const track = layer.tracks.find((t) => t.id[0] === trackId[0]);
    if (!track) return false;

    const index = track.clips.findIndex((c) => c.id[0] === clipId[0]);
    if (index === -1) return false;

    track.clips.splice(index, 1);
    return true;
  }

  getActiveClipsAtTime(timelineId: string, time: RationalTime): ActiveClip[] {
    const timeline = this.timelines.get(timelineId);
    if (!timeline) return [];

    const activeClips: ActiveClip[] = [];
    const timeValue = time.numerator / time.denominator;

    // Sort layers by z_order for composition order
    const sortedLayers = [...timeline.layers].sort((a, b) => {
      if (a.z_order !== b.z_order) return a.z_order - b.z_order;
      return 0;
    });

    for (const layer of sortedLayers) {
      for (const track of layer.tracks) {
        for (const clip of track.clips) {
          if (!clip.enabled) continue;

          const clipStart = clip.range.start.numerator / clip.range.start.denominator;
          const clipEnd = clipStart + clip.range.duration.numerator / clip.range.duration.denominator;

          if (timeValue >= clipStart && timeValue < clipEnd) {
            activeClips.push({
              layer_id: layer.id,
              track_id: track.id,
              clip_id: clip.id,
            });
          }
        }
      }
    }

    return activeClips;
  }

  serializeTimeline(timelineId: string): string | null {
    const timeline = this.timelines.get(timelineId);
    if (!timeline) return null;
    return JSON.stringify(timeline);
  }

  deserializeTimeline(json: string): string | null {
    try {
      const timeline: Timeline = JSON.parse(json);
      const id = `timeline_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
      this.timelines.set(id, timeline);
      return id;
    } catch (e) {
      return null;
    }
  }

  deleteTimeline(timelineId: string): boolean {
    return this.timelines.delete(timelineId);
  }
}

// Singleton instance
let rustEngineInstance: RustEngine | null = null;

export function getRustEngine(): RustEngine {
  if (!rustEngineInstance) {
    rustEngineInstance = new RustEngine();
  }
  return rustEngineInstance;
}

export function resetRustEngine(): void {
  rustEngineInstance = null;
}
