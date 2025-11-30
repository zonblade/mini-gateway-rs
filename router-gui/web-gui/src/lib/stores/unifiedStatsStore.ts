import { writable, derived } from 'svelte/store';

/** Statistics for a single target (gateway or proxy) */
export interface TargetStats {
    req: number;
    res: number;
    bytes_in: number;
    bytes_out: number;
    status: Record<string, number>;
}

/** A single data point from SSE */
export interface StatsDataPoint {
    ts: string;
    gateway: TargetStats;
    proxy: TargetStats;
}

/** Display interval options */
export type DisplayInterval = 'live' | '5s' | '10s' | '15s';

// Raw buffer (all incoming SSE data - last 5 minutes)
export const rawBuffer = writable<StatsDataPoint[]>([]);

// User-selected interval
export const displayInterval = writable<DisplayInterval>('live');

// Connection status
export const connectionStatus = writable<'connected' | 'disconnected'>('disconnected');

// Selected status code for graph
export const selectedStatusCode = writable<string>('200');

// Add data point to buffer (keeps last 300 points = 5 min at 1s interval)
export function addDataPoint(point: StatsDataPoint): void {
    rawBuffer.update(buf => {
        buf.push(point);
        if (buf.length > 300) buf.shift();
        return buf;
    });
}

// Clear buffer
export function clearBuffer(): void {
    rawBuffer.set([]);
}

// Get sparkline data for a metric (last 60 points = 60 seconds)
export function getSparklineData(
    buffer: StatsDataPoint[],
    target: 'gateway' | 'proxy',
    metric: 'req' | 'res' | 'bytes_in' | 'bytes_out'
): number[] {
    return buffer.slice(-60).map(p => p[target][metric]);
}

// Get status code count for sparkline
export function getStatusSparklineData(
    buffer: StatsDataPoint[],
    target: 'gateway' | 'proxy',
    statusCode: string
): number[] {
    return buffer.slice(-120).map(p => p[target].status[statusCode] || 0);
}

// Aggregate stats based on interval
export function aggregateStats(
    buffer: StatsDataPoint[],
    interval: DisplayInterval
): StatsDataPoint | null {
    if (buffer.length === 0) return null;

    const count = interval === 'live' ? 1 : parseInt(interval);
    const slice = buffer.slice(-count);

    if (slice.length === 0) return null;

    // Sum all values in the slice
    const result: StatsDataPoint = {
        ts: slice[slice.length - 1].ts,
        gateway: { req: 0, res: 0, bytes_in: 0, bytes_out: 0, status: {} },
        proxy: { req: 0, res: 0, bytes_in: 0, bytes_out: 0, status: {} }
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

        // Proxy
        result.proxy.req += point.proxy.req;
        result.proxy.res += point.proxy.res;
        result.proxy.bytes_in += point.proxy.bytes_in;
        result.proxy.bytes_out += point.proxy.bytes_out;
        for (const [code, count] of Object.entries(point.proxy.status)) {
            result.proxy.status[code] = (result.proxy.status[code] || 0) + count;
        }
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
