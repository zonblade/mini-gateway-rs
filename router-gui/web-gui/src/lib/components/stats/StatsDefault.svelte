<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import LineChart from '$lib/components/LineChart.svelte';
    import { defaultStatistics, currentTarget } from '$lib/stores/statisticsStore';
    import { statisticsActions } from '$lib/actions/statisticsAction';
    import type { StatisticsDataPoint } from '$lib/types/statistics';

    // Define chart data type to match LineChart component
    interface DataSeries {
        values: number[];
        name?: string;
        color?: string;
    }

    // Chart data
    let chartData: DataSeries[] = [];
    let labels: string[] = [];
    let visiblePoints = 24; // Show 24 points by default (6 minutes at 15s intervals)
    
    // Store subscriptions
    let unsubscribeDefaultStats: () => void;
    let unsubscribeTarget: () => void;
    
    // Polling interval handler
    let stopPolling: () => void;
    
    // Process data for the chart
    function processChartData(data: StatisticsDataPoint[]) {
        if (!data || data.length === 0) return;
        
        // Extract data for chart - split into two lines
        const responseValues = data.map(point => point.high); // Response count
        const requestValues = data.map(point => point.low);   // Request count
        
        // Extract time labels
        labels = data.map(point => {
            const date = new Date(point.date_time);
            return date.toLocaleTimeString();
        });
        
        // Format for LineChart - two separate lines
        chartData = [
            {
                values: responseValues,
                name: 'Responses',
                color: '#4caf50' // Green
            },
            {
                values: requestValues,
                name: 'Requests',
                color: '#2196f3' // Blue
            }
        ];
    }
    
    onMount(() => {
        // Subscribe to default statistics store
        unsubscribeDefaultStats = defaultStatistics.subscribe(data => {
            processChartData(data);
        });
        
        // Subscribe to target changes
        unsubscribeTarget = currentTarget.subscribe(() => {
            // Reload data when target changes
            statisticsActions.loadDefaultStatistics();
        });
        
        // Initial data load
        statisticsActions.loadDefaultStatistics();
        
        // Setup polling
        stopPolling = statisticsActions.setupPolling(15000); // Poll every 15 seconds
    });
    
    onDestroy(() => {
        // Clean up subscriptions
        if (unsubscribeDefaultStats) unsubscribeDefaultStats();
        if (unsubscribeTarget) unsubscribeTarget();
        if (stopPolling) stopPolling();
    });
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