<script lang="ts">
    import Sparkline from './Sparkline.svelte';
    import { formatBytes, formatNumber } from '$lib/stores/unifiedStatsStore';

    export let label: string;
    export let value: number;
    export let sparklineData: number[] = [];
    export let format: 'number' | 'bytes' = 'number';
    export let color: string = '#3b82f6'; // blue-500

    $: displayValue = format === 'bytes' ? formatBytes(value) : formatNumber(value);
</script>

<div class="bg-white dark:bg-gray-800 rounded-lg shadow p-4 flex flex-col">
    <div class="text-sm text-gray-500 dark:text-gray-400 mb-1">{label}</div>
    <div class="text-2xl font-bold text-gray-900 dark:text-white mb-2">{displayValue}</div>
    <div class="flex-1 min-h-[40px]">
        <Sparkline data={sparklineData} {color} />
    </div>
</div>
