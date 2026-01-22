/**
 * Structured logging utilities using Pino.
 */

import pino from "pino";

export type LogLevel = "fatal" | "error" | "warn" | "info" | "debug" | "trace";

export interface LogContext {
  [key: string]: unknown;
}

export class Logger {
  private pino: pino.Logger;

  constructor(name: string, level: LogLevel = "info") {
    this.pino = pino({
      name,
      level,
      formatters: {
        level: (label) => {
          return { level: label };
        },
      },
      timestamp: pino.stdTimeFunctions.isoTime,
      serializers: {
        err: pino.stdSerializers.err,
      },
    });
  }

  private formatContext(context?: LogContext): LogContext {
    return {
      ...(context || {}),
      timestamp: new Date().toISOString(),
    };
  }

  fatal(message: string, context?: LogContext): void {
    this.pino.fatal(this.formatContext(context), message);
  }

  error(message: string, error?: Error | string | LogContext, additionalContext?: LogContext): void {
    if (error instanceof Error) {
      this.pino.error(
        {
          ...this.formatContext(additionalContext),
          err: pino.stdSerializers.err(error),
        },
        message
      );
    } else if (typeof error === "string") {
      this.pino.error(this.formatContext(additionalContext), message, error);
    } else {
      this.pino.error(this.formatContext({ ...additionalContext, ...error }), message);
    }
  }

  warn(message: string, context?: LogContext): void {
    this.pino.warn(this.formatContext(context), message);
  }

  info(message: string, context?: LogContext): void {
    this.pino.info(this.formatContext(context), message);
  }

  debug(message: string, context?: LogContext): void {
    this.pino.debug(this.formatContext(context), message);
  }

  trace(message: string, context?: LogContext): void {
    this.pino.trace(this.formatContext(context), message);
  }

  http(message: string, context?: LogContext): void {
    this.pino.info(this.formatContext(context), message);
  }

  child(name: string): Logger {
    const childLogger = new Logger(name, this.pino.level as LogLevel);
    childLogger.pino = this.pino.child({ name });
    return childLogger;
  }
}

// Singleton instances
const loggers = new Map<string, Logger>();

export function getLogger(name: string, level?: LogLevel): Logger {
  if (!loggers.has(name)) {
    loggers.set(name, new Logger(name, level));
  }
  return loggers.get(name)!;
}

export function setLogLevel(level: LogLevel): void {
  for (const logger of loggers.values()) {
    const log = logger as any;
    log.pino.level = level;
  }
}
