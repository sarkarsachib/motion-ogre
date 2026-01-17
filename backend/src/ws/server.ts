/**
 * WebSocket server for real-time timeline synchronization.
 */

import { WebSocketServer, WebSocket } from "ws";
import { v4 as uuidv4 } from "uuid";
import { EventEmitter } from "events";
import { Logger } from "../utils/logger";
import { Command, Response, Event, ResponseStatus } from "../types/commands";
import { getRustEngine } from "../bridge/rust";
import { PythonAgentBridge } from "../bridge/python";
import { getDatabase } from "../db/schema";

interface ClientConnection {
  id: string;
  ws: WebSocket;
  projectId: string | null;
  userId: string | null;
  connectedAt: number;
}

export class WebSocketServerManager extends EventEmitter {
  private wss: WebSocketServer;
  private clients: Map<string, ClientConnection>;
  private clientToProject: Map<string, string>;
  private projectToClients: Map<string, Set<string>>;
  private logger: Logger;
  private pythonBridge: PythonAgentBridge;
  private rustEngine = getRustEngine();
  private db = getDatabase();

  constructor(port: number, logger: Logger, pythonBridge: PythonAgentBridge) {
    super();
    this.wss = new WebSocketServer({ port });
    this.clients = new Map();
    this.clientToProject = new Map();
    this.projectToClients = new Map();
    this.logger = logger.child("WebSocketServer");
    this.pythonBridge = pythonBridge;

    this.setupServer();
    this.setupPythonBridgeListeners();

    this.logger.info(`WebSocket server started on port ${port}`);
  }

  private setupServer(): void {
    this.wss.on("connection", (ws: WebSocket) => {
      const clientId = uuidv4();
      const client: ClientConnection = {
        id: clientId,
        ws,
        projectId: null,
        userId: null,
        connectedAt: Date.now(),
      };

      this.clients.set(clientId, client);
      this.logger.info("Client connected", { client_id: clientId });

      // Send welcome message
      this.sendToClient(clientId, {
        id: uuidv4(),
        status: "success",
        timestamp: Date.now(),
        data: {
          type: "connected",
          client_id: clientId,
        },
      });

      // Handle messages
      ws.on("message", (data: Buffer) => {
        this.handleMessage(clientId, data);
      });

      // Handle disconnect
      ws.on("close", () => {
        this.handleDisconnect(clientId);
      });

      // Handle errors
      ws.on("error", (error) => {
        this.logger.error("WebSocket error", { client_id: clientId, error: String(error) });
        this.handleDisconnect(clientId);
      });

      // Handle pings
      ws.on("ping", () => {
        ws.pong();
      });
    });
  }

  private setupPythonBridgeListeners(): void {
    this.pythonBridge.on("progress", (progress) => {
      const event: Event = {
        type: "agent_progress",
        project_id: "", // Will be set based on execution context
        timestamp: Date.now(),
        ...progress,
      };
      this.broadcastEvent(event);
    });
  }

  private async handleMessage(clientId: string, data: Buffer): Promise<void> {
    try {
      const message: Command = JSON.parse(data.toString());

      this.logger.debug("Received command", {
        client_id: clientId,
        command_type: message.type,
        command_id: message.id,
      });

      const response = await this.handleCommand(clientId, message);
      this.sendToClient(clientId, response);

    } catch (error) {
      this.logger.error("Error handling message", {
        client_id: clientId,
        error: String(error),
      });

      this.sendToClient(clientId, {
        id: uuidv4(),
        status: "error",
        timestamp: Date.now(),
        error: {
          code: "INVALID_MESSAGE",
          message: "Failed to parse message",
          details: String(error),
        },
      });
    }
  }

