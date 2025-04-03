<script lang="ts">
    import { onMount } from "svelte";
    import { page } from "$app/stores";
    import { goto } from "$app/navigation";
    import { user } from "$lib/stores/userStore";
    
    // Define interfaces
    interface GwNode {
        id: number;
        title: string;
        proxyId: number;
        proxyTitle: string;
        proxyListen: string;
        target: string;
    }
    
    // Mock gwnode data (in a real app, this would be fetched from an API)
    const mockGwNodes: GwNode[] = [
        { id: 1, title: "API Gateway", proxyId: 1, proxyTitle: "Main Proxy", proxyListen: "0.0.0.0:8080", target: "192.168.1.100:3000" },
        { id: 2, title: "Web Server", proxyId: 2, proxyTitle: "Secure API", proxyListen: "0.0.0.0:443", target: "192.168.1.101:8080" },
        { id: 3, title: "Admin Panel", proxyId: 3, proxyTitle: "Internal Service", proxyListen: "127.0.0.1:9000", target: "192.168.1.102:8080" },
        { id: 4, title: "Database Access", proxyId: 4, proxyTitle: "Legacy App", proxyListen: "192.168.1.10:8000", target: "192.168.1.103:5432" },
        { id: 5, title: "Mail Server", proxyId: 5, proxyTitle: "Custom SSL", proxyListen: "0.0.0.0:8443", target: "192.168.1.104:25" },
        { id: 6, title: "File Server", proxyId: 1, proxyTitle: "Main Proxy", proxyListen: "0.0.0.0:8080", target: "192.168.1.105:21" },
        { id: 7, title: "Monitoring", proxyId: 3, proxyTitle: "Internal Service", proxyListen: "127.0.0.1:9000", target: "192.168.1.106:9090" },
        { id: 8, title: "Authentication", proxyId: 2, proxyTitle: "Secure API", proxyListen: "0.0.0.0:443", target: "192.168.1.107:8080" },
    ];
    
    // Get the gwnode ID from the route parameter
    let gwnodeId = $page.params.id ? parseInt($page.params.id) : 0;
    
    // Find the matching gwnode
    let gwnode = mockGwNodes.find(n => n.id === gwnodeId);
    
    // Set page title
    onMount(() => {
        document.title = gwnode ? `${gwnode.title} | Gateway Node` : "Gateway Node Not Found";
    });
    
    // Return to gwnode list
    function goBack() {
        window.history.back();
    }

    // Authentication and loading states
    let isLoggedIn = false;
    let isLoading = true;

    const unsubAuthCheck = user.subscribe(value => {
        isLoggedIn = !!value;
        isLoading = false;
    });

    onMount(() => {
        // Redirect happens after auth check is complete
        if (!isLoading && !isLoggedIn) {
            goto('/');
        }

        return () => {
            unsubAuthCheck(); // Clean up subscription
        };
    });
</script>

<svelte:head>
    <title>{gwnode ? `${gwnode.title} | Gateway Node` : "Gateway Node Not Found"}</title>
</svelte:head>

{#if isLoading}
    <div class="flex items-center justify-center h-screen">
        <div class="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-blue-600"></div>
    </div>
{:else if isLoggedIn}
    <div class="container mx-auto px-4 py-8">
        <!-- Back button -->
        <button 
            on:click={goBack}
            class="mb-4 flex items-center text-blue-600 hover:text-blue-800 dark:text-blue-400 dark:hover:text-blue-200"
        >
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="mr-1">
                <path d="M19 12H5M12 19l-7-7 7-7"/>
            </svg>
            Back to Gateway Nodes
        </button>
        
        {#if gwnode}
            <!-- Gateway Node Detail Card -->
            <div class="bg-white dark:bg-gray-800 rounded-lg shadow-md overflow-hidden">
                <div class="p-6">
                    <h1 class="text-2xl font-bold text-gray-900 dark:text-white mb-4">{gwnode.title}</h1>
                    
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                        <!-- Proxy Information -->
                        <div class="space-y-3">
                            <h3 class="text-lg font-medium text-gray-700 dark:text-gray-300">Proxy Information</h3>
                            <div class="bg-gray-50 dark:bg-gray-700 p-4 rounded-md">
                                <div class="mb-1">
                                    <span class="font-medium">Name:</span> {gwnode.proxyTitle}
                                </div>
                                <div>
                                    <span class="font-medium">Listen:</span> 
                                    <code class="text-sm bg-gray-200 dark:bg-gray-600 px-1 py-0.5 rounded">{gwnode.proxyListen}</code>
                                </div>
                            </div>
                        </div>
                        
                        <!-- Target Information -->
                        <div class="space-y-3">
                            <h3 class="text-lg font-medium text-gray-700 dark:text-gray-300">Target Information</h3>
                            <div class="bg-gray-50 dark:bg-gray-700 p-4 rounded-md">
                                <div>
                                    <span class="font-medium">Address:</span> 
                                    <code class="text-sm bg-gray-200 dark:bg-gray-600 px-1 py-0.5 rounded">{gwnode.target}</code>
                                </div>
                            </div>
                        </div>
                    </div>
                    
                    <!-- Detailed Status and Stats (Mock data) -->
                    <div class="mt-8">
                        <h3 class="text-lg font-medium text-gray-700 dark:text-gray-300 mb-3">Node Status and Statistics</h3>
                        <div class="bg-gray-50 dark:bg-gray-700 p-4 rounded-md grid grid-cols-1 md:grid-cols-3 gap-4">
                            <div class="text-center p-3 bg-white dark:bg-gray-800 rounded-md shadow-sm">
                                <div class="text-sm text-gray-500 dark:text-gray-400">Status</div>
                                <div class="text-lg font-medium text-green-500">Online</div>
                            </div>
                            <div class="text-center p-3 bg-white dark:bg-gray-800 rounded-md shadow-sm">
                                <div class="text-sm text-gray-500 dark:text-gray-400">Active Connections</div>
                                <div class="text-lg font-medium">143</div>
                            </div>
                            <div class="text-center p-3 bg-white dark:bg-gray-800 rounded-md shadow-sm">
                                <div class="text-sm text-gray-500 dark:text-gray-400">Traffic (today)</div>
                                <div class="text-lg font-medium">2.4 GB</div>
                            </div>
                        </div>
                    </div>
                    
                    <!-- Actions -->
                    <div class="mt-8 flex justify-end space-x-4">
                        <button 
                            class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-md text-sm font-medium"
                        >
                            Edit
                        </button>
                        <button 
                            class="px-4 py-2 bg-red-600 hover:bg-red-700 text-white rounded-md text-sm font-medium"
                        >
                            Delete
                        </button>
                    </div>
                </div>
            </div>
        {:else}
            <!-- Not Found -->
            <div class="text-center py-12">
                <h1 class="text-2xl font-bold text-gray-900 dark:text-white mb-2">Gateway Node Not Found</h1>
                <p class="text-gray-600 dark:text-gray-400">
                    The gateway node you're looking for does not exist or may have been deleted.
                </p>
            </div>
        {/if}
    </div>
{/if}