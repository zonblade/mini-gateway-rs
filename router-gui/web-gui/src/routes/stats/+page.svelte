<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { goto } from '$app/navigation';
    import { user } from '$lib/stores/userStore';
    import StatCard from '$lib/components/stats/StatCard.svelte';
    import StatusCodeGraph from '$lib/components/stats/StatusCodeGraph.svelte';
    import {
        rawBuffer,
        displayInterval,
        connectionStatus,
        aggregatedStats,
        getSparklineData,
        type DisplayInterval
    } from '$lib/stores/unifiedStatsStore';
    import { connectSSE, disconnectSSE } from '$lib/services/unifiedStatsService';

    // Check auth
    $: if (!$user) {
        goto('/');
    }

    // Interval options
    const intervals: { value: DisplayInterval; label: string }[] = [
        { value: 'live', label: 'Live (15s)' },
        { value: '30s', label: '30s' },
        { value: '1m', label: '1m' },
        { value: '5m', label: '5m' }
    ];

    // Sparkline data (reactive)
    $: gatewayReqSparkline = getSparklineData($rawBuffer, 'gateway', 'req');
    $: gatewayResSparkline = getSparklineData($rawBuffer, 'gateway', 'res');
    $: gatewayBytesInSparkline = getSparklineData($rawBuffer, 'gateway', 'bytes_in');
    $: gatewayBytesOutSparkline = getSparklineData($rawBuffer, 'gateway', 'bytes_out');

    $: proxyReqSparkline = getSparklineData($rawBuffer, 'proxy', 'req');
    $: proxyResSparkline = getSparklineData($rawBuffer, 'proxy', 'res');
    $: proxyBytesInSparkline = getSparklineData($rawBuffer, 'proxy', 'bytes_in');
    $: proxyBytesOutSparkline = getSparklineData($rawBuffer, 'proxy', 'bytes_out');

    $: gatewayFailedSparkline = getSparklineData($rawBuffer, 'gateway', 'failed');
    $: proxyFailedSparkline = getSparklineData($rawBuffer, 'proxy', 'failed');

    onMount(() => {
        connectSSE();
    });

    onDestroy(() => {
        disconnectSSE();
    });
</script>

<svelte:head>
    <title>Stats Dashboard</title>
</svelte:head>

<div class="px-4 flex flex-col items-center">
    <div class="py-8 w-full max-w-[1200px]">
        <!-- Header -->
        <div class="flex justify-between items-center mb-6">
        <h1 class="text-2xl font-bold text-gray-900 dark:text-white">Stats Dashboard</h1>
        <div class="flex items-center gap-4">
            <!-- Interval selector -->
            <div class="flex bg-white dark:bg-gray-800 rounded-lg shadow overflow-hidden">
                {#each intervals as interval}
                    <button
                        on:click={() => displayInterval.set(interval.value)}
                        class="px-3 py-1.5 text-sm font-medium transition-colors
                            {$displayInterval === interval.value
                                ? 'bg-blue-500 text-white'
                                : 'text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700'}"
                    >
                        {interval.label}
                    </button>
                {/each}
            </div>

            <!-- Connection status -->
            <div class="flex items-center gap-2 text-sm">
                <div
                    class="w-2 h-2 rounded-full {$connectionStatus === 'connected'
                        ? 'bg-green-500'
                        : 'bg-red-500'}"
                ></div>
                <span class="text-gray-600 dark:text-gray-400">
                    {$connectionStatus === 'connected' ? 'Live' : 'Offline'}
                </span>
            </div>
        </div>
    </div>

    <!-- Gateway Section -->
    <div class="mb-6">
        <h2 class="text-lg font-semibold text-gray-800 dark:text-gray-200 mb-3">Gateway</h2>
        <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-4">
            <StatCard
                label="Requests"
                value={$aggregatedStats?.gateway.req ?? 0}
                sparklineData={gatewayReqSparkline}
                color="#3b82f6"
            />
            <StatCard
                label="Responses"
                value={$aggregatedStats?.gateway.res ?? 0}
                sparklineData={gatewayResSparkline}
                color="#22c55e"
            />
            <StatCard
                label="Bytes In"
                value={$aggregatedStats?.gateway.bytes_in ?? 0}
                sparklineData={gatewayBytesInSparkline}
                format="bytes"
                color="#8b5cf6"
            />
            <StatCard
                label="Bytes Out"
                value={$aggregatedStats?.gateway.bytes_out ?? 0}
                sparklineData={gatewayBytesOutSparkline}
                format="bytes"
                color="#f59e0b"
            />
            <StatCard
                label="Failed"
                value={$aggregatedStats?.gateway.failed ?? 0}
                sparklineData={gatewayFailedSparkline}
                color="#ef4444"
            />
            <StatCard
                label="Stalled"
                value={$aggregatedStats?.gateway.stalled_count ?? 0}
                color="#f97316"
            />
        </div>
    </div>

    <!-- Proxy Section -->
    <div class="mb-6">
        <h2 class="text-lg font-semibold text-gray-800 dark:text-gray-200 mb-3">Proxy</h2>
        <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-4">
            <StatCard
                label="Requests"
                value={$aggregatedStats?.proxy.req ?? 0}
                sparklineData={proxyReqSparkline}
                color="#3b82f6"
            />
            <StatCard
                label="Responses"
                value={$aggregatedStats?.proxy.res ?? 0}
                sparklineData={proxyResSparkline}
                color="#22c55e"
            />
            <StatCard
                label="Bytes In"
                value={$aggregatedStats?.proxy.bytes_in ?? 0}
                sparklineData={proxyBytesInSparkline}
                format="bytes"
                color="#8b5cf6"
            />
            <StatCard
                label="Bytes Out"
                value={$aggregatedStats?.proxy.bytes_out ?? 0}
                sparklineData={proxyBytesOutSparkline}
                format="bytes"
                color="#f59e0b"
            />
            <StatCard
                label="Failed"
                value={$aggregatedStats?.proxy.failed ?? 0}
                sparklineData={proxyFailedSparkline}
                color="#ef4444"
            />
            <StatCard
                label="Stalled"
                value={$aggregatedStats?.proxy.stalled_count ?? 0}
                color="#f97316"
            />
        </div>
    </div>

    <!-- Status Code Section -->
    <div>
        <h2 class="text-lg font-semibold text-gray-800 dark:text-gray-200 mb-3">Status Code</h2>
        <StatusCodeGraph />
    </div>
    </div>
</div>
