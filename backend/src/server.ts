/**
 * Main server entry point for Motion Ogre backend.
 */

import express from "express";
import cors from "cors";
import { getLogger, Logger } from "./utils/logger";
import { getMetricsCollector } from "./utils/metrics";
import { createRoutes } from "./api/routes";
import { WebSocketServerManager } from "./ws/server";
import { PythonAgentBridge } from "./bridge/python";

const PORT = process.env.PORT || 3000;
const WS_PORT = process.env.WS_PORT || 3001;
const LOG_LEVEL = (process.env.LOG_LEVEL as "info" | "debug" | "error") || "info";

async function main() {
  // Initialize logger
  const logger = getLogger("MotionOgreBackend", LOG_LEVEL);
  logger.info("Starting Motion Ogre backend", {
    port: PORT,
    ws_port: WS_PORT,
    log_level: LOG_LEVEL,
  });

  const metrics = getMetricsCollector(logger);

  // Initialize Express app
  const app = express();
  app.use(cors());
  app.use(express.json());
  app.use(express.urlencoded({ extended: true }));

  // Request logging middleware
  app.use((req, res, next) => {
    const start = Date.now();

    res.on("finish", () => {
      const duration = Date.now() - start;
      logger.http(`${req.method} ${req.path}`, {
        status: res.statusCode,
        duration_ms: duration,
      });
      metrics.recordTiming(`http_request_${req.method.toLowerCase()}_${res.statusCode}`, duration);
    });

    next();
  });

  // Mount API routes
  app.use("/api", createRoutes(logger));

  // Error handling middleware
  app.use((err: Error, req: express.Request, res: express.Response, next: express.NextFunction) => {
    logger.error("Unhandled error", {
      error: err.message,
      stack: err.stack,
      path: req.path,
      method: req.method,
    });

    res.status(500).json({
      error: "Internal server error",
      message: err.message,
    });
  });

  // Initialize Python agent bridge
  const pythonBridge = new PythonAgentBridge(logger);

  // Check Python availability
  const pythonAvailable = await pythonBridge.checkPythonAvailable();
  if (!pythonAvailable) {
    logger.warn("Python is not available. Agent execution will fail.");
  } else {
    logger.info("Python is available for agent execution");
  }

  // Initialize WebSocket server
  const wsServer = new WebSocketServerManager(WS_PORT, logger, pythonBridge);

  // Graceful shutdown handler
  const shutdown = async (signal: string) => {
    logger.info(`Received ${signal}, shutting down gracefully...`);

    wsServer.close();
    pythonBridge.cleanup();

    process.exit(0);
  };

  process.on("SIGTERM", () => shutdown("SIGTERM"));
  process.on("SIGINT", () => shutdown("SIGINT"));

  // Start HTTP server
  const server = app.listen(PORT, () => {
    logger.info(`HTTP server listening on port ${PORT}`);
    logger.info(`WebSocket server listening on port ${WS_PORT}`);
    logger.info("Ready to accept connections");
  });

  server.on("error", (error) => {
    logger.fatal("Failed to start server", { error: String(error) });
    process.exit(1);
  });
}

main().catch((error) => {
  console.error("Fatal error starting server:", error);
  process.exit(1);
});
