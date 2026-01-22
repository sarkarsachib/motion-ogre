/**
 * Integration tests for Rust engine bridge.
 */

import { getRustEngine, resetRustEngine } from "../../src/bridge/rust";

describe("Rust Engine Bridge Integration Tests", () => {
  beforeEach(() => {
    resetRustEngine();
  });

  describe("Timeline Management", () => {
    it("should create a new timeline", () => {
      const engine = getRustEngine();
      const timelineId = engine.createTimeline({ numerator: 24, denominator: 1 });

      expect(timelineId).toBeDefined();
      expect(timelineId).toMatch(/^timeline_/);

      const timeline = engine.getTimeline(timelineId);
      expect(timeline).not.toBeNull();
      expect(timeline?.frame_rate.fps).toEqual({ numerator: 24, denominator: 1 });
      expect(timeline?.layers).toEqual([]);
    });

    it("should create timeline with different frame rates", () => {
      const engine = getRustEngine();

      const fps24 = engine.createTimeline({ numerator: 24, denominator: 1 });
      const fps30 = engine.createTimeline({ numerator: 30, denominator: 1 });
      const fps2997 = engine.createTimeline({ numerator: 30000, denominator: 1001 });

      const timeline24 = engine.getTimeline(fps24);
      const timeline30 = engine.getTimeline(fps30);
      const timeline2997 = engine.getTimeline(fps2997);

      expect(timeline24?.frame_rate.fps.numerator).toBe(24);
      expect(timeline30?.frame_rate.fps.numerator).toBe(30);
      expect(timeline2997?.frame_rate.fps.numerator).toBe(30000);
      expect(timeline2997?.frame_rate.fps.denominator).toBe(1001);
    });

    it("should delete a timeline", () => {
      const engine = getRustEngine();
      const timelineId = engine.createTimeline({ numerator: 24, denominator: 1 });

      expect(engine.getTimeline(timelineId)).not.toBeNull();
      expect(engine.deleteTimeline(timelineId)).toBe(true);
      expect(engine.getTimeline(timelineId)).toBeNull();
    });

    it("should return null for non-existent timeline", () => {
      const engine = getRustEngine();
      expect(engine.getTimeline("non-existent")).toBeNull();
    });
  });

  describe("Layer Management", () => {
    it("should create a layer", () => {
      const engine = getRustEngine();
      const timelineId = engine.createTimeline({ numerator: 24, denominator: 1 });

      const layerId = engine.createLayer(timelineId, "Video", 0, "Video Layer 1");

      expect(layerId).toEqual({ 0: 1 });

      const timeline = engine.getTimeline(timelineId);
      expect(timeline?.layers).toHaveLength(1);
      expect(timeline?.layers[0].kind).toBe("Video");
      expect(timeline?.layers[0].name).toBe("Video Layer 1");
      expect(timeline?.layers[0].z_order).toBe(0);
    });

    it("should create layers of different kinds", () => {
      const engine = getRustEngine();
      const timelineId = engine.createTimeline({ numerator: 24, denominator: 1 });

      engine.createLayer(timelineId, "Video", 0, "Video Layer");
      engine.createLayer(timelineId, "Audio", 1, "Audio Layer");
      engine.createLayer(timelineId, "Vfx", 2, "VFX Layer");
      engine.createLayer(timelineId, "Adjustment", 3, "Adjustment Layer");

      const timeline = engine.getTimeline(timelineId);
      expect(timeline?.layers).toHaveLength(4);

      const kinds = timeline?.layers.map((l) => l.kind);
      expect(kinds).toEqual(["Video", "Audio", "Vfx", "Adjustment"]);
    });

    it("should remove a layer", () => {
      const engine = getRustEngine();
      const timelineId = engine.createTimeline({ numerator: 24, denominator: 1 });

      const layerId = engine.createLayer(timelineId, "Video", 0, "Video Layer");
      expect(engine.getTimeline(timelineId)?.layers).toHaveLength(1);

      const result = engine.removeLayer(timelineId, layerId);
      expect(result).toBe(true);
      expect(engine.getTimeline(timelineId)?.layers).toHaveLength(0);
    });

    it("should return null when creating layer on non-existent timeline", () => {
      const engine = getRustEngine();
      const layerId = engine.createLayer("non-existent", "Video", 0, "Layer");

      expect(layerId).toBeNull();
    });

    it("should return false when removing layer from non-existent timeline", () => {
      const engine = getRustEngine();
      const result = engine.removeLayer("non-existent", { 0: 1 });

      expect(result).toBe(false);
    });
  });

  describe("Track Management", () => {
    it("should create a track", () => {
      const engine = getRustEngine();
      const timelineId = engine.createTimeline({ numerator: 24, denominator: 1 });
      const layerId = engine.createLayer(timelineId, "Video", 0, "Video Layer");

      const trackId = engine.createTrack(timelineId, layerId, "Track 1");

      expect(trackId).toEqual({ 0: 2 });

      const timeline = engine.getTimeline(timelineId);
      expect(timeline?.layers[0].tracks).toHaveLength(1);
      expect(timeline?.layers[0].tracks[0].name).toBe("Track 1");
    });

    it("should create multiple tracks on a layer", () => {
      const engine = getRustEngine();
      const timelineId = engine.createTimeline({ numerator: 24, denominator: 1 });
      const layerId = engine.createLayer(timelineId, "Video", 0, "Video Layer");

      engine.createTrack(timelineId, layerId, "Track 1");
      engine.createTrack(timelineId, layerId, "Track 2");
      engine.createTrack(timelineId, layerId, "Track 3");

      const timeline = engine.getTimeline(timelineId);
      expect(timeline?.layers[0].tracks).toHaveLength(3);
    });

    it("should return null when creating track on non-existent layer", () => {
      const engine = getRustEngine();
      const timelineId = engine.createTimeline({ numerator: 24, denominator: 1 });

      const trackId = engine.createTrack(timelineId, { 0: 999 }, "Track 1");
      expect(trackId).toBeNull();
    });
  });

  describe("Clip Management", () => {
    it("should create a clip", () => {
      const engine = getRustEngine();
      const timelineId = engine.createTimeline({ numerator: 24, denominator: 1 });
      const layerId = engine.createLayer(timelineId, "Video", 0, "Video Layer");
      const trackId = engine.createTrack(timelineId, layerId, "Track 1");

      const clipId = engine.createClip(
        timelineId,
        layerId,
        trackId,
        "Clip 1",
        {
          start: { numerator: 0, denominator: 1 },
          duration: { numerator: 100, denominator: 1 },
        },
        { PlaceholderMedia: { media_id: "media_123" } }
      );

      expect(clipId).toEqual({ 0: 3 });

      const timeline = engine.getTimeline(timelineId);
      const track = timeline?.layers[0].tracks[0];
      expect(track?.clips).toHaveLength(1);
      expect(track?.clips[0].name).toBe("Clip 1");
      expect(track?.clips[0].enabled).toBe(true);
    });

    it("should sort clips by start time", () => {
      const engine = getRustEngine();
      const timelineId = engine.createTimeline({ numerator: 24, denominator: 1 });
      const layerId = engine.createLayer(timelineId, "Video", 0, "Video Layer");
      const trackId = engine.createTrack(timelineId, layerId, "Track 1");

      engine.createClip(
        timelineId,
        layerId,
        trackId,
        "Clip 2",
        {
          start: { numerator: 100, denominator: 1 },
          duration: { numerator: 50, denominator: 1 },
        },
        { PlaceholderMedia: { media_id: "media_2" } }
      );

      engine.createClip(
        timelineId,
        layerId,
        trackId,
        "Clip 1",
        {
          start: { numerator: 0, denominator: 1 },
          duration: { numerator: 50, denominator: 1 },
        },
        { PlaceholderMedia: { media_id: "media_1" } }
      );

      engine.createClip(
        timelineId,
        layerId,
        trackId,
        "Clip 3",
        {
          start: { numerator: 50, denominator: 1 },
          duration: { numerator: 50, denominator: 1 },
        },
        { PlaceholderMedia: { media_id: "media_3" } }
      );

      const timeline = engine.getTimeline(timelineId);
      const track = timeline?.layers[0].tracks[0];
      expect(track?.clips[0].name).toBe("Clip 1");
      expect(track?.clips[1].name).toBe("Clip 2");
      expect(track?.clips[2].name).toBe("Clip 3");
    });

    it("should update clip range", () => {
      const engine = getRustEngine();
      const timelineId = engine.createTimeline({ numerator: 24, denominator: 1 });
      const layerId = engine.createLayer(timelineId, "Video", 0, "Video Layer");
      const trackId = engine.createTrack(timelineId, layerId, "Track 1");

      const clipId = engine.createClip(
        timelineId,
        layerId,
        trackId,
        "Clip 1",
        {
          start: { numerator: 0, denominator: 1 },
          duration: { numerator: 100, denominator: 1 },
        },
        { PlaceholderMedia: { media_id: "media_123" } }
      );

      const result = engine.updateClipRange(
        timelineId,
        layerId,
        trackId,
        clipId,
        {
          start: { numerator: 50, denominator: 1 },
          duration: { numerator: 100, denominator: 1 },
        }
      );

      expect(result).toBe(true);

      const timeline = engine.getTimeline(timelineId);
      const clip = timeline?.layers[0].tracks[0].clips[0];
      expect(clip?.range.start.numerator).toBe(50);
      expect(clip?.range.duration.numerator).toBe(100);
    });

    it("should set clip enabled state", () => {
      const engine = getRustEngine();
      const timelineId = engine.createTimeline({ numerator: 24, denominator: 1 });
      const layerId = engine.createLayer(timelineId, "Video", 0, "Video Layer");
      const trackId = engine.createTrack(timelineId, layerId, "Track 1");

      const clipId = engine.createClip(
        timelineId,
        layerId,
        trackId,
        "Clip 1",
        {
          start: { numerator: 0, denominator: 1 },
          duration: { numerator: 100, denominator: 1 },
        },
        { PlaceholderMedia: { media_id: "media_123" } }
      );

      const result = engine.setClipEnabled(timelineId, layerId, trackId, clipId, false);

      expect(result).toBe(true);

      const timeline = engine.getTimeline(timelineId);
      const clip = timeline?.layers[0].tracks[0].clips[0];
      expect(clip?.enabled).toBe(false);
    });

    it("should remove a clip", () => {
      const engine = getRustEngine();
      const timelineId = engine.createTimeline({ numerator: 24, denominator: 1 });
      const layerId = engine.createLayer(timelineId, "Video", 0, "Video Layer");
      const trackId = engine.createTrack(timelineId, layerId, "Track 1");

      const clipId = engine.createClip(
        timelineId,
        layerId,
        trackId,
        "Clip 1",
        {
          start: { numerator: 0, denominator: 1 },
          duration: { numerator: 100, denominator: 1 },
        },
        { PlaceholderMedia: { media_id: "media_123" } }
      );

      expect(engine.getTimeline(timelineId)?.layers[0].tracks[0].clips).toHaveLength(1);

      const result = engine.removeClip(timelineId, layerId, trackId, clipId);
      expect(result).toBe(true);
      expect(engine.getTimeline(timelineId)?.layers[0].tracks[0].clips).toHaveLength(0);
    });
  });

  describe("Timeline Querying", () => {
    it("should get active clips at a given time", () => {
      const engine = getRustEngine();
      const timelineId = engine.createTimeline({ numerator: 24, denominator: 1 });
      const layerId = engine.createLayer(timelineId, "Video", 0, "Video Layer");
      const trackId = engine.createTrack(timelineId, layerId, "Track 1");

      // Create clips at different times
      engine.createClip(
        timelineId,
        layerId,
        trackId,
        "Clip 1",
        {
          start: { numerator: 0, denominator: 1 },
          duration: { numerator: 50, denominator: 1 },
        },
        { PlaceholderMedia: { media_id: "media_1" } }
      );

      engine.createClip(
        timelineId,
        layerId,
        trackId,
        "Clip 2",
        {
          start: { numerator: 50, denominator: 1 },
          duration: { numerator: 50, denominator: 1 },
        },
        { PlaceholderMedia: { media_id: "media_2" } }
      );

      // Query at time 25 (should get Clip 1)
      const activeAt25 = engine.getActiveClipsAtTime(timelineId, {
        numerator: 25,
        denominator: 1,
      });
      expect(activeAt25).toHaveLength(1);
      expect(activeAt25[0].clip_id[0]).toBe(1);

      // Query at time 75 (should get Clip 2)
      const activeAt75 = engine.getActiveClipsAtTime(timelineId, {
        numerator: 75,
        denominator: 1,
      });
      expect(activeAt75).toHaveLength(1);
      expect(activeAt75[0].clip_id[0]).toBe(2);
    });

    it("should not return disabled clips as active", () => {
      const engine = getRustEngine();
      const timelineId = engine.createTimeline({ numerator: 24, denominator: 1 });
      const layerId = engine.createLayer(timelineId, "Video", 0, "Video Layer");
      const trackId = engine.createTrack(timelineId, layerId, "Track 1");

      const clipId = engine.createClip(
        timelineId,
        layerId,
        trackId,
        "Clip 1",
        {
          start: { numerator: 0, denominator: 1 },
          duration: { numerator: 100, denominator: 1 },
        },
        { PlaceholderMedia: { media_id: "media_1" } }
      );

      engine.setClipEnabled(timelineId, layerId, trackId, clipId, false);

      const activeClips = engine.getActiveClipsAtTime(timelineId, {
        numerator: 50,
        denominator: 1,
      });
      expect(activeClips).toHaveLength(0);
    });
  });

  describe("Serialization", () => {
    it("should serialize timeline to JSON", () => {
      const engine = getRustEngine();
      const timelineId = engine.createTimeline({ numerator: 24, denominator: 1 });
      const layerId = engine.createLayer(timelineId, "Video", 0, "Video Layer");
      const trackId = engine.createTrack(timelineId, layerId, "Track 1");

      engine.createClip(
        timelineId,
        layerId,
        trackId,
        "Clip 1",
        {
          start: { numerator: 0, denominator: 1 },
          duration: { numerator: 100, denominator: 1 },
        },
        { PlaceholderMedia: { media_id: "media_1" } }
      );

      const json = engine.serializeTimeline(timelineId);
      expect(json).toBeDefined();

      const parsed = JSON.parse(json);
      expect(parsed.frame_rate).toBeDefined();
      expect(parsed.layers).toBeDefined();
      expect(parsed.layers[0].tracks[0].clips[0].name).toBe("Clip 1");
    });

    it("should deserialize timeline from JSON", () => {
      const engine = getRustEngine();

      const json = JSON.stringify({
        frame_rate: { fps: { numerator: 30, denominator: 1 } },
        layers: [
          {
            id: { 0: 1 },
            name: "Layer 1",
            kind: "Video",
            z_order: 0,
            tracks: [],
          },
        ],
        next_id: 2,
      });

      const timelineId = engine.deserializeTimeline(json);
      expect(timelineId).toBeDefined();

      const timeline = engine.getTimeline(timelineId);
      expect(timeline?.frame_rate.fps.numerator).toBe(30);
      expect(timeline?.layers[0].name).toBe("Layer 1");
    });

    it("should return null when deserializing invalid JSON", () => {
      const engine = getRustEngine();
      const timelineId = engine.deserializeTimeline("invalid json");

      expect(timelineId).toBeNull();
    });
  });
});
