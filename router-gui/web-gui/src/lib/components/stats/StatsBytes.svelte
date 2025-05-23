<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import HighLowChart from '$lib/components/HighLowChart.svelte';
    import { bytesStatistics, currentTarget } from '$lib/stores/statisticsStore';
    import { statisticsActions } from '$lib/actions/statisticsAction';
    import type { StatisticsDataPoint } from '$lib/types/statistics';

    // Define chart data type to match HighLowChart component
    interface ChartDataSeries {
        highs: number[];
        lows: number[];
        avgs: number[];
        color?: string;
        avgColor?: string;
        name?: string;
    }

    // Chart data
    let chartData: ChartDataSeries[] = [];
    let labels: string[] = [];
    let visiblePoints = 24; // Show 24 points by default (6 minutes at 15s intervals)
    
    // Store subscriptions
    let unsubscribeBytesStats: () => void;
    let unsubscribeTarget: () => void;
    
    // Polling interval handler
    let stopPolling: () => void;
    
    // Process data for the chart
    function processChartData(data: StatisticsDataPoint[]) {
        if (!data || data.length === 0) return;
        
        // Extract data for chart
        const highs = data.map(point => point.high);
        const lows = data.map(point => point.low);
        const avgs = data.map(point => point.value);
        
        // Extract time labels
        labels = data.map(point => {
            const date = new Date(point.date_time);
            return date.toLocaleTimeString();
        });
        
        // Format for HighLowChart
        chartData = [{
            highs: highs,
            lows: lows,
            avgs: avgs,
            color: '#4caf50', // Green
            avgColor: '#2196f3' // Blue
        }];
    }
    
    onMount(() => {
        // Subscribe to bytes statistics store
        unsubscribeBytesStats = bytesStatistics.subscribe(data => {
            processChartData(data);
        });
        
        // Subscribe to target changes
        unsubscribeTarget = currentTarget.subscribe(() => {
            // Reload data when target changes
            statisticsActions.loadBytesStatistics();
        });
        
        // Initial data load
        statisticsActions.loadBytesStatistics();
        
        // Setup polling
        stopPolling = statisticsActions.setupPolling(15000); // Poll every 15 seconds
    });
    
    onDestroy(() => {
        // Clean up subscriptions
        if (unsubscribeBytesStats) unsubscribeBytesStats();
        if (unsubscribeTarget) unsubscribeTarget();
        if (stopPolling) stopPolling();
    });
</script>

<div class="h-full w-full">
    <HighLowChart 
        data={chartData} 
        labels={labels}
        visiblePoints={visiblePoints}
        showXLabels={false}
        yAxisLabelColor="#9ca3af" 
    />
</div>
