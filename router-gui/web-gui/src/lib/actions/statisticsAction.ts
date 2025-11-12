import { statisticsService } from "$lib/services/statisticsService";
import { defaultStatistics, statusCodeStatistics, bytesStatistics, currentStatusCode, currentTarget, connectionStatus } from "$lib/stores/statisticsStore";
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
    },

    /**
     * Setup hybrid approach: API for initial load + SSE for real-time updates
     * Falls back to polling if SSE fails
     * @param intervalMs Fallback polling interval in milliseconds (default: 15000 = 15 seconds)
     * @returns Function to stop all connections
     */
    setupHybridUpdates(intervalMs: number = 15000): () => void {
        // Track active SSE connections
        let sseConnections: Array<() => void> = [];
        let fallbackPollingId: number | null = null;
        let sseFailureCount = 0;
        const maxSSEFailures = 3;

        // Initial data load via API (same as before)
        this.loadDefaultStatistics(this.getCurrentTarget());
        this.loadBytesStatistics(this.getCurrentTarget());
        
        let currentStatus: string = '';
        currentStatusCode.subscribe(value => {
            currentStatus = value;
        })();
        
        if (currentStatus) {
            this.loadStatusCodeStatistics(currentStatus, this.getCurrentTarget());
        }

        // Setup SSE connections for real-time updates
        const setupSSEConnections = () => {
            // Clear existing connections
            sseConnections.forEach(cleanup => cleanup());
            sseConnections = [];

            const currentTarget = this.getCurrentTarget();

            try {
                // Default statistics SSE
                const defaultSSE = statisticsService.createSSEConnection(
                    'default',
                    currentTarget,
                    undefined,
                    (data) => {
                        // Update store with new data point
                        defaultStatistics.update(existing => {
                            const updated = [...existing];
                            // Find existing point with same timestamp or add new one
                            const existingIndex = updated.findIndex(point => point.date_time === data.date_time);
                            if (existingIndex >= 0) {
                                updated[existingIndex] = data;
                            } else {
                                updated.push(data);
                                // Keep only recent data (last 120 minutes worth)
                                updated.sort((a, b) => new Date(a.date_time).getTime() - new Date(b.date_time).getTime());
                                if (updated.length > 480) { // 120 minutes * 4 intervals per minute
                                    updated.splice(0, updated.length - 480);
                                }
                            }
                            return updated;
                        });
                        sseFailureCount = 0; // Reset failure count on successful data
                        
                        // Update connection status
                        connectionStatus.update(status => ({
                            ...status,
                            sseConnected: true,
                            fallbackActive: false,
                            lastUpdate: new Date(),
                            errorCount: sseFailureCount
                        }));
                    },
                    (error) => {
                        console.warn('Default statistics SSE error:', error);
                        sseFailureCount++;
                        
                        // Update connection status
                        connectionStatus.update(status => ({
                            ...status,
                            sseConnected: false,
                            errorCount: sseFailureCount
                        }));
                        
                        if (sseFailureCount >= maxSSEFailures) {
                            startFallbackPolling();
                        }
                    }
                );
                sseConnections.push(defaultSSE);

                // Bytes statistics SSE
                const bytesSSE = statisticsService.createSSEConnection(
                    'bytes',
                    currentTarget,
                    undefined,
                    (data) => {
                        bytesStatistics.update(existing => {
                            const updated = [...existing];
                            const existingIndex = updated.findIndex(point => point.date_time === data.date_time);
                            if (existingIndex >= 0) {
                                updated[existingIndex] = data;
                            } else {
                                updated.push(data);
                                updated.sort((a, b) => new Date(a.date_time).getTime() - new Date(b.date_time).getTime());
                                if (updated.length > 480) {
                                    updated.splice(0, updated.length - 480);
                                }
                            }
                            return updated;
                        });
                    },
                    (error) => {
                        console.warn('Bytes statistics SSE error:', error);
                        sseFailureCount++;
                        if (sseFailureCount >= maxSSEFailures) {
                            startFallbackPolling();
                        }
                    }
                );
                sseConnections.push(bytesSSE);

                // Status code statistics SSE (if status is active)
                if (currentStatus) {
                    const statusSSE = statisticsService.createSSEConnection(
                        'status',
                        currentTarget,
                        currentStatus,
                        (data) => {
                            statusCodeStatistics.update(existing => {
                                const updated = [...existing];
                                const existingIndex = updated.findIndex(point => point.date_time === data.date_time);
                                if (existingIndex >= 0) {
                                    updated[existingIndex] = data;
                                } else {
                                    updated.push(data);
                                    updated.sort((a, b) => new Date(a.date_time).getTime() - new Date(b.date_time).getTime());
                                    if (updated.length > 480) {
                                        updated.splice(0, updated.length - 480);
                                    }
                                }
                                return updated;
                            });
                        },
                        (error) => {
                            console.warn('Status statistics SSE error:', error);
                            sseFailureCount++;
                            if (sseFailureCount >= maxSSEFailures) {
                                startFallbackPolling();
                            }
                        }
                    );
                    sseConnections.push(statusSSE);
                }

                console.log(`SSE connections established for target: ${currentTarget}${currentStatus ? ', status: ' + currentStatus : ''}`);
                
                // Update connection status to indicate SSE is connected
                connectionStatus.update(status => ({
                    ...status,
                    sseConnected: true,
                    fallbackActive: false,
                    errorCount: 0
                }));

            } catch (error) {
                console.error('Failed to establish SSE connections:', error);
                connectionStatus.update(status => ({
                    ...status,
                    sseConnected: false,
                    errorCount: sseFailureCount + 1
                }));
                startFallbackPolling();
            }
        };

        // Fallback to polling if SSE fails
        const startFallbackPolling = () => {
            if (fallbackPollingId) return; // Already polling

            console.log('Starting fallback polling due to SSE failures');
            
            // Update connection status to indicate fallback is active
            connectionStatus.update(status => ({
                ...status,
                sseConnected: false,
                fallbackActive: true,
                errorCount: sseFailureCount
            }));
            
            // Clear SSE connections
            sseConnections.forEach(cleanup => cleanup());
            sseConnections = [];

            // Start traditional polling
            fallbackPollingId = setInterval(() => {
                const target = this.getCurrentTarget();
                this.loadDefaultStatistics(target);
                this.loadBytesStatistics(target);
                
                let status: string = '';
                currentStatusCode.subscribe(value => {
                    status = value;
                })();
                
                if (status) {
                    this.loadStatusCodeStatistics(status, target);
                }
            }, intervalMs);
        };

        // Watch for target/status changes and restart SSE connections
        const targetUnsubscribe = currentTarget.subscribe(() => {
            if (sseConnections.length > 0) {
                console.log('Target changed, restarting SSE connections');
                setupSSEConnections();
            }
        });

        const statusUnsubscribe = currentStatusCode.subscribe(() => {
            if (sseConnections.length > 0) {
                console.log('Status changed, restarting SSE connections');
                setupSSEConnections();
            }
        });

        // Initial SSE setup
        setupSSEConnections();

        // Return cleanup function
        return () => {
            console.log('Cleaning up hybrid statistics connections');
            
            // Clear SSE connections
            sseConnections.forEach(cleanup => cleanup());
            sseConnections = [];
            
            // Clear fallback polling
            if (fallbackPollingId) {
                clearInterval(fallbackPollingId);
                fallbackPollingId = null;
            }
            
            // Reset connection status
            connectionStatus.set({
                sseConnected: false,
                fallbackActive: false,
                lastUpdate: null,
                errorCount: 0
            });
            
            // Unsubscribe from store changes
            targetUnsubscribe();
            statusUnsubscribe();
        };
    }
};
