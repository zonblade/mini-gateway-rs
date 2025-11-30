<script lang="ts">
    import { selectedStatusCode, rawBuffer, getStatusSparklineData } from '$lib/stores/unifiedStatsStore';

    // Common status codes for dropdown
    const commonStatusCodes = ['200', '201', '204', '301', '302', '304', '400', '401', '403', '404', '500', '502', '503'];

    let customCode: string = '';
    let showCustomInput: boolean = false;

    $: statusData = getStatusSparklineData($rawBuffer, 'gateway', $selectedStatusCode);
    $: proxyStatusData = getStatusSparklineData($rawBuffer, 'proxy', $selectedStatusCode);
    $: totalCount = statusData.reduce((a, b) => a + b, 0) + proxyStatusData.reduce((a, b) => a + b, 0);

    // Generate SVG path for the graph
    function generatePath(values: number[], height: number, width: number): string {
        if (values.length === 0) return '';

        const max = Math.max(...values, 1);
        const padding = 20;
        const effectiveHeight = height - padding * 2;
        const effectiveWidth = width - padding * 2;

        const stepX = values.length > 1 ? effectiveWidth / (values.length - 1) : 0;

        return values
            .map((v, i) => {
                const x = padding + i * stepX;
                const y = padding + effectiveHeight - (v / max) * effectiveHeight;
                return `${i === 0 ? 'M' : 'L'} ${x.toFixed(1)} ${y.toFixed(1)}`;
            })
            .join(' ');
    }

    function handleCustomCode(): void {
        const code = parseInt(customCode);
        if (code >= 100 && code <= 599) {
            selectedStatusCode.set(customCode);
            showCustomInput = false;
            customCode = '';
        }
    }

    function getStatusColor(code: string): string {
        const num = parseInt(code);
        if (num >= 200 && num < 300) return '#22c55e'; // green
        if (num >= 300 && num < 400) return '#3b82f6'; // blue
        if (num >= 400 && num < 500) return '#f59e0b'; // amber
        if (num >= 500) return '#ef4444'; // red
        return '#6b7280'; // gray
    }

    $: lineColor = getStatusColor($selectedStatusCode);
</script>

<div class="bg-white dark:bg-gray-800 rounded-lg shadow p-4">
    <!-- Header with controls -->
    <div class="flex justify-between items-center mb-4">
        <div class="text-sm font-medium text-gray-700 dark:text-gray-300">
            Status Code
        </div>
        <div class="flex items-center gap-2">
            <!-- Dropdown -->
            <select
                bind:value={$selectedStatusCode}
                class="text-sm border border-gray-300 dark:border-gray-600 rounded px-2 py-1 bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
            >
                {#each commonStatusCodes as code}
                    <option value={code}>{code}</option>
                {/each}
            </select>

            <!-- Custom input toggle -->
            {#if showCustomInput}
                <div class="flex items-center gap-1">
                    <input
                        type="number"
                        bind:value={customCode}
                        placeholder="100-599"
                        min="100"
                        max="599"
                        class="w-20 text-sm border border-gray-300 dark:border-gray-600 rounded px-2 py-1 bg-white dark:bg-gray-700 text-gray-900 dark:text-white"
                        on:keydown={(e) => e.key === 'Enter' && handleCustomCode()}
                    />
                    <button
                        on:click={handleCustomCode}
                        class="text-sm px-2 py-1 bg-blue-500 text-white rounded hover:bg-blue-600"
                    >
                        Go
                    </button>
                    <button
                        on:click={() => { showCustomInput = false; customCode = ''; }}
                        class="text-sm px-2 py-1 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
                    >
                        X
                    </button>
                </div>
            {:else}
                <button
                    on:click={() => showCustomInput = true}
                    class="text-sm text-blue-500 hover:text-blue-700 dark:text-blue-400 dark:hover:text-blue-300"
                >
                    Custom
                </button>
            {/if}
        </div>
    </div>

    <!-- Count display -->
    <div class="text-lg font-bold text-gray-900 dark:text-white mb-2">
        Count: {totalCount.toLocaleString()}
    </div>

    <!-- Graph -->
    <div class="h-32 w-full">
        <svg viewBox="0 0 400 120" class="w-full h-full" preserveAspectRatio="none">
            <!-- Grid lines -->
            <line x1="20" y1="20" x2="20" y2="100" stroke="#e5e7eb" stroke-width="1" />
            <line x1="20" y1="100" x2="380" y2="100" stroke="#e5e7eb" stroke-width="1" />

            <!-- Gateway line -->
            {#if statusData.length > 0}
                <path
                    d={generatePath(statusData, 120, 400)}
                    fill="none"
                    stroke={lineColor}
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                />
            {/if}

            <!-- Proxy line (dashed) -->
            {#if proxyStatusData.length > 0}
                <path
                    d={generatePath(proxyStatusData, 120, 400)}
                    fill="none"
                    stroke={lineColor}
                    stroke-width="2"
                    stroke-dasharray="4 2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    opacity="0.6"
                />
            {/if}

            <!-- No data placeholder -->
            {#if statusData.length === 0 && proxyStatusData.length === 0}
                <text x="200" y="60" text-anchor="middle" fill="#9ca3af" font-size="12">
                    No data
                </text>
            {/if}
        </svg>
    </div>

    <!-- Legend -->
    <div class="flex justify-center gap-4 mt-2 text-xs text-gray-500 dark:text-gray-400">
        <div class="flex items-center gap-1">
            <div class="w-4 h-0.5" style="background-color: {lineColor}"></div>
            <span>Gateway</span>
        </div>
        <div class="flex items-center gap-1">
            <div class="w-4 h-0.5 border-t border-dashed" style="border-color: {lineColor}"></div>
            <span>Proxy</span>
        </div>
    </div>
</div>