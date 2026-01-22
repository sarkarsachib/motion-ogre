/**
 * Integration tests for REST API.
 */

import request from "supertest";
import express from "express";
import { getDatabase, resetDatabase } from "../../src/db/schema";
import { getRustEngine, resetRustEngine } from "../../src/bridge/rust";
import { createRoutes } from "../../src/api/routes";
import { Logger } from "../../src/utils/logger";

describe("REST API Integration Tests", () => {
  let app: express.Application;
  let logger: Logger;

  beforeEach(() => {
    // Reset singletons
    resetDatabase();
    resetRustEngine();

    logger = new Logger("test");
    app = express();
    app.use(express.json());
    app.use("/api", createRoutes(logger));
  });

  describe("Health Check", () => {
    it("should return health status", async () => {
      const response = await request(app).get("/api/health");

      expect(response.status).toBe(200);
      expect(response.body.status).toBe("ok");
      expect(response.body.version).toBe("0.1.0");
      expect(response.body.timestamp).toBeDefined();
    });
  });

  describe("Project CRUD", () => {
    it("should create a new project", async () => {
      const newProject = {
        name: "Test Project",
        frame_rate: { numerator: 24, denominator: 1 },
      };

      const response = await request(app)
        .post("/api/projects")
        .send(newProject);

      expect(response.status).toBe(201);
      const body = response.body as any;
      expect(body.project_id).toBeDefined();
      expect(response.body.name).toBe(newProject.name);
      expect(response.body.timeline).toBeDefined();
      expect(response.body.timeline.frame_rate).toBeDefined();
    });

    it("should reject project creation with invalid frame rate", async () => {
      const newProject = {
        name: "Test Project",
        frame_rate: { numerator: 0, denominator: 1 }, // Invalid: not positive
      };

      const response = await request(app)
        .post("/api/projects")
        .send(newProject);

      expect(response.status).toBe(400);
      expect(response.body.error).toBeDefined();
    });

    it("should list all projects", async () => {
      // Create multiple projects
      const project1 = {
        name: "Project 1",
        frame_rate: { numerator: 24, denominator: 1 },
      };
      const project2 = {
        name: "Project 2",
        frame_rate: { numerator: 30, denominator: 1 },
      };

      await request(app).post("/api/projects").send(project1);
      await request(app).post("/api/projects").send(project2);

      const response = await request(app).get("/api/projects");

      expect(response.status).toBe(200);
      expect(response.body.projects).toHaveLength(2);
      expect(response.body.count).toBe(2);
    });

    it("should get a specific project", async () => {
      const newProject = {
        name: "Test Project",
        frame_rate: { numerator: 24, denominator: 1 },
      };

      const createResponse = await request(app)
        .post("/api/projects")
        .send(newProject);

      const projectId = createResponse.body.project_id;

      const getResponse = await request(app).get(`/api/projects/${projectId}`);

      expect(getResponse.status).toBe(200);
      expect(getResponse.body.project_id).toBe(projectId);
      expect(getResponse.body.name).toBe(newProject.name);
    });

    it("should return 404 for non-existent project", async () => {
      const response = await request(app).get("/api/projects/non-existent-id");

      expect(response.status).toBe(404);
      expect(response.body.error).toBe("Project not found");
    });

    it("should update a project", async () => {
      const newProject = {
        name: "Test Project",
        frame_rate: { numerator: 24, denominator: 1 },
      };

      const createResponse = await request(app)
        .post("/api/projects")
        .send(newProject);

      const projectId = createResponse.body.project_id;

      const updatedTimeline = {
        ...createResponse.body.timeline,
        layers: [],
      };

      const updateResponse = await request(app)
        .put(`/api/projects/${projectId}`)
        .send({ timeline: updatedTimeline });

      expect(updateResponse.status).toBe(200);
      expect(updateResponse.body.version).toBe(2);
    });

    it("should delete a project", async () => {
      const newProject = {
        name: "Test Project",
        frame_rate: { numerator: 24, denominator: 1 },
      };

      const createResponse = await request(app)
        .post("/api/projects")
        .send(newProject);

      const projectId = createResponse.body.project_id;

      const deleteResponse = await request(app).delete(`/api/projects/${projectId}`);

      expect(deleteResponse.status).toBe(204);

      // Verify project is gone
      const getResponse = await request(app).get(`/api/projects/${projectId}`);
      expect(getResponse.status).toBe(404);
    });
  });

  describe("Timeline Query", () => {
    it("should query active clips at a specific time", async () => {
      // Create a project
      const newProject = {
        name: "Test Project",
        frame_rate: { numerator: 24, denominator: 1 },
      };

      const createResponse = await request(app)
        .post("/api/projects")
        .send(newProject);

      const projectId = createResponse.body.project_id;

      // Query timeline at time 0
      const queryResponse = await request(app)
        .get(`/api/projects/${projectId}/timeline/query`)
        .query({ numerator: "0", denominator: "1" });

      expect(queryResponse.status).toBe(200);
      expect(queryResponse.body.time).toEqual({ numerator: 0, denominator: 1 });
      expect(queryResponse.body.active_clips).toBeInstanceOf(Array);
    });

    it("should reject timeline query with missing parameters", async () => {
      const newProject = {
        name: "Test Project",
        frame_rate: { numerator: 24, denominator: 1 },
      };

      const createResponse = await request(app)
        .post("/api/projects")
        .send(newProject);

      const projectId = createResponse.body.project_id;

      const queryResponse = await request(app)
        .get(`/api/projects/${projectId}/timeline/query`);

      expect(queryResponse.status).toBe(400);
      expect(queryResponse.body.error).toBeDefined();
    });
  });

  describe("Drafts", () => {
    it("should create a draft for a project", async () => {
      const newProject = {
        name: "Test Project",
        frame_rate: { numerator: 24, denominator: 1 },
      };

      const createResponse = await request(app)
        .post("/api/projects")
        .send(newProject);

      const projectId = createResponse.body.project_id;

      const draftData = {
        name: "Draft 1",
        timeline: createResponse.body.timeline,
      };

      const draftResponse = await request(app)
        .post(`/api/projects/${projectId}/drafts`)
        .send(draftData);

      expect(draftResponse.status).toBe(201);
      expect(draftResponse.body.draft_id).toBeDefined();
      expect(draftResponse.body.name).toBe(draftData.name);
    });

    it("should list drafts for a project", async () => {
      const newProject = {
        name: "Test Project",
        frame_rate: { numerator: 24, denominator: 1 },
      };

      const createResponse = await request(app)
        .post("/api/projects")
        .send(newProject);

      const projectId = createResponse.body.project_id;

      const draftData = {
        name: "Draft 1",
        timeline: createResponse.body.timeline,
      };

      await request(app)
        .post(`/api/projects/${projectId}/drafts`)
        .send(draftData);

      const listResponse = await request(app).get(`/api/projects/${projectId}/drafts`);

      expect(listResponse.status).toBe(200);
      expect(listResponse.body.drafts).toHaveLength(1);
      expect(listResponse.body.count).toBe(1);
    });

    it("should delete a draft", async () => {
      const newProject = {
        name: "Test Project",
        frame_rate: { numerator: 24, denominator: 1 },
      };

      const createResponse = await request(app)
        .post("/api/projects")
        .send(newProject);

      const projectId = createResponse.body.project_id;

      const draftData = {
        name: "Draft 1",
        timeline: createResponse.body.timeline,
      };

      const draftResponse = await request(app)
        .post(`/api/projects/${projectId}/drafts`)
        .send(draftData);

      const draftId = draftResponse.body.draft_id;

      const deleteResponse = await request(app).delete(`/api/drafts/${draftId}`);

      expect(deleteResponse.status).toBe(204);
    });
  });
});
