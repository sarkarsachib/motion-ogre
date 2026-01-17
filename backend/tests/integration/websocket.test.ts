/**
 * Integration tests for WebSocket server.
 */

import WebSocket from "ws";
import { getDatabase, resetDatabase } from "../../src/db/schema";
import { getRustEngine, resetRustEngine } from "../../src/bridge/rust";
import { WebSocketServerManager } from "../../src/ws/server";
import { PythonAgentBridge } from "../../src/bridge/python";
import { Logger } from "../../src/utils/logger";
import { v4 as uuidv4 } from "uuid";

describe("WebSocket Server Integration Tests", () => {
  let wsServer: WebSocketServerManager;
  let logger: Logger;
  let pythonBridge: PythonAgentBridge;
  let wsPort: number;
  let clients: WebSocket[];

  beforeEach((done) => {
    // Reset singletons
    resetDatabase();
    resetRustEngine();

    // Use a random available port
    wsPort = 3100 + Math.floor(Math.random() * 100);

    logger = new Logger("test");
    pythonBridge = new PythonAgentBridge(logger);
    wsServer = new WebSocketServerManager(wsPort, logger, pythonBridge);
    clients = [];

    // Wait for server to be ready
    setTimeout(done, 100);
  });

  afterEach(() => {
    // Close all client connections
    for (const client of clients) {
      if (client.readyState === WebSocket.OPEN) {
        client.close();
      }
    }
    clients = [];

    // Close server
    wsServer.close();
  });

  const createClient = (): Promise<WebSocket> => {
    return new Promise((resolve, reject) => {
      const client = new WebSocket(`ws://localhost:${wsPort}`);
      clients.push(client);

      client.on("open", () => resolve(client));
      client.on("error", reject);
    });
  };

  const sendCommand = (client: WebSocket, command: any): Promise<any> => {
    return new Promise((resolve, reject) => {
      const messageHandler = (data: Buffer) => {
        try {
          const response = JSON.parse(data.toString());
          if (response.id === command.id) {
            client.off("message", messageHandler);
            resolve(response);
          }
        } catch (e) {
          client.off("message", messageHandler);
          reject(e);
        }
      };

      client.on("message", messageHandler);
      client.send(JSON.stringify(command));
    });
  };

  describe("Connection Management", () => {
    it("should accept WebSocket connections", async () => {
      const client = await createClient();

      expect(client.readyState).toBe(WebSocket.OPEN);

      // Should receive welcome message
      const welcome = await new Promise((resolve) => {
        client.once("message", (data: Buffer) => {
          const message = JSON.parse(data.toString());
          resolve(message);
        });
      });

      const welcomeAny = welcome as any;
      expect(welcomeAny.status).toBe("success");
      expect(welcomeAny.data.type).toBe("connected");
      expect(welcomeAny.data.client_id).toBeDefined();
    });

    it("should handle multiple clients", async () => {
      const client1 = await createClient();
      const client2 = await createClient();
      const client3 = await createClient();

      expect(wsServer.getConnectedClientCount()).toBe(3);
    });

    it("should track disconnections", async () => {
      const client = await createClient();
      expect(wsServer.getConnectedClientCount()).toBe(1);

      client.close();

      // Wait for disconnect to be processed
      await new Promise((resolve) => setTimeout(resolve, 100));

      expect(wsServer.getConnectedClientCount()).toBe(0);
    });
  });

  describe("Timeline Commands", () => {
    it("should create a timeline via WebSocket", async () => {
      const client = await createClient();

      // Wait for welcome message
      await new Promise((resolve) => client.once("message", resolve));

      const command = {
        id: uuidv4(),
        type: "timeline_create",
        timestamp: Date.now(),
        frame_rate: { numerator: 24, denominator: 1 },
      };

      const response = await sendCommand(client, command);

      expect(response.status).toBe("success");
      expect(response.data.project_id).toBeDefined();
      expect(response.data.timeline_id).toBeDefined();
      expect(response.data.timeline).toBeDefined();
    });

    it("should create a layer", async () => {
      const client = await createClient();

      // Wait for welcome message
      await new Promise((resolve) => client.once("message", resolve));

      // Create a timeline first
      const createCommand = {
        id: uuidv4(),
        type: "timeline_create",
        timestamp: Date.now(),
        frame_rate: { numerator: 24, denominator: 1 },
      };

      const createResponse = await sendCommand(client, createCommand);
      const projectId = createResponse.data.project_id;

      // Create a layer
      const layerCommand = {
        id: uuidv4(),
        type: "layer_create",
        timestamp: Date.now(),
        project_id: projectId,
        data: {
          kind: "Video",
          z_order: 0,
          name: "Video Layer 1",
        },
      };

      const layerResponse = await sendCommand(client, layerCommand);

      expect(layerResponse.status).toBe("success");
      expect(layerResponse.data.layer_id).toBeDefined();
    });

    it("should create a track", async () => {
      const client = await createClient();

      // Wait for welcome message
      await new Promise((resolve) => client.once("message", resolve));

      // Create a timeline
      const createCommand = {
        id: uuidv4(),
        type: "timeline_create",
        timestamp: Date.now(),
        frame_rate: { numerator: 24, denominator: 1 },
      };

      const createResponse = await sendCommand(client, createCommand);
      const projectId = createResponse.data.project_id;

      // Create a layer first (we need a layer for track)
      const layerCommand = {
        id: uuidv4(),
        type: "layer_create",
        timestamp: Date.now(),
        project_id: projectId,
        data: {
          kind: "Video",
          z_order: 0,
          name: "Video Layer 1",
        },
      };

      const layerResponse = await sendCommand(client, layerCommand);
      const layerId = layerResponse.data.layer_id;

      // Create a track
      const trackCommand = {
        id: uuidv4(),
        type: "track_create",
        timestamp: Date.now(),
        project_id: projectId,
        data: {
          layer_id: layerId,
          name: "Track 1",
        },
      };

      const trackResponse = await sendCommand(client, trackCommand);

      expect(trackResponse.status).toBe("success");
      expect(trackResponse.data.track_id).toBeDefined();
    });

    it("should create a clip", async () => {
      const client = await createClient();

      // Wait for welcome message
      await new Promise((resolve) => client.once("message", resolve));

      // Create a timeline
      const createCommand = {
        id: uuidv4(),
        type: "timeline_create",
        timestamp: Date.now(),
        frame_rate: { numerator: 24, denominator: 1 },
      };

      const createResponse = await sendCommand(client, createCommand);
      const projectId = createResponse.data.project_id;

      // Create a layer
      const layerCommand = {
        id: uuidv4(),
        type: "layer_create",
        timestamp: Date.now(),
        project_id: projectId,
        data: {
          kind: "Video",
          z_order: 0,
          name: "Video Layer 1",
        },
      };

      const layerResponse = await sendCommand(client, layerCommand);
      const layerId = layerResponse.data.layer_id;

      // Create a track
      const trackCommand = {
        id: uuidv4(),
        type: "track_create",
        timestamp: Date.now(),
        project_id: projectId,
        data: {
          layer_id: layerId,
          name: "Track 1",
        },
      };

      const trackResponse = await sendCommand(client, trackCommand);
      const trackId = trackResponse.data.track_id;

      // Create a clip
      const clipCommand = {
        id: uuidv4(),
        type: "clip_create",
        timestamp: Date.now(),
        project_id: projectId,
        data: {
          layer_id: layerId,
          track_id: trackId,
          name: "Clip 1",
          range: {
            start: { numerator: 0, denominator: 1 },
            duration: { numerator: 100, denominator: 1 },
          },
          source: {
            PlaceholderMedia: { media_id: "media_123" },
          },
        },
      };

      const clipResponse = await sendCommand(client, clipCommand);

      expect(clipResponse.status).toBe("success");
      expect(clipResponse.data.clip_id).toBeDefined();
    });
  });

  describe("Playhead Commands", () => {
    it("should handle playhead move command", async () => {
      const client = await createClient();

      // Wait for welcome message
      await new Promise((resolve) => client.once("message", resolve));

      // Create a timeline
      const createCommand = {
        id: uuidv4(),
        type: "timeline_create",
        timestamp: Date.now(),
        frame_rate: { numerator: 24, denominator: 1 },
      };

      const createResponse = await sendCommand(client, createCommand);
      const projectId = createResponse.data.project_id;

      // Move playhead
      const playheadCommand = {
        id: uuidv4(),
        type: "playhead_move",
        timestamp: Date.now(),
        project_id: projectId,
        time: { numerator: 50, denominator: 1 },
      };

      const response = await sendCommand(client, playheadCommand);

      expect(response.status).toBe("success");
    });

    it("should handle playhead play command", async () => {
      const client = await createClient();

      // Wait for welcome message
      await new Promise((resolve) => client.once("message", resolve));

      // Create a timeline
      const createCommand = {
        id: uuidv4(),
        type: "timeline_create",
        timestamp: Date.now(),
        frame_rate: { numerator: 24, denominator: 1 },
      };

      const createResponse = await sendCommand(client, createCommand);
      const projectId = createResponse.data.project_id;

      // Play
      const playCommand = {
        id: uuidv4(),
        type: "playhead_play",
        timestamp: Date.now(),
        project_id: projectId,
      };

      const response = await sendCommand(client, playCommand);

      expect(response.status).toBe("success");
    });

    it("should handle playhead pause command", async () => {
      const client = await createClient();

      // Wait for welcome message
      await new Promise((resolve) => client.once("message", resolve));

      // Create a timeline
      const createCommand = {
        id: uuidv4(),
        type: "timeline_create",
        timestamp: Date.now(),
        frame_rate: { numerator: 24, denominator: 1 },
      };

      const createResponse = await sendCommand(client, createCommand);
      const projectId = createResponse.data.project_id;

      // Pause
      const pauseCommand = {
        id: uuidv4(),
        type: "playhead_pause",
        timestamp: Date.now(),
        project_id: projectId,
        time: { numerator: 100, denominator: 1 },
      };

      const response = await sendCommand(client, pauseCommand);

      expect(response.status).toBe("success");
    });
  });
});
