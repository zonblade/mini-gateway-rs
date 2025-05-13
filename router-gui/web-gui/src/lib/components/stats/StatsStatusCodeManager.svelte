<script lang="ts">
    import { currentTarget, currentStatusCode } from '$lib/stores/statisticsStore';
    import { statisticsActions } from '$lib/actions/statisticsAction';
    import StatsStatusCode from './StatsStatusCode.svelte';
    import type { StatisticsTarget } from '$lib/types/statistics';
    
    // Local state
    let selectedTarget: StatisticsTarget = 'domain';
    let selectedStatusCode: string = '200';
    
    // Common HTTP status codes for quick selection
    const commonStatusCodes = [
        { code: '200', description: 'OK' },
        { code: '201', description: 'Created' },
        { code: '204', description: 'No Content' },
        { code: '301', description: 'Moved Permanently' },
        { code: '302', description: 'Found' },
        { code: '304', description: 'Not Modified' },
        { code: '400', description: 'Bad Request' },
        { code: '401', description: 'Unauthorized' },
        { code: '403', description: 'Forbidden' },
        { code: '404', description: 'Not Found' },
        { code: '500', description: 'Internal Server Error' },
        { code: '502', description: 'Bad Gateway' },
        { code: '503', description: 'Service Unavailable' },
        { code: '504', description: 'Gateway Timeout' }
    ];
    
    // Handle target change
    function handleTargetChange(event: Event) {
        const select = event.target as HTMLSelectElement;
        selectedTarget = select.value as StatisticsTarget;
        currentTarget.set(selectedTarget);
        loadStatusCodeData();
    }
    
    // Handle status code change
    function handleStatusCodeChange(event: Event) {
        const select = event.target as HTMLSelectElement;
        selectedStatusCode = select.value;
        currentStatusCode.set(selectedStatusCode);
        loadStatusCodeData();
    }
    
    // Custom status code input
    function handleCustomStatusCode(event: Event) {
        const input = event.target as HTMLInputElement;
        const code = input.value.trim();
        
        // Validate input is a number between 100-599
        const codeNum = parseInt(code, 10);
        if (code && !isNaN(codeNum) && codeNum >= 100 && codeNum <= 599) {
            selectedStatusCode = code;
            currentStatusCode.set(selectedStatusCode);
            loadStatusCodeData();
        }
    }
    
    // Load data for the selected status code and target
    function loadStatusCodeData() {
        statisticsActions.loadStatusCodeStatistics(selectedStatusCode, selectedTarget);
    }
    
    // Initialize with current values from stores
    function initialize() {
        currentTarget.subscribe(value => {
            selectedTarget = value;
        })();
        
        currentStatusCode.subscribe(value => {
            if (value) selectedStatusCode = value;
        })();
    }
    
    // Initialize on component creation
    initialize();
    
    // Helper function to get color based on status code
    function getStatusCodeColor(code: string): string {
        const codeNum = parseInt(code, 10);
        if (codeNum >= 200 && codeNum < 300) return '#4caf50'; // Green for 2xx
        if (codeNum >= 300 && codeNum < 400) return '#ff9800'; // Orange for 3xx
        if (codeNum >= 400 && codeNum < 500) return '#f44336'; // Red for 4xx
        if (codeNum >= 500) return '#9c27b0';                  // Purple for 5xx
        return '#2196f3';                                      // Blue for others
    }
</script>

<div class="flex flex-col gap-4 h-full">
    <div class="flex justify-between items-center flex-wrap">
        <h2 class="text-xl font-semibold text-gray-900 dark:text-gray-100">HTTP Status Code Statistics</h2>
        <div class="flex items-center gap-2 flex-wrap">
            <label for="status-select" class="text-gray-700 dark:text-gray-300">Status Code:</label>
            <select 
                id="status-select" 
                value={selectedStatusCode} 
                on:change={handleStatusCodeChange}
                class="bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded px-2 py-1 text-gray-700 dark:text-gray-200"
            >
                {#each commonStatusCodes as status}
                    <option value={status.code}>{status.code} - {status.description}</option>
                {/each}
                <option value="custom">Custom...</option>
            </select>
            
            {#if selectedStatusCode === 'custom'}
                <input 
                    type="number" 
                    min="100" 
                    max="599" 
                    placeholder="Enter status code" 
                    on:change={handleCustomStatusCode}
                    class="w-24 bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded px-2 py-1 text-gray-700 dark:text-gray-200"
                />
            {/if}
            
            <label for="target-select" class="text-gray-700 dark:text-gray-300 ml-2">Data Source:</label>
            <select 
                id="target-select" 
                value={selectedTarget} 
                on:change={handleTargetChange}
                class="bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded px-2 py-1 text-gray-700 dark:text-gray-200"
            >
                <option value="domain">By Domain</option>
                <option value="proxy">By Proxy</option>
            </select>
        </div>
    </div>
    
    <div class="flex-1 min-h-[400px]">
        <StatsStatusCode statusCode={selectedStatusCode} />
    </div>
    
    <div class="flex gap-4 mt-2">
        <div class="flex items-center gap-2">
            <span class="inline-block w-4 h-4 rounded" style="background-color: {getStatusCodeColor(selectedStatusCode)};"></span>
            <span class="text-sm text-gray-700 dark:text-gray-300">Status {selectedStatusCode} count per interval (15s)</span>
        </div>
    </div>
</div>

<style>
    .stats-manager {
        display: flex;
        flex-direction: column;
        gap: 1rem;
        height: 100%;
    }
    
    .stats-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
    }
    
    .stats-header h2 {
        margin: 0;
        font-size: 1.5rem;
    }
    
    .controls {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        flex-wrap: wrap;
        justify-content: flex-end;
    }
    
    .custom-status-input {
        width: 80px;
    }
    
    .stats-content {
        flex: 1;
        min-height: 400px;
    }
    
    .stats-legend {
        display: flex;
        gap: 1rem;
        margin-top: 0.5rem;
    }
    
    .legend-item {
        display: flex;
        align-items: center;
        gap: 0.5rem;
    }
    
    .legend-color {
        display: inline-block;
        width: 16px;
        height: 16px;
        border-radius: 4px;
    }
</style>
