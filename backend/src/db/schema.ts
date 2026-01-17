/**
 * Database schema for Motion Ogre projects.
 */

import SqliteDatabase from "better-sqlite3";
import { v4 as uuidv4 } from "uuid";
import path from "path";

const DB_PATH = process.env.DB_PATH || path.join(__dirname, "../../data/motion-ogre.db");

export interface ProjectRecord {
  id: string;
  name: string;
  created_at: number;
  updated_at: number;
  timeline_json: string;
  version: number;
}

export interface DraftRecord {
  id: string;
  project_id: string;
  name: string;
  created_at: number;
  timeline_json: string;
}

export class Database {
  private db: SqliteDatabase.Database;

  constructor(dbPath: string = DB_PATH) {
    this.db = new SqliteDatabase(dbPath);
    this.db.pragma("journal_mode = WAL");
    this.initSchema();
  }

  private initSchema(): void {
    // Projects table
    this.db.exec(`
      CREATE TABLE IF NOT EXISTS projects (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        created_at INTEGER NOT NULL,
        updated_at INTEGER NOT NULL,
        timeline_json TEXT NOT NULL,
        version INTEGER NOT NULL DEFAULT 1
      )
    `);

    // Drafts table
    this.db.exec(`
      CREATE TABLE IF NOT EXISTS drafts (
        id TEXT PRIMARY KEY,
        project_id TEXT NOT NULL,
        name TEXT NOT NULL,
        created_at INTEGER NOT NULL,
        timeline_json TEXT NOT NULL,
        FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
      )
    `);

    // Indexes
    this.db.exec(`
      CREATE INDEX IF NOT EXISTS idx_projects_created_at ON projects(created_at)
    `);

    this.db.exec(`
      CREATE INDEX IF NOT EXISTS idx_drafts_project_id ON drafts(project_id)
    `);
  }

  // Project operations
  createProject(name: string, timelineJson: string): ProjectRecord {
    const id = uuidv4();
    const now = Date.now();

    const stmt = this.db.prepare(`
      INSERT INTO projects (id, name, created_at, updated_at, timeline_json, version)
      VALUES (?, ?, ?, ?, ?, 1)
    `);

    stmt.run(id, name, now, now, timelineJson);

    return this.getProject(id)!;
  }

  getProject(id: string): ProjectRecord | null {
    const stmt = this.db.prepare("SELECT * FROM projects WHERE id = ?");
    const row = stmt.get(id) as any;
    return row || null;
  }

  listProjects(limit: number = 100, offset: number = 0): ProjectRecord[] {
    const stmt = this.db.prepare(`
      SELECT * FROM projects ORDER BY updated_at DESC LIMIT ? OFFSET ?
    `);
    return stmt.all(limit, offset) as ProjectRecord[];
  }

  updateProject(id: string, timelineJson: string): ProjectRecord | null {
    const now = Date.now();

    const stmt = this.db.prepare(`
      UPDATE projects
      SET updated_at = ?, timeline_json = timeline_json || ?, version = version + 1
      WHERE id = ?
    `);

    const result = stmt.run(now, timelineJson, id);
    if (result.changes === 0) return null;

    return this.getProject(id);
  }

  deleteProject(id: string): boolean {
    const stmt = this.db.prepare("DELETE FROM projects WHERE id = ?");
    const result = stmt.run(id);
    return result.changes > 0;
  }

  // Draft operations
  createDraft(projectId: string, name: string, timelineJson: string): DraftRecord {
    const id = uuidv4();
    const now = Date.now();

    const stmt = this.db.prepare(`
      INSERT INTO drafts (id, project_id, name, created_at, timeline_json)
      VALUES (?, ?, ?, ?, ?)
    `);

    stmt.run(id, projectId, name, now, timelineJson);

    return this.getDraft(id)!;
  }

  getDraft(id: string): DraftRecord | null {
    const stmt = this.db.prepare("SELECT * FROM drafts WHERE id = ?");
    const row = stmt.get(id) as any;
    return row || null;
  }

  listDrafts(projectId: string): DraftRecord[] {
    const stmt = this.db.prepare(`
      SELECT * FROM drafts WHERE project_id = ? ORDER BY created_at DESC
    `);
    return stmt.all(projectId) as DraftRecord[];
  }

  deleteDraft(id: string): boolean {
    const stmt = this.db.prepare("DELETE FROM drafts WHERE id = ?");
    const result = stmt.run(id);
    return result.changes > 0;
  }

  close(): void {
    this.db.close();
  }

  // Cleanup old drafts (older than 7 days)
  cleanupOldDrafts(daysOld: number = 7): number {
    const cutoff = Date.now() - daysOld * 24 * 60 * 60 * 1000;
    const stmt = this.db.prepare("DELETE FROM drafts WHERE created_at < ?");
    const result = stmt.run(cutoff);
    return result.changes;
  }
}

// Singleton instance
let dbInstance: Database | null = null;

export function getDatabase(): Database {
  if (!dbInstance) {
    dbInstance = new Database();
  }
  return dbInstance;
}

export function resetDatabase(): void {
  if (dbInstance) {
    dbInstance.close();
    dbInstance = null;
  }
}
