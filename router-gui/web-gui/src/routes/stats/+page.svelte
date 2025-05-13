<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import StatsDefaultManager from "$lib/components/stats/StatsDefaultManager.svelte";
    import StatsBytesManager from "$lib/components/stats/StatsBytesManager.svelte";
    import StatsStatusCodeManager from "$lib/components/stats/StatsStatusCodeManager.svelte";
    import { statisticsActions } from "$lib/actions/statisticsAction";

    // Polling interval handler
    let stopPolling: () => void;

    // Tab management
    let activeTab = "default";

    function setActiveTab(tab: string) {
        activeTab = tab;
    }

    onMount(() => {
        // Setup polling for automatic data refresh
        stopPolling = statisticsActions.setupPolling(15000); // Poll every 15 seconds
    });

    onDestroy(() => {
        // Clean up polling
        if (stopPolling) stopPolling();
    });
</script>

<div class="p-4 flex justify-center">
    <div class="flex flex-col gap-4 w-full max-w-[900px]">
        <div class="mb-4">
            <h1
                class="text-2xl font-bold text-gray-900 dark:text-gray-100 mb-2"
            >
                Router Statistics
            </h1>
            <p class="text-gray-600 dark:text-gray-400">
                Real-time monitoring of router performance and traffic
            </p>
        </div>

        <div
            class="flex gap-2 border-b border-gray-200 dark:border-gray-700 pb-2"
        >
            <button
                class="px-4 py-2 border border-gray-200 dark:border-gray-700 rounded-t-lg cursor-pointer transition-all
                      {activeTab === 'default'
                    ? 'bg-white dark:bg-gray-800 border-b-white dark:border-b-gray-800 font-medium -mb-px'
                    : 'bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600'}"
                on:click={() => setActiveTab("default")}
            >
                Request & Response
            </button>
            <button
                class="px-4 py-2 border border-gray-200 dark:border-gray-700 rounded-t-lg cursor-pointer transition-all
                      {activeTab === 'bytes'
                    ? 'bg-white dark:bg-gray-800 border-b-white dark:border-b-gray-800 font-medium -mb-px'
                    : 'bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600'}"
                on:click={() => setActiveTab("bytes")}
            >
                Bytes Transferred
            </button>
            <button
                class="px-4 py-2 border border-gray-200 dark:border-gray-700 rounded-t-lg cursor-pointer transition-all
                      {activeTab === 'status'
                    ? 'bg-white dark:bg-gray-800 border-b-white dark:border-b-gray-800 font-medium -mb-px'
                    : 'bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600'}"
                on:click={() => setActiveTab("status")}
            >
                Status Codes
            </button>
        </div>

        <div
            class="flex-1 min-h-[500px] p-4 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg"
        >
            {#if activeTab === "default"}
                <StatsDefaultManager />
            {:else if activeTab === "bytes"}
                <StatsBytesManager />
            {:else if activeTab === "status"}
                <StatsStatusCodeManager />
            {/if}
        </div>

        <div class="text-center text-sm text-gray-500 dark:text-gray-400 mt-2">
            <p>
                Data refreshes automatically every 15 seconds. Last update: {new Date().toLocaleTimeString()}
            </p>
        </div>
    </div>
</div>
