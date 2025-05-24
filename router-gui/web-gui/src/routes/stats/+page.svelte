<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import { goto } from "$app/navigation";
    import { user } from "$lib/stores/userStore";
    import StatsDefaultManager from "$lib/components/stats/StatsDefaultManager.svelte";
    import StatsBytesManager from "$lib/components/stats/StatsBytesManager.svelte";
    import StatsStatusCodeManager from "$lib/components/stats/StatsStatusCodeManager.svelte";
    import { statisticsActions } from "$lib/actions/statisticsAction";
    import LoadingSpinner from "$lib/components/common/LoadingSpinner.svelte";

    // Authentication state
    let isLoggedIn = false;
    let isLoading = true;

    // Subscribe to auth store
    const unsubAuthCheck = user.subscribe(value => {
        isLoggedIn = !!value;
        isLoading = false;
    });

    // Polling interval handler
    let stopPolling: () => void;

    // Tab management
    let activeTab = "default";

    function setActiveTab(tab: string) {
        activeTab = tab;
    }

    onMount(() => {
        // Setup polling for automatic data refresh only if logged in
        if (isLoggedIn) {
            stopPolling = statisticsActions.setupPolling(15000); // Poll every 15 seconds
        }
    });

    onDestroy(() => {
        // Clean up polling and auth subscription
        if (stopPolling) stopPolling();
        unsubAuthCheck();
    });

    // Handle authentication effect
    $: if (!isLoading && !isLoggedIn) {
        goto('/');
    }
</script>

{#if isLoading}
    <LoadingSpinner />
{:else if isLoggedIn}
    <div class="p-4 flex justify-center">
        <div class="flex flex-col w-full max-w-[900px]">
            <div class="mb-4">
                <h1
                    class="text-2xl font-normal text-gray-900 dark:text-gray-100 mb-2"
                >
                    Mini Gateway Statistics
                </h1>
                <p class="text-gray-600 dark:text-gray-400">
                    Real-time monitoring of router performance and traffic
                </p>
            </div>

            <div class="flex gap-2 border-b border-gray-200 dark:border-gray-700">
                <button
                    class="px-4 py-2 cursor-pointer transition-colors
                          {activeTab === 'default'
                        ? 'border-l-6 border-indigo-500 bg-gray-50 dark:bg-gray-900 font-normal'
                        : 'border-l-2 border-transparent hover:bg-gray-50 dark:hover:bg-gray-900'}"
                    on:click={() => setActiveTab('default')}
                >
                    Request & Response
                </button>
                <button
                    class="px-4 py-2 cursor-pointer transition-colors
                          {activeTab === 'bytes'
                        ? 'border-l-6 border-indigo-500 bg-gray-50 dark:bg-gray-900 font-normal'
                        : 'border-l-2 border-transparent hover:bg-gray-50 dark:hover:bg-gray-900'}"
                    on:click={() => setActiveTab('bytes')}
                >
                    Bytes Transferred
                </button>
                <button
                    class="px-4 py-2 cursor-pointer transition-colors
                          {activeTab === 'status'
                        ? 'border-l-6 border-indigo-500 bg-gray-50 dark:bg-gray-900 font-normal'
                        : 'border-l-2 border-transparent hover:bg-gray-50 dark:hover:bg-gray-900'}"
                    on:click={() => setActiveTab('status')}
                >
                    Status Codes
                </button>
            </div>

            <div
                class="flex-1 min-h-[500px] p-4 bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700"
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
{/if}