  private async handleCommand(clientId: string, command: Command): Promise<Response> {
    const client = this.clients.get(clientId);
    if (!client) {
      return {
        id: command.id,
        status: "error",
        timestamp: Date.now(),
        error: {
          code: "CLIENT_NOT_FOUND",
          message: "Client not found",
        },
      };
    }

    switch (command.type) {
      case "timeline_create":
        return await this.handleTimelineCreate(client, command);
      case "timeline_get":
        return await this.handleTimelineGet(client, command);
      case "layer_create":
        return await this.handleLayerCreate(client, command);
      case "track_create":
        return await this.handleTrackCreate(client, command);
      case "clip_create":
        return await this.handleClipCreate(client, command);
      case "clip_update":
        return await this.handleClipUpdate(client, command);
      case "playhead_move":
        return await this.handlePlayheadMove(client, command);
      case "playhead_play":
        return await this.handlePlayheadPlay(client, command);
      case "playhead_pause":
        return await this.handlePlayheadPause(client, command);
      case "agent_execute":
        return await this.handleAgentExecute(client, command);
      default:
        return {
          id: command.id,
          status: "error",
          timestamp: Date.now(),
          error: {
            code: "UNKNOWN_COMMAND",
            message: `Unknown command type: ${command.type}`,
          },
        };
    }
  }

  private async handleTimelineCreate(
    client: ClientConnection,
    command: any
  ): Promise<Response> {
    const timelineId = this.rustEngine.createTimeline(command.frame_rate);

    // Create project in database
    const project = this.db.createProject("Untitled Project", this.rustEngine.serializeTimeline(timelineId)!);

    // Associate client with project
    client.projectId = project.id;
    this.clientToProject.set(client.id, project.id);

    if (!this.projectToClients.has(project.id)) {
      this.projectToClients.set(project.id, new Set());
    }
    this.projectToClients.get(project.id)!.add(client.id);

    const timeline = this.rustEngine.getTimeline(timelineId);

    return {
      id: command.id,
      status: "success",
      timestamp: Date.now(),
      data: {
        project_id: project.id,
        timeline_id: timelineId,
        timeline,
      },
    };
  }

  private async handleTimelineGet(
    client: ClientConnection,
    command: any
  ): Promise<Response> {
    const project = this.db.getProject(command.project_id);
    if (!project) {
      return {
        id: command.id,
        status: "error",
        timestamp: Date.now(),
        error: {
          code: "PROJECT_NOT_FOUND",
          message: "Project not found",
        },
      };
    }

    const timeline = JSON.parse(project.timeline_json);

    return {
      id: command.id,
      status: "success",
      timestamp: Date.now(),
      data: {
        project_id: project.id,
        timeline,
      },
    };
  }

  private async handleLayerCreate(
    client: ClientConnection,
    command: any
  ): Promise<Response> {
    const project = this.db.getProject(command.project_id);
    if (!project) {
      return {
        id: command.id,
        status: "error",
        timestamp: Date.now(),
        error: {
          code: "PROJECT_NOT_FOUND",
          message: "Project not found",
        },
      };
    }

    // For now, we'll store the timeline in memory
    // In production, this should be loaded from the project
    const timelineId = "default_timeline"; // This should be stored in project
    const layerId = this.rustEngine.createLayer(
      timelineId,
      command.data.kind,
      command.data.z_order,
      command.data.name
    );

    if (!layerId) {
      return {
        id: command.id,
        status: "error",
        timestamp: Date.now(),
        error: {
          code: "LAYER_CREATE_FAILED",
          message: "Failed to create layer",
        },
      };
    }

    const timeline = this.rustEngine.getTimeline(timelineId);
    if (!timeline) {
      return {
        id: command.id,
        status: "error",
        timestamp: Date.now(),
        error: {
          code: "TIMELINE_NOT_FOUND",
          message: "Timeline not found",
        },
      };
    }

    // Broadcast update to all clients in project
    this.broadcastEvent({
      type: "timeline_updated",
      project_id: command.project_id,
      timestamp: Date.now(),
      timeline,
      change_summary: `Created layer: ${command.data.name}`,
    });

    return {
      id: command.id,
      status: "success",
      timestamp: Date.now(),
      data: {
        layer_id: layerId,
      },
    };
  }

