/**
 * Performance metrics and monitoring utilities.
 */

export class MetricsCollector {
  private metrics: Map<string, number[]>;
  private counters: Map<string, number>;
  private timers: Map<string, number>;
  private logger: any;

  constructor(logger: any) {
    this.metrics = new Map();
    this.counters = new Map();
    this.timers = new Map();
    this.logger = logger;
  }

  /**
   * Record a timing metric in milliseconds.
   */
  recordTiming(name: string, durationMs: number): void {
    if (!this.metrics.has(name)) {
      this.metrics.set(name, []);
    }
    this.metrics.get(name)!.push(durationMs);

    // Keep only last 1000 measurements
    const measurements = this.metrics.get(name)!;
    if (measurements.length > 1000) {
      measurements.shift();
    }
  }

  /**
   * Record a counter increment.
   */
  incrementCounter(name: string, delta: number = 1): void {
    const current = this.counters.get(name) || 0;
    this.counters.set(name, current + delta);
  }

  /**
   * Start a timer for a named operation.
   */
  startTimer(name: string): void {
    this.timers.set(name, Date.now());
  }

  /**
   * Stop a timer and record the duration.
   */
  stopTimer(name: string): number {
    const startTime = this.timers.get(name);
    if (!startTime) {
      this.logger.warn(`Timer '${name}' was not started`);
      return 0;
    }

    const duration = Date.now() - startTime;
    this.timers.delete(name);
    this.recordTiming(name, duration);
    return duration;
  }

  /**
   * Get statistics for a timing metric.
   */
  getTimingStats(name: string): { count: number; min: number; max: number; avg: number; p95: number; p99: number } | null {
    const measurements = this.metrics.get(name);
    if (!measurements || measurements.length === 0) {
      return null;
    }

    const sorted = [...measurements].sort((a, b) => a - b);
    const sum = sorted.reduce((acc, val) => acc + val, 0);
    const count = sorted.length;

    const p95Index = Math.floor(count * 0.95);
    const p99Index = Math.floor(count * 0.99);

    return {
      count,
      min: sorted[0],
      max: sorted[count - 1],
      avg: sum / count,
      p95: sorted[p95Index],
      p99: sorted[p99Index],
    };
  }

  /**
   * Get counter value.
   */
  getCounter(name: string): number {
    return this.counters.get(name) || 0;
  }

  /**
   * Reset a metric.
   */
  resetMetric(name: string): void {
    this.metrics.delete(name);
    this.counters.delete(name);
    this.timers.delete(name);
  }

  /**
   * Get all metrics as a plain object.
   */
  getAllMetrics(): Record<string, unknown> {
    const result: Record<string, unknown> = {};

    // Timing stats
    for (const [name, _] of this.metrics) {
      const stats = this.getTimingStats(name);
      if (stats) {
        result[`${name}_stats`] = stats;
      }
    }

    // Counters
    for (const [name, value] of this.counters) {
      result[`${name}_count`] = value;
    }

    return result;
  }

  /**
   * Reset all metrics.
   */
  resetAll(): void {
    this.metrics.clear();
    this.counters.clear();
    this.timers.clear();
  }
}

// Singleton instance
let metricsInstance: MetricsCollector | null = null;

export function getMetricsCollector(logger: any): MetricsCollector {
  if (!metricsInstance) {
    metricsInstance = new MetricsCollector(logger);
  }
  return metricsInstance;
}

export function resetMetricsCollector(): void {
  metricsInstance = null;
}
