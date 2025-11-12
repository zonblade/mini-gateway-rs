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
    },

    /**
     * Create SSE connection for real-time statistics updates
     * @param statsType Type of statistics: "default", "bytes", or "status"
     * @param target Optional data source: "domain" (default) or "proxy"
     * @param status Optional HTTP status code (required when statsType is "status")
     * @param onData Callback function to handle incoming data
     * @param onError Callback function to handle errors
     * @returns Function to close the SSE connection
     */
    createSSEConnection(
        statsType: string,
        target?: StatisticsTarget,
        status?: string,
        onData?: (data: StatisticsDataPoint) => void,
        onError?: (error: Event) => void
    ): () => void {
        const baseUrl = getApiBaseUrl();
        const targetParam = target ? `&target=${target}` : '';
        const statusParam = status ? `&status=${status}` : '';
        
        // Validate required parameters
        if (statsType === 'status' && !status) {
            throw new Error('Status parameter is required for status statistics');
        }
        
        const url = `${baseUrl}/statistics/events?type=${statsType}${targetParam}${statusParam}`;
        
        // Create EventSource with auth headers if available
        const token = getAuthToken();
        const eventSource = new EventSource(url);
        
        // Add auth header if token exists (Note: EventSource doesn't support custom headers directly)
        // We'll need to handle auth via query parameter or cookies in production
        
        eventSource.onmessage = (event) => {
            try {
                const parsed = JSON.parse(event.data);
                if (parsed.type === 'statistics_update' && parsed.data && onData) {
                    onData(parsed.data);
                }
            } catch (error) {
                console.error('Error parsing SSE data:', error);
                if (onError) {
                    onError(event);
                }
            }
        };

        eventSource.onerror = (event) => {
            console.error('SSE connection error:', event);
            if (onError) {
                onError(event);
            }
        };

        eventSource.onopen = () => {
            console.log(`SSE connected for ${statsType} statistics`);
        };

        // Return cleanup function
        return () => {
            console.log(`Closing SSE connection for ${statsType} statistics`);
            eventSource.close();
        };
    }
};
