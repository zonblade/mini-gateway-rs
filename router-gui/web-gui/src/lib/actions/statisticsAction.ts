import { statisticsService } from "$lib/services/statisticsService";
import { defaultStatistics, statusCodeStatistics, bytesStatistics, currentStatusCode, currentTarget } from "$lib/stores/statisticsStore";
import type { StatisticsDataPoint, StatisticsTarget } from "$lib/types/statistics";

/**
 * Actions for fetching statistics and updating the stores
 */
export const statisticsActions = {
    /**
     * Load default statistics (request/response counts)
     * @param target Optional data source: "domain" (default) or "proxy"
     * @returns Array of statistics data points
     */
    async loadDefaultStatistics(target?: StatisticsTarget): Promise<StatisticsDataPoint[]> {
        try {
            const data = await statisticsService.getDefaultStatistics(target);
            defaultStatistics.set(data);
            
            if (target) {
                currentTarget.set(target);
            }
            
            return data;
        } catch (error) {
            console.error("Failed to load default statistics:", error);
            throw error;
        }
    },

    /**
     * Load statistics for a specific HTTP status code
     * @param status HTTP status code to filter by
     * @param target Optional data source: "domain" (default) or "proxy"
     * @returns Array of statistics data points
     */
    async loadStatusCodeStatistics(status: string, target?: StatisticsTarget): Promise<StatisticsDataPoint[]> {
        try {
            const data = await statisticsService.getStatusCodeStatistics(status, target);
            statusCodeStatistics.set(data);
            currentStatusCode.set(status);
            
            if (target) {
                currentTarget.set(target);
            }
            
            return data;
        } catch (error) {
            console.error(`Failed to load status code statistics for status ${status}:`, error);
            throw error;
        }
    },

    /**
     * Load bytes statistics (data transfer)
     * @param target Optional data source: "domain" (default) or "proxy"
     * @returns Array of statistics data points
     */
    async loadBytesStatistics(target?: StatisticsTarget): Promise<StatisticsDataPoint[]> {
        try {
            const data = await statisticsService.getBytesStatistics(target);
            bytesStatistics.set(data);
            
            if (target) {
                currentTarget.set(target);
            }
            
            return data;
        } catch (error) {
            console.error("Failed to load bytes statistics:", error);
            throw error;
        }
    },

    /**
     * Set the current target for statistics
     * @param target Target to set (domain or proxy)
     */
    setCurrentTarget(target: StatisticsTarget): void {
        currentTarget.set(target);
    },

    /**
     * Get the current target from the store
     * @returns Current target value
     */
    getCurrentTarget(): StatisticsTarget {
        let target: StatisticsTarget = 'domain';
        currentTarget.subscribe(value => {
            target = value;
        })();
        return target;
    },

    /**
     * Setup polling to automatically refresh statistics data
     * @param intervalMs Polling interval in milliseconds (default: 15000 = 15 seconds)
     * @returns Function to stop polling
     */
    setupPolling(intervalMs: number = 15000): () => void {
        // Start initial data loads
        this.loadDefaultStatistics(this.getCurrentTarget());
        this.loadBytesStatistics(this.getCurrentTarget());
        
        // Get current status code if any
        let status: string = '';
        currentStatusCode.subscribe(value => {
            status = value;
        })();
        
        if (status) {
            this.loadStatusCodeStatistics(status, this.getCurrentTarget());
        }
        
        // Set up polling interval
        const intervalId = setInterval(() => {
            this.loadDefaultStatistics(this.getCurrentTarget());
            this.loadBytesStatistics(this.getCurrentTarget());
            
            // Refresh status code data if we have a current status code
            currentStatusCode.subscribe(value => {
                status = value;
            })();
            
            if (status) {
                this.loadStatusCodeStatistics(status, this.getCurrentTarget());
            }
        }, intervalMs);
        
        // Return function to stop polling
        return () => clearInterval(intervalId);
    }
};
