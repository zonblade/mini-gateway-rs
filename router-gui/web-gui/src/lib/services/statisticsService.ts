import type { StatisticsDataPoint, StatisticsTarget } from "$lib/types/statistics";
import { user } from '$lib/stores/userStore';

// Helper function to get the current API base URL from the user store
function getApiBaseUrl(): string {
    let apiUrl: string = '';
    user.subscribe(value => {
        apiUrl = value?.api_base_url || '/api/v1';
    })();
    return apiUrl;
}

// Helper function to get the auth token from the store
function getAuthToken(): string | null {
    let token: string | null = null;
    user.subscribe(value => {
        token = value?.token || null;
    })();
    return token;
}

// Helper function to create request headers with auth token
function getHeaders(): Record<string, string> {
    const token = getAuthToken();
    const headers: Record<string, string> = {
        'Content-Type': 'application/json',
    };
    
    if (token) {
        headers['Authorization'] = `Bearer ${token}`;
    }
    
    return headers;
}

/**
 * Service for fetching statistics data through the API
 */
export const statisticsService = {
    /**
     * Fetch default statistics for request and response counts
     * @param target Optional data source: "domain" (default) or "proxy"
     * @returns Promise with array of statistics data points
     */
    async getDefaultStatistics(target?: StatisticsTarget): Promise<StatisticsDataPoint[]> {
        try {
            const baseUrl = getApiBaseUrl();
            const targetParam = target ? `?target=${target}` : '';
            const response = await fetch(`${baseUrl}/statistics/default${targetParam}`, {
                method: 'GET',
                headers: getHeaders()
            });

            if (!response.ok) {
                throw new Error(`Failed to fetch default statistics: ${response.statusText}`);
            }

            return await response.json();
        } catch (error) {
            console.error('Error fetching default statistics:', error);
            throw error;
        }
    },

    /**
     * Fetch statistics filtered by HTTP status code
     * @param status HTTP status code to filter by
     * @param target Optional data source: "domain" (default) or "proxy"
     * @returns Promise with array of statistics data points
     */
    async getStatusCodeStatistics(status: string, target?: StatisticsTarget): Promise<StatisticsDataPoint[]> {
        try {
            const baseUrl = getApiBaseUrl();
            const targetParam = target ? `?target=${target}` : '';
            const response = await fetch(`${baseUrl}/statistics/status/${status}${targetParam}`, {
                method: 'GET',
                headers: getHeaders()
            });

            if (!response.ok) {
                throw new Error(`Failed to fetch status code statistics: ${response.statusText}`);
            }

            return await response.json();
        } catch (error) {
            console.error(`Error fetching status code statistics for status ${status}:`, error);
            throw error;
        }
    },

    /**
     * Fetch statistics about bytes transferred
     * @param target Optional data source: "domain" (default) or "proxy"
     * @returns Promise with array of statistics data points
     */
    async getBytesStatistics(target?: StatisticsTarget): Promise<StatisticsDataPoint[]> {
        try {
            const baseUrl = getApiBaseUrl();
            const targetParam = target ? `?target=${target}` : '';
            const response = await fetch(`${baseUrl}/statistics/bytes${targetParam}`, {
                method: 'GET',
                headers: getHeaders()
            });

            if (!response.ok) {
                throw new Error(`Failed to fetch bytes statistics: ${response.statusText}`);
            }

            return await response.json();
        } catch (error) {
            console.error('Error fetching bytes statistics:', error);
            throw error;
        }
    }
};
