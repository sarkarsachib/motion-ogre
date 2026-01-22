/**
 * WebSocket command types for real-time communication.
 */

import { Timeline, CreateLayerCommand, CreateTrackCommand, CreateClipCommand, UpdateClipCommand } from "./timeline";

export type CommandType =
  | "timeline_create"
  | "timeline_get"
  | "timeline_update"
  | "timeline_query"
  | "layer_create"
  | "layer_remove"
  | "track_create"
  | "clip_create"
  | "clip_update"
  | "clip_remove"
  | "playhead_move"
  | "playhead_play"
  | "playhead_pause"
  | "agent_execute"
  | "render_start"
  | "render_cancel";

export interface BaseCommand {
  id: string;
  type: CommandType;
  timestamp: number;
}

export interface TimelineCreateCommand extends BaseCommand {
  type: "timeline_create";
  frame_rate: { numerator: number; denominator: number };
}

export interface TimelineGetCommand extends BaseCommand {
  type: "timeline_get";
  project_id: string;
}

export interface LayerCreateCommand extends BaseCommand {
  type: "layer_create";
  project_id: string;
  data: CreateLayerCommand;
}

export interface TrackCreateCommand extends BaseCommand {
  type: "track_create";
  project_id: string;
  data: CreateTrackCommand;
}

export interface ClipCreateCommand extends BaseCommand {
  type: "clip_create";
  project_id: string;
  data: CreateClipCommand;
}

export interface ClipUpdateCommand extends BaseCommand {
  type: "clip_update";
  project_id: string;
  data: UpdateClipCommand;
}

export interface PlayheadMoveCommand extends BaseCommand {
  type: "playhead_move";
  project_id: string;
  time: { numerator: number; denominator: number };
}

export interface PlayheadPlayCommand extends BaseCommand {
  type: "playhead_play";
  project_id: string;
}

export interface PlayheadPauseCommand extends BaseCommand {
  type: "playhead_pause";
  project_id: string;
}

export interface AgentExecuteCommand extends BaseCommand {
  type: "agent_execute";
  project_id: string;
  agent_type: string;
  intent: string;
  context?: Record<string, unknown>;
}

export interface RenderStartCommand extends BaseCommand {
  type: "render_start";
  project_id: string;
  output_path: string;
  format: "mp4" | "webm" | "prores";
  quality: "low" | "medium" | "high";
}

export type Command =
  | TimelineCreateCommand
  | TimelineGetCommand
  | LayerCreateCommand
  | TrackCreateCommand
  | ClipCreateCommand
  | ClipUpdateCommand
  | PlayheadMoveCommand
  | PlayheadPlayCommand
  | PlayheadPauseCommand
  | AgentExecuteCommand
  | RenderStartCommand;

/**
 * Response types
 */
export type ResponseStatus = "success" | "error" | "pending";

export interface BaseResponse {
  id: string;
  status: ResponseStatus;
  timestamp: number;
}

export interface SuccessResponse<T = unknown> extends BaseResponse {
  status: "success";
  data: T;
}

export interface ErrorResponse extends BaseResponse {
  status: "error";
  error: {
    code: string;
    message: string;
    details?: unknown;
  };
}

export interface PendingResponse extends BaseResponse {
  status: "pending";
  message: string;
}

export type Response = SuccessResponse | ErrorResponse | PendingResponse;

/**
 * Broadcast events
 */
export type EventType =
  | "timeline_updated"
  | "playhead_moved"
  | "playhead_playing"
  | "playhead_paused"
  | "agent_started"
  | "agent_progress"
  | "agent_completed"
  | "render_started"
  | "render_progress"
  | "render_completed";

export interface BaseEvent {
  type: EventType;
  project_id: string;
  timestamp: number;
}

export interface TimelineUpdatedEvent extends BaseEvent {
  type: "timeline_updated";
  timeline: Timeline;
  change_summary: string;
}

export interface PlayheadMovedEvent extends BaseEvent {
  type: "playhead_moved";
  time: { numerator: number; denominator: number };
  is_playing: boolean;
}

export interface PlayheadPlayingEvent extends BaseEvent {
  type: "playhead_playing";
}

export interface PlayheadPausedEvent extends BaseEvent {
  type: "playhead_paused";
  time: { numerator: number; denominator: number };
}

export interface AgentStartedEvent extends BaseEvent {
  type: "agent_started";
  agent_type: string;
  execution_id: string;
}

export interface AgentProgressEvent extends BaseEvent {
  type: "agent_progress";
  agent_type: string;
  execution_id: string;
  progress: number;
  message: string;
}

export interface AgentCompletedEvent extends BaseEvent {
  type: "agent_completed";
  agent_type: string;
  execution_id: string;
  result: unknown;
}

export interface RenderStartedEvent extends BaseEvent {
  type: "render_started";
  render_id: string;
  output_path: string;
}

export interface RenderProgressEvent extends BaseEvent {
  type: "render_progress";
  render_id: string;
  progress: number;
  frame: number;
  total_frames: number;
}

export interface RenderCompletedEvent extends BaseEvent {
  type: "render_completed";
  render_id: string;
  output_path: string;
  duration_ms: number;
}

export type Event =
  | TimelineUpdatedEvent
  | PlayheadMovedEvent
  | PlayheadPlayingEvent
  | PlayheadPausedEvent
  | AgentStartedEvent
  | AgentProgressEvent
  | AgentCompletedEvent
  | RenderStartedEvent
  | RenderProgressEvent
  | RenderCompletedEvent;
