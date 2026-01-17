/**
 * Python agent process bridge.
 * 
 * This module manages spawning, communicating with, and collecting results
 * from Python agent processes.
 */

import { spawn, ChildProcess } from "child_process";
import path from "path";
import { EventEmitter } from "events";
import { Logger } from "../utils/logger";

interface AgentExecutionRequest {
  id: string;
  agent_type: string;
  intent: string;
  context?: Record<string, unknown>;
  timeline_json?: string;
}

interface AgentExecutionResult {
  id: string;
  agent_type: string;
  success: boolean;
  result?: unknown;
  error?: string;
  execution_time_ms: number;
}

interface AgentProgressUpdate {
  id: string;
  agent_type: string;
  progress: number;
  message: string;
}

export class PythonAgentBridge extends EventEmitter {
  private agentProcesses: Map<string, ChildProcess>;
  private activeExecutions: Map<string, AgentExecutionRequest>;
  private logger: Logger;
  private pythonPath: string;
  private agentScriptPath: string;

  constructor(logger: Logger) {
    super();
    this.agentProcesses = new Map();
    this.activeExecutions = new Map();
    this.logger = logger;
    this.pythonPath = process.env.PYTHON_PATH || "python3";
    this.agentScriptPath = path.join(
      __dirname,
      "../../../src/motion_ogre_agents/bridge.py"
    );
  }

  /**
   * Execute an agent request.
   */
  async executeAgent(request: AgentExecutionRequest): Promise<AgentExecutionResult> {
    const startTime = Date.now();
    this.activeExecutions.set(request.id, request);

    this.logger.info("Starting agent execution", {
      execution_id: request.id,
      agent_type: request.agent_type,
      intent: request.intent,
    });

    return new Promise((resolve) => {
      const args = [
        this.agentScriptPath,
        "--agent-type",
        request.agent_type,
        "--intent",
        request.intent,
        "--execution-id",
        request.id,
      ];

      if (request.context) {
        args.push("--context", JSON.stringify(request.context));
      }

      if (request.timeline_json) {
        args.push("--timeline", request.timeline_json);
      }

      const childProcess = spawn(this.pythonPath, args, {
        stdio: ["pipe", "pipe", "pipe"],
        env: {
          ...process.env,
          PYTHONPATH: path.join(__dirname, "../../../src"),
        },
      });

      this.agentProcesses.set(request.id, childProcess);

      let stdout = "";
      let stderr = "";

      childProcess.stdout.on("data", (data: Buffer) => {
        const chunk = data.toString();
        stdout += chunk;

        // Try to parse progress updates
        try {
          const lines = chunk.trim().split("\n");
          for (const line of lines) {
            if (line.startsWith("PROGRESS:")) {
              const progressJson = line.substring("PROGRESS:".length);
              const progress = JSON.parse(progressJson) as AgentProgressUpdate;
              this.emit("progress", progress);
            }
          }
        } catch (e) {
          // Ignore JSON parse errors for non-progress output
        }
      });

      childProcess.stderr.on("data", (data: Buffer) => {
        stderr += data.toString();
      });

      childProcess.on("close", (code: number | null) => {
        this.agentProcesses.delete(request.id);
        this.activeExecutions.delete(request.id);

        const executionTime = Date.now() - startTime;

        if (code === 0) {
          try {
            const result = JSON.parse(stdout) as unknown;
            this.logger.info("Agent execution completed successfully", {
              execution_id: request.id,
              agent_type: request.agent_type,
              execution_time_ms: executionTime,
            });

            resolve({
              id: request.id,
              agent_type: request.agent_type,
              success: true,
              result,
              execution_time_ms: executionTime,
            });
          } catch (e) {
            this.logger.error("Failed to parse agent output", {
              execution_id: request.id,
              error: String(e),
              stdout: stdout.substring(0, 500),
            });

            resolve({
              id: request.id,
              agent_type: request.agent_type,
              success: false,
              error: `Failed to parse agent output: ${String(e)}`,
              execution_time_ms: executionTime,
            });
          }
        } else {
          this.logger.error("Agent execution failed", {
            execution_id: request.id,
            agent_type: request.agent_type,
            exit_code: code,
            stderr: stderr.substring(0, 500),
          });

          resolve({
            id: request.id,
            agent_type: request.agent_type,
            success: false,
            error: `Agent process exited with code ${code}: ${stderr}`,
            execution_time_ms: executionTime,
          });
        }
      });

      childProcess.on("error", (error: Error) => {
        this.agentProcesses.delete(request.id);
        this.activeExecutions.delete(request.id);

        const executionTime = Date.now() - startTime;

        this.logger.error("Agent process error", {
          execution_id: request.id,
          agent_type: request.agent_type,
          error: String(error),
        });

        resolve({
          id: request.id,
          agent_type: request.agent_type,
          success: false,
          error: `Process error: ${String(error)}`,
          execution_time_ms: executionTime,
        });
      });
    });
  }

  /**
   * Cancel an active agent execution.
   */
  cancelExecution(executionId: string): boolean {
    const childProcess = this.agentProcesses.get(executionId);
    if (!childProcess) return false;

    childProcess.kill("SIGTERM");
    this.agentProcesses.delete(executionId);
    this.activeExecutions.delete(executionId);

    this.logger.info("Agent execution cancelled", { execution_id: executionId });
    return true;
  }

  /**
   * Get list of active executions.
   */
  getActiveExecutions(): AgentExecutionRequest[] {
    return Array.from(this.activeExecutions.values());
  }

  /**
   * Cleanup all processes.
   */
  cleanup(): void {
    for (const [id, childProcess] of this.agentProcesses) {
      childProcess.kill("SIGTERM");
    }
    this.agentProcesses.clear();
    this.activeExecutions.clear();
  }

  /**
   * Check if Python is available.
   */
  async checkPythonAvailable(): Promise<boolean> {
    return new Promise((resolve) => {
      const childProcess = spawn(this.pythonPath, ["--version"], {
        stdio: ["ignore", "pipe", "pipe"],
      });

      childProcess.on("close", (code: number | null) => {
        resolve(code === 0);
      });

      childProcess.on("error", () => {
        resolve(false);
      });
    });
  }
}
