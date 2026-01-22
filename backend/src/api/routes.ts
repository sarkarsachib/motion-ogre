/**
 * REST API routes for Motion Ogre backend.
 */

import express, { Request, Response, Router } from "express";
import { getDatabase } from "../db/schema";
import { getRustEngine } from "../bridge/rust";
import { Logger } from "../utils/logger";
import { z } from "zod";

// Validation schemas
const createProjectSchema = z.object({
  name: z.string().min(1).max(255),
  frame_rate: z.object({
    numerator: z.number().positive(),
    denominator: z.number().positive(),
  }),
});

const createLayerSchema = z.object({
  kind: z.enum(["Video", "Audio", "Adjustment", "Vfx"]),
  z_order: z.number().integer(),
  name: z.string().min(1).max(255),
});

const createTrackSchema = z.object({
  layer_id: z.object({ 0: z.number().positive() }),
  name: z.string().min(1).max(255),
});

const createClipSchema = z.object({
  layer_id: z.object({ 0: z.number().positive() }),
  track_id: z.object({ 0: z.number().positive() }),
  name: z.string().min(1).max(255),
  range: z.object({
    start: z.object({
      numerator: z.number(),
      denominator: z.number().positive(),
    }),
    duration: z.object({
      numerator: z.number().positive(),
      denominator: z.number().positive(),
    }),
  }),
  source: z.object({
    PlaceholderMedia: z.object({
      media_id: z.string(),
    }),
  }),
});