  private async handleTrackCreate(
    client: ClientConnection,
    command: any
  ): Promise<Response> {
    const project = this.db.getProject(command.project_id);
    if (!project) {
      return {
        id: command.id,
        status: "error",
        timestamp: Date.now(),
        error: {
          code: "PROJECT_NOT_FOUND",
          message: "Project not found",
        },
      };
    }

    const timelineId = "default_timeline";
    const trackId = this.rustEngine.createTrack(
      timelineId,
      command.data.layer_id,
      command.data.name
    );

    if (!trackId) {
      return {
        id: command.id,
        status: "error",
        timestamp: Date.now(),
        error: {
          code: "TRACK_CREATE_FAILED",
          message: "Failed to create track",
        },
      };
    }

    const timeline = this.rustEngine.getTimeline(timelineId);

    this.broadcastEvent({
      type: "timeline_updated",
      project_id: command.project_id,
      timestamp: Date.now(),
      timeline: timeline!,
      change_summary: `Created track: ${command.data.name}`,
    });

    return {
      id: command.id,
      status: "success",
      timestamp: Date.now(),
      data: {
        track_id: trackId,
      },
    };
  }

  private async handleClipCreate(
    client: ClientConnection,
    command: any
  ): Promise<Response> {
    const project = this.db.getProject(command.project_id);
    if (!project) {
      return {
        id: command.id,
        status: "error",
        timestamp: Date.now(),
        error: {
          code: "PROJECT_NOT_FOUND",
          message: "Project not found",
        },
      };
    }

    const timelineId = "default_timeline";
    const clipId = this.rustEngine.createClip(
      timelineId,
      command.data.layer_id,
      command.data.track_id,
      command.data.name,
      command.data.range,
      command.data.source
    );

    if (!clipId) {
      return {
        id: command.id,
        status: "error",
        timestamp: Date.now(),
        error: {
          code: "CLIP_CREATE_FAILED",
          message: "Failed to create clip",
        },
      };
    }

    const timeline = this.rustEngine.getTimeline(timelineId);

    this.broadcastEvent({
      type: "timeline_updated",
      project_id: command.project_id,
      timestamp: Date.now(),
      timeline: timeline!,
      change_summary: `Created clip: ${command.data.name}`,
    });

    return {
      id: command.id,
      status: "success",
      timestamp: Date.now(),
      data: {
        clip_id: clipId,
      },
    };
  }

  private async handleClipUpdate(
    client: ClientConnection,
    command: any
  ): Promise<Response> {
    const project = this.db.getProject(command.project_id);
    if (!project) {
      return {
        id: command.id,
        status: "error",
        timestamp: Date.now(),
        error: {
          code: "PROJECT_NOT_FOUND",
          message: "Project not found",
        },
      };
    }

    const timelineId = "default_timeline";
    const success = this.rustEngine.updateClipRange(
      timelineId,
      command.data.layer_id,
      command.data.track_id,
      command.data.clip_id,
      command.data.range
    );

    if (!success) {
      return {
        id: command.id,
        status: "error",
        timestamp: Date.now(),
        error: {
          code: "CLIP_UPDATE_FAILED",
          message: "Failed to update clip",
        },
      };
    }

    const timeline = this.rustEngine.getTimeline(timelineId);

    this.broadcastEvent({
      type: "timeline_updated",
      project_id: command.project_id,
      timestamp: Date.now(),
      timeline: timeline!,
      change_summary: "Updated clip range",
    });

    return {
      id: command.id,
      status: "success",
      timestamp: Date.now(),
      data: { success: true },
    };
  }

  private async handlePlayheadMove(
    client: ClientConnection,
    command: any
  ): Promise<Response> {
    this.broadcastEvent({
      type: "playhead_moved",
      project_id: command.project_id,
      timestamp: Date.now(),
      time: command.time,
      is_playing: false,
    });

    return {
      id: command.id,
      status: "success",
      timestamp: Date.now(),
      data: { success: true },
    };
  }

