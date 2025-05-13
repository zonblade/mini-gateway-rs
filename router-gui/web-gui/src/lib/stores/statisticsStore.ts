import { writable } from 'svelte/store';
import type { StatisticsDataPoint } from '$lib/types/statistics';

// Create writable stores for different types of statistics data
export const defaultStatistics = writable<StatisticsDataPoint[]>([]);
export const statusCodeStatistics = writable<StatisticsDataPoint[]>([]);
export const bytesStatistics = writable<StatisticsDataPoint[]>([]);

// Store for current status code being viewed
export const currentStatusCode = writable<string>('');

// Store for current target (domain or proxy)
export const currentTarget = writable<'domain' | 'proxy'>('domain');

// Helper function to find the max value in the data set
export function getMaxValue(data: StatisticsDataPoint[]): number {
    if (data.length === 0) return 0;
    return Math.max(...data.map(point => Math.max(point.value, point.high, point.low)));
}

// Helper function to find the min value in the data set
export function getMinValue(data: StatisticsDataPoint[]): number {
    if (data.length === 0) return 0;
    return Math.min(...data.map(point => Math.min(point.value, point.high, point.low)));
}

// Helper function to get the most recent data point
export function getMostRecentDataPoint(data: StatisticsDataPoint[]): StatisticsDataPoint | null {
    if (data.length === 0) return null;
    return data[data.length - 1];
}

// Helper function to get the average value over all data points
export function getAverageValue(data: StatisticsDataPoint[]): number {
    if (data.length === 0) return 0;
    const sum = data.reduce((total, point) => total + point.value, 0);
    return sum / data.length;
}