export function createRoutes(logger: Logger): Router {
  const router = express.Router();
  const db = getDatabase();
  const rustEngine = getRustEngine();

  // Health check
  router.get("/health", (req: Request, res: Response) => {
    res.json({
      status: "ok",
      timestamp: Date.now(),
      version: "0.1.0",
    });
  });

  // List projects
  router.get("/projects", (req: Request, res: Response) => {
    try {
      const limit = Math.min(parseInt(req.query.limit as string) || 100, 1000);
      const offset = parseInt(req.query.offset as string) || 0;

      const projects = db.listProjects(limit, offset);
      res.json({
        projects,
        count: projects.length,
      });
    } catch (error) {
      logger.error("Error listing projects", { error: String(error) });
      res.status(500).json({
        error: "Failed to list projects",
      });
    }
  });

  // Create project
  router.post("/projects", async (req: Request, res: Response) => {
    try {
      const validated = createProjectSchema.parse(req.body);

      // Create timeline in Rust engine
      const timelineId = rustEngine.createTimeline(validated.frame_rate);
      const timelineJson = rustEngine.serializeTimeline(timelineId)!;

      // Create project in database
      const project = db.createProject(validated.name, timelineJson);

      logger.info("Project created", { project_id: project.id, name: project.name });

      res.status(201).json({
        project_id: project.id,
        name: project.name,
        created_at: project.created_at,
        updated_at: project.updated_at,
        timeline: JSON.parse(timelineJson),
      });
    } catch (error) {
      if (error instanceof z.ZodError) {
        res.status(400).json({
          error: "Invalid request body",
          details: error.errors,
        });
      } else {
        logger.error("Error creating project", { error: String(error) });
        res.status(500).json({
          error: "Failed to create project",
        });
      }
    }
  });

  // Get project
  router.get("/projects/:id", (req: Request, res: Response) => {
    try {
      const project = db.getProject(req.params.id);
      if (!project) {
        return res.status(404).json({
          error: "Project not found",
        });
      }

      res.json({
        project_id: project.id,
        name: project.name,
        created_at: project.created_at,
        updated_at: project.updated_at,
        version: project.version,
        timeline: JSON.parse(project.timeline_json),
      });
    } catch (error) {
      logger.error("Error getting project", {
        project_id: req.params.id,
        error: String(error),
      });
      res.status(500).json({
        error: "Failed to get project",
      });
    }
  });

  // Update project (save timeline)
  router.put("/projects/:id", async (req: Request, res: Response) => {
    try {
      const timelineSchema = z.object({
        timeline: z.any(),
      });
      const validated = timelineSchema.parse(req.body);

      const timelineJson = JSON.stringify(validated.timeline);
      const project = db.updateProject(req.params.id, timelineJson);

      if (!project) {
        return res.status(404).json({
          error: "Project not found",
        });
      }

      logger.info("Project updated", { project_id: project.id });

      res.json({
        project_id: project.id,
        name: project.name,
        updated_at: project.updated_at,
        version: project.version,
      });
    } catch (error) {
      if (error instanceof z.ZodError) {
        res.status(400).json({
          error: "Invalid request body",
          details: error.errors,
        });
      } else {
        logger.error("Error updating project", {
          project_id: req.params.id,
          error: String(error),
        });
        res.status(500).json({
          error: "Failed to update project",
        });
      }
    }
  });

  // Delete project
  router.delete("/projects/:id", (req: Request, res: Response) => {
    try {
      const success = db.deleteProject(req.params.id);
      if (!success) {
        return res.status(404).json({
          error: "Project not found",
        });
      }

      logger.info("Project deleted", { project_id: req.params.id });

      res.status(204).send();
    } catch (error) {
      logger.error("Error deleting project", {
        project_id: req.params.id,
        error: String(error),
      });
      res.status(500).json({
        error: "Failed to delete project",
      });
    }
  });

  // Timeline query - get active clips at time
  router.get("/projects/:id/timeline/query", (req: Request, res: Response) => {
    try {
      const project = db.getProject(req.params.id);
      if (!project) {
        return res.status(404).json({
          error: "Project not found",
        });
      }

      const { numerator, denominator } = req.query;
      if (!numerator || !denominator) {
        return res.status(400).json({
          error: "Missing time parameters: numerator and denominator required",
        });
      }

      const timeline = JSON.parse(project.timeline_json);
      const timelineId = "temp_timeline";
      rustEngine.deserializeTimeline(project.timeline_json);

      const time = {
        numerator: parseInt(numerator as string),
        denominator: parseInt(denominator as string),
      };

      const activeClips = rustEngine.getActiveClipsAtTime(timelineId, time);

      res.json({
        time,
        active_clips: activeClips,
      });
    } catch (error) {
      logger.error("Error querying timeline", {
        project_id: req.params.id,
        error: String(error),
      });
      res.status(500).json({
        error: "Failed to query timeline",
      });
    }
  });

  // List drafts for project
  router.get("/projects/:id/drafts", (req: Request, res: Response) => {
    try {
      const drafts = db.listDrafts(req.params.id);
      res.json({
        drafts,
        count: drafts.length,
      });
    } catch (error) {
      logger.error("Error listing drafts", {
        project_id: req.params.id,
        error: String(error),
      });
      res.status(500).json({
        error: "Failed to list drafts",
      });
    }
  });

  // Create draft
  router.post("/projects/:id/drafts", async (req: Request, res: Response) => {
    try {
      const draftSchema = z.object({
        name: z.string().min(1).max(255),
        timeline: z.any(),
      });
      const validated = draftSchema.parse(req.body);

      const timelineJson = JSON.stringify(validated.timeline);
      const draft = db.createDraft(req.params.id, validated.name, timelineJson);

      logger.info("Draft created", {
        project_id: req.params.id,
        draft_id: draft.id,
      });

      res.status(201).json({
        draft_id: draft.id,
        project_id: draft.project_id,
        name: draft.name,
        created_at: draft.created_at,
        timeline: JSON.parse(timelineJson),
      });
    } catch (error) {
      if (error instanceof z.ZodError) {
        res.status(400).json({
          error: "Invalid request body",
          details: error.errors,
        });
      } else {
        logger.error("Error creating draft", {
          project_id: req.params.id,
          error: String(error),
        });
        res.status(500).json({
          error: "Failed to create draft",
        });
      }
    }
  });

  // Delete draft
  router.delete("/drafts/:id", (req: Request, res: Response) => {
    try {
      const success = db.deleteDraft(req.params.id);
      if (!success) {
        return res.status(404).json({
          error: "Draft not found",
        });
      }

      logger.info("Draft deleted", { draft_id: req.params.id });

      res.status(204).send();
    } catch (error) {
      logger.error("Error deleting draft", {
        draft_id: req.params.id,
        error: String(error),
      });
      res.status(500).json({
        error: "Failed to delete draft",
      });
    }
  });

  return router;
}