  private async handlePlayheadPlay(
    client: ClientConnection,
    command: any
  ): Promise<Response> {
    this.broadcastEvent({
      type: "playhead_playing",
      project_id: command.project_id,
      timestamp: Date.now(),
    });

    return {
      id: command.id,
      status: "success",
      timestamp: Date.now(),
      data: { success: true },
    };
  }

  private async handlePlayheadPause(
    client: ClientConnection,
    command: any
  ): Promise<Response> {
    this.broadcastEvent({
      type: "playhead_paused",
      project_id: command.project_id,
      timestamp: Date.now(),
      time: command.time,
    });

    return {
      id: command.id,
      status: "success",
      timestamp: Date.now(),
      data: { success: true },
    };
  }

  private async handleAgentExecute(
    client: ClientConnection,
    command: any
  ): Promise<Response> {
    const project = this.db.getProject(command.project_id);
    if (!project) {
      return {
        id: command.id,
        status: "error",
        timestamp: Date.now(),
        error: {
          code: "PROJECT_NOT_FOUND",
          message: "Project not found",
        },
      };
    }

    // Broadcast agent started event
    this.broadcastEvent({
      type: "agent_started",
      project_id: command.project_id,
      timestamp: Date.now(),
      agent_type: command.agent_type,
      execution_id: command.id,
    });

    // Execute agent asynchronously
    const result = await this.pythonBridge.executeAgent({
      id: command.id,
      agent_type: command.agent_type,
      intent: command.intent,
      context: command.context,
    });

    // Broadcast agent completed event
    this.broadcastEvent({
      type: "agent_completed",
      project_id: command.project_id,
      timestamp: Date.now(),
      agent_type: command.agent_type,
      execution_id: command.id,
      result: result.result,
    });

    return {
      id: command.id,
      status: result.success ? "success" : "error",
      timestamp: Date.now(),
      data: result.success ? result.result : undefined,
      error: result.success ? undefined : {
        code: "AGENT_EXECUTION_FAILED",
        message: result.error || "Agent execution failed",
      },
    };
  }

  private handleDisconnect(clientId: string): void {
    const client = this.clients.get(clientId);
    if (!client) return;

    // Remove from project mappings
    if (client.projectId) {
      this.clientToProject.delete(clientId);
      const projectClients = this.projectToClients.get(client.projectId);
      if (projectClients) {
        projectClients.delete(clientId);
        if (projectClients.size === 0) {
          this.projectToClients.delete(client.projectId);
        }
      }
    }

    this.clients.delete(clientId);
    this.logger.info("Client disconnected", { client_id: clientId });
  }

  private sendToClient(clientId: string, message: Response): void {
    const client = this.clients.get(clientId);
    if (!client) return;

    try {
      client.ws.send(JSON.stringify(message));
    } catch (error) {
      this.logger.error("Error sending to client", {
        client_id: clientId,
        error: String(error),
      });
    }
  }

  private broadcastEvent(event: Event): void {
    const projectClients = this.projectToClients.get(event.project_id);
    if (!projectClients) return;

    const message = JSON.stringify(event);

    for (const clientId of projectClients) {
      const client = this.clients.get(clientId);
      if (client && client.ws.readyState === client.ws.OPEN) {
        try {
          client.ws.send(message);
        } catch (error) {
          this.logger.error("Error broadcasting to client", {
            client_id: clientId,
            error: String(error),
          });
        }
      }
    }
  }

  /**
   * Get connected client count.
   */
  getConnectedClientCount(): number {
    return this.clients.size;
  }

  /**
   * Get client info.
   */
  getClientInfo(clientId: string): ClientConnection | undefined {
    return this.clients.get(clientId);
  }

  /**
   * Get all connected clients.
   */
  getAllClients(): ClientConnection[] {
    return Array.from(this.clients.values());
  }

  /**
   * Close all connections.
   */
  close(): void {
    for (const client of this.clients.values()) {
      client.ws.close();
    }
    this.clients.clear();
    this.clientToProject.clear();
    this.projectToClients.clear();
    this.wss.close();
  }
}
