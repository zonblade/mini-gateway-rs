<script lang="ts">
    export let totalModels: number = 0;
    export let enabledModels: number = 0;
    export let totalInferences: number = 0;

    function formatNumber(num: number): string {
        return num.toLocaleString();
    }
</script>

<div class="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
    <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-4">
        <div class="text-sm text-gray-500 dark:text-gray-400">Total Models</div>
        <div class="text-2xl font-bold text-gray-900 dark:text-white">{formatNumber(totalModels)}</div>
    </div>
    <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-4">
        <div class="text-sm text-gray-500 dark:text-gray-400">Enabled Models</div>
        <div class="text-2xl font-bold text-green-600 dark:text-green-400">{formatNumber(enabledModels)}</div>
    </div>
    <div class="bg-white dark:bg-gray-800 rounded-lg shadow p-4">
        <div class="text-sm text-gray-500 dark:text-gray-400">Total Inferences</div>
        <div class="text-2xl font-bold text-blue-600 dark:text-blue-400">{formatNumber(totalInferences)}</div>
    </div>
</div>
