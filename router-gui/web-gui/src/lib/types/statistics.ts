/**
 * Statistics type definitions
 */

/**
 * Target type for statistics (domain or proxy)
 */
export type StatisticsTarget = 'domain' | 'proxy';

/**
 * Represents a data point in the time series
 */
export interface StatisticsDataPoint {
    /** ISO-8601 formatted timestamp for the data point */
    date_time: string;
    /** Value - depends on context (e.g., failed/unmatched requests for default stats) */
    value: number;
    /** High value - depends on context (e.g., response count for default stats) */
    high: number;
    /** Low value - depends on context (e.g., request count for default stats) */
    low: number;
}

/**
 * Request parameters for default statistics
 */
export interface DefaultStatisticsRequest {
    /** Data source: "domain" (default) or "proxy" */
    target?: StatisticsTarget;
}

/**
 * Request parameters for status code statistics
 */
export interface StatusCodeStatisticsRequest {
    /** HTTP status code to filter by */
    status: string;
    /** Data source: "domain" (default) or "proxy" */
    target?: StatisticsTarget;
}

/**
 * Request parameters for bytes statistics
 */
export interface BytesStatisticsRequest {
    /** Data source: "domain" (default) or "proxy" */
    target?: StatisticsTarget;
}
