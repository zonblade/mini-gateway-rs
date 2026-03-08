import { writable, derived } from 'svelte/store';

/** Statistics for a single target (gateway or proxy) */
export interface TargetStats {
    req: number;
    res: number;
    bytes_in: number;
    bytes_out: number;
    status: Record<string, number>;
    failed: number;
    bytes_in_min: number;
    bytes_in_max: number;
    bytes_in_avg: number;
    bytes_out_min: number;
    bytes_out_max: number;
    bytes_out_avg: number;
    stalled_count: number;
}

/** A single data point from SSE */
export interface StatsDataPoint {
    ts: string;
    gateway: TargetStats;
    proxy: TargetStats;
}

/** Display interval options */
export type DisplayInterval = 'live' | '30s' | '1m' | '5m';

// Raw buffer (all incoming SSE data - last 5 minutes)
export const rawBuffer = writable<StatsDataPoint[]>([]);

// User-selected interval
export const displayInterval = writable<DisplayInterval>('live');

// Connection status
export const connectionStatus = writable<'connected' | 'disconnected'>('disconnected');

// Selected status code for graph
export const selectedStatusCode = writable<string>('200');

// Add data point to buffer (keeps last 120 points = 30 min at 15s interval)
export function addDataPoint(point: StatsDataPoint): void {
    rawBuffer.update(buf => {
        buf.push(point);
        if (buf.length > 120) buf.shift();
        return buf;
    });
}

// Clear buffer
export function clearBuffer(): void {
    rawBuffer.set([]);
}

// Get sparkline data for a metric (last 20 points = 5 minutes at 15s intervals)
export function getSparklineData(
    buffer: StatsDataPoint[],
    target: 'gateway' | 'proxy',
    metric: 'req' | 'res' | 'bytes_in' | 'bytes_out' | 'failed' | 'stalled_count'
): number[] {
    return buffer.slice(-20).map(p => p[target][metric]);
}

// Get status code count for sparkline
export function getStatusSparklineData(
    buffer: StatsDataPoint[],
    target: 'gateway' | 'proxy',
    statusCode: string
): number[] {
    return buffer.slice(-40).map(p => p[target].status[statusCode] || 0);
}

// Aggregate stats based on interval
export function aggregateStats(
    buffer: StatsDataPoint[],
    interval: DisplayInterval
): StatsDataPoint | null {
    if (buffer.length === 0) return null;

    const count = interval === 'live' ? 1
        : interval === '30s' ? 2
        : interval === '1m' ? 4
        : 20; // 5m = 20 * 15s
    const slice = buffer.slice(-count);

    if (slice.length === 0) return null;

    // Sum all values in the slice
    const result: StatsDataPoint = {
        ts: slice[slice.length - 1].ts,
        gateway: { req: 0, res: 0, bytes_in: 0, bytes_out: 0, status: {}, failed: 0, bytes_in_min: 0, bytes_in_max: 0, bytes_in_avg: 0, bytes_out_min: 0, bytes_out_max: 0, bytes_out_avg: 0, stalled_count: 0 },
        proxy: { req: 0, res: 0, bytes_in: 0, bytes_out: 0, status: {}, failed: 0, bytes_in_min: 0, bytes_in_max: 0, bytes_in_avg: 0, bytes_out_min: 0, bytes_out_max: 0, bytes_out_avg: 0, stalled_count: 0 }
    };

    for (const point of slice) {
        // Gateway
        result.gateway.req += point.gateway.req;
        result.gateway.res += point.gateway.res;
        result.gateway.bytes_in += point.gateway.bytes_in;
        result.gateway.bytes_out += point.gateway.bytes_out;
        for (const [code, count] of Object.entries(point.gateway.status)) {
            result.gateway.status[code] = (result.gateway.status[code] || 0) + count;
        }
        // Gateway new fields
        result.gateway.failed += point.gateway.failed;
        result.gateway.stalled_count += point.gateway.stalled_count;
        // For min/max/avg, just take the latest values (they're already calculated server-side per interval)
        result.gateway.bytes_in_min = point.gateway.bytes_in_min;
        result.gateway.bytes_in_max = point.gateway.bytes_in_max;
        result.gateway.bytes_in_avg = point.gateway.bytes_in_avg;
        result.gateway.bytes_out_min = point.gateway.bytes_out_min;
        result.gateway.bytes_out_max = point.gateway.bytes_out_max;
        result.gateway.bytes_out_avg = point.gateway.bytes_out_avg;

        // Proxy
        result.proxy.req += point.proxy.req;
        result.proxy.res += point.proxy.res;
        result.proxy.bytes_in += point.proxy.bytes_in;
        result.proxy.bytes_out += point.proxy.bytes_out;
        for (const [code, count] of Object.entries(point.proxy.status)) {
            result.proxy.status[code] = (result.proxy.status[code] || 0) + count;
        }
        // Proxy new fields (same pattern)
        result.proxy.failed += point.proxy.failed;
        result.proxy.stalled_count += point.proxy.stalled_count;
        result.proxy.bytes_in_min = point.proxy.bytes_in_min;
        result.proxy.bytes_in_max = point.proxy.bytes_in_max;
        result.proxy.bytes_in_avg = point.proxy.bytes_in_avg;
        result.proxy.bytes_out_min = point.proxy.bytes_out_min;
        result.proxy.bytes_out_max = point.proxy.bytes_out_max;
        result.proxy.bytes_out_avg = point.proxy.bytes_out_avg;
    }

    return result;
}

// Derived store: aggregated stats based on current interval
export const aggregatedStats = derived(
    [rawBuffer, displayInterval],
    ([$buffer, $interval]) => aggregateStats($buffer, $interval)
);

// Format bytes to human readable
export function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
}

// Format number with comma separators
export function formatNumber(num: number): string {
    return num.toLocaleString();
}
