<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import LineChart from '$lib/components/LineChart.svelte';
    import { statusCodeStatistics, currentStatusCode, currentTarget } from '$lib/stores/statisticsStore';
    import { statisticsActions } from '$lib/actions/statisticsAction';
    import type { StatisticsDataPoint } from '$lib/types/statistics';

    // Define chart data type to match LineChart component
    interface DataSeries {
        values: number[];
        name?: string;
        color?: string;
    }

    // Props
    export let statusCode: string = '200';

    // Chart data
    let chartData: DataSeries[] = [];
    let labels: string[] = [];
    let visiblePoints = 24; // Show 24 points by default (6 minutes at 15s intervals)
    
    // Store subscriptions
    let unsubscribeStatusCodeStats: () => void;
    let unsubscribeCurrentCode: () => void;
    let unsubscribeTarget: () => void;
    
    // Polling interval handler
    let stopPolling: () => void;
    
    // Process data for the chart
    function processChartData(data: StatisticsDataPoint[]) {
        if (!data || data.length === 0) return;
        
        // Extract data for chart - just the value field
        const values = data.map(point => point.value);
        
        // Extract time labels
        labels = data.map(point => {
            const date = new Date(point.date_time);
            return date.toLocaleTimeString();
        });
        
        // Format for LineChart - single line
        chartData = [
            {
                values: values,
                name: `Status ${statusCode}`,
                color: getColorForStatusCode(statusCode)
            }
        ];
    }
    
    // Get appropriate color based on status code
    function getColorForStatusCode(code: string): string {
        const codeNum = parseInt(code, 10);
        if (codeNum >= 200 && codeNum < 300) return '#4caf50'; // Green for 2xx
        if (codeNum >= 300 && codeNum < 400) return '#ff9800'; // Orange for 3xx
        if (codeNum >= 400 && codeNum < 500) return '#f44336'; // Red for 4xx
        if (codeNum >= 500) return '#9c27b0';                  // Purple for 5xx
        return '#2196f3';                                      // Blue for others
    }
    
    // Load data for specific status code
    function loadStatusCodeData() {
        statisticsActions.loadStatusCodeStatistics(statusCode);
    }
    
    onMount(() => {
        // Set current status code in store
        currentStatusCode.set(statusCode);
        
        // Subscribe to status code statistics store
        unsubscribeStatusCodeStats = statusCodeStatistics.subscribe(data => {
            processChartData(data);
        });
        
        // Subscribe to current status code changes
        unsubscribeCurrentCode = currentStatusCode.subscribe(code => {
            if (code !== statusCode) {
                statusCode = code;
                loadStatusCodeData();
            }
        });
        
        // Subscribe to target changes
        unsubscribeTarget = currentTarget.subscribe(() => {
            loadStatusCodeData();
        });
        
        // Initial data load
        loadStatusCodeData();
        
        // Setup polling
        stopPolling = statisticsActions.setupPolling(15000); // Poll every 15 seconds
    });
    
    onDestroy(() => {
        // Clean up subscriptions
        if (unsubscribeStatusCodeStats) unsubscribeStatusCodeStats();
        if (unsubscribeCurrentCode) unsubscribeCurrentCode();
        if (unsubscribeTarget) unsubscribeTarget();
        if (stopPolling) stopPolling();
    });
    
    // Watch for status code prop changes
    $: if (statusCode) {
        currentStatusCode.set(statusCode);
        if (typeof loadStatusCodeData === 'function') {
            loadStatusCodeData();
        }
    }
</script>

<div class="h-full w-full">
    <LineChart 
        data={chartData} 
        labels={labels}
        visiblePoints={visiblePoints}
        showXLabels={false}
        yAxisLabelColor="#9ca3af"
    />
</div>
