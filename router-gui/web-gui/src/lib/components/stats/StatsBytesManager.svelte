<script lang="ts">
    import { currentTarget } from '$lib/stores/statisticsStore';
    import { statisticsActions } from '$lib/actions/statisticsAction';
    import StatsBytes from './StatsBytes.svelte';
    import type { StatisticsTarget } from '$lib/types/statistics';
    
    // Local state
    let selectedTarget: StatisticsTarget = 'domain';
    
    // Handle target change
    function handleTargetChange(event: Event) {
        const select = event.target as HTMLSelectElement;
        selectedTarget = select.value as StatisticsTarget;
        currentTarget.set(selectedTarget);
        statisticsActions.loadBytesStatistics(selectedTarget);
    }
    
    // Initialize with current target from store
    function initializeTarget() {
        currentTarget.subscribe(value => {
            selectedTarget = value;
        })();
    }
    
    // Initialize on component creation
    initializeTarget();
</script>

<div class="flex flex-col gap-4 h-full">
    <div class="flex justify-between items-center">
        <h2 class="text-xl font-semibold text-gray-900 dark:text-gray-100">Bytes Transferred</h2>
        <div class="flex items-center gap-2">
            <label for="target-select" class="text-gray-700 dark:text-gray-300">Data Source:</label>
            <select 
                id="target-select" 
                value={selectedTarget} 
                on:change={handleTargetChange}
                class="bg-white dark:bg-gray-700 border border-gray-300 dark:border-gray-600 rounded px-2 py-1 text-gray-700 dark:text-gray-200"
            >
                <option value="domain">By Gateway</option>
                <option value="proxy">By Proxy</option>
            </select>
        </div>
    </div>
    
    <div class="flex-1 min-h-[400px]">
        <StatsBytes />
    </div>
    
    <div class="flex gap-4 mt-2">
        <div class="flex items-center gap-2">
            <span class="inline-block w-4 h-4 bg-[#4caf50] rounded"></span>
            <span class="text-sm text-gray-700 dark:text-gray-300">High: Maximum bytes/s</span>
        </div>
        <div class="flex items-center gap-2">
            <span class="inline-block w-4 h-4 bg-[#2196f3] rounded"></span>
            <span class="text-sm text-gray-700 dark:text-gray-300">Average: Average bytes/s</span>
        </div>
        <div class="flex items-center gap-2">
            <span class="inline-block w-4 h-4 bg-[#888] rounded"></span>
            <span class="text-sm text-gray-700 dark:text-gray-300">Low: Minimum bytes/s</span>
        </div>
    </div>
</div>

