<script lang="ts">
    import { currentTarget } from '$lib/stores/statisticsStore';
    import { statisticsActions } from '$lib/actions/statisticsAction';
    import StatsDefault from './StatsDefault.svelte';
    import type { StatisticsTarget } from '$lib/types/statistics';
    
    // Local state
    let selectedTarget: StatisticsTarget = 'domain';
    
    // Handle target change
    function handleTargetChange(event: Event) {
        const select = event.target as HTMLSelectElement;
        selectedTarget = select.value as StatisticsTarget;
        currentTarget.set(selectedTarget);
        statisticsActions.loadDefaultStatistics(selectedTarget);
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
        <h2 class="text-xl font-normal text-gray-900 dark:text-gray-100">Request & Response Statistics</h2>
        <div class="flex items-center gap-2">
            <label for="target-select" class="text-gray-700 dark:text-gray-300">Data Source:</label>
            <select 
                id="target-select" 
                value={selectedTarget} 
                on:change={handleTargetChange}
                class="bg-transparent border-b border-gray-300 dark:border-gray-600 px-2 py-1 text-gray-700 dark:text-gray-200 focus:outline-none focus:border-indigo-500"
            >
                <option value="domain">By Gateway</option>
                <option value="proxy">By Proxy</option>
            </select>
        </div>
    </div>
    
    <div class="flex-1 min-h-[400px]">
        <StatsDefault />
    </div>
    
    <div class="flex gap-6 mt-2">
        <div class="flex items-center gap-2">
            <span class="inline-block w-4 h-2 bg-[#4caf50]"></span>
            <span class="text-sm text-gray-700 dark:text-gray-300">Responses: Successful responses</span>
        </div>
        <div class="flex items-center gap-2">
            <span class="inline-block w-4 h-2 bg-[#2196f3]"></span>
            <span class="text-sm text-gray-700 dark:text-gray-300">Requests: Total requests</span>
        </div>
        <div class="flex items-center gap-2">
            <span class="text-sm italic text-gray-500 dark:text-gray-400">Failed/Unmatched = Requests - Responses</span>
        </div>
    </div>
</div>
