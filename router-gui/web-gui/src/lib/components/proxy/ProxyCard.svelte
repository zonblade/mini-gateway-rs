<script lang="ts">
    interface Proxy {
        id: number;
        title: string;
        listen: string;
        useTls: boolean;
        autoTls: boolean;
        certPem: string;
        certKey: string;
        domain: string;
    }
    
    export let proxy: Proxy;
    export let onEdit: (proxy: Proxy) => void;
    export let onDelete: (id: number) => void;
</script>

<div class="border border-gray-200 dark:border-gray-700 rounded-lg shadow-sm hover:shadow-md transition-shadow duration-300 overflow-hidden">
    <div class="p-4 relative">
        <!-- Action buttons -->
        <div class="absolute top-2 right-2 flex space-x-2">
            <button 
                on:click={() => onEdit(proxy)}
                aria-label="Edit proxy"
                class="p-1 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
            >
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path>
                    <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path>
                </svg>
            </button>
            <button 
                on:click={() => onDelete(proxy.id)}
                aria-label="Delete proxy"
                class="p-1 text-red-500 hover:text-red-700 dark:text-red-400 dark:hover:text-red-300"
            >
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <polyline points="3 6 5 6 21 6"></polyline>
                    <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
                    <line x1="10" y1="11" x2="10" y2="17"></line>
                    <line x1="14" y1="11" x2="14" y2="17"></line>
                </svg>
            </button>
        </div>
        
        <!-- Title -->
        <h3 class="text-lg font-bold text-gray-900 dark:text-gray-100 mb-3 pr-16">
            {proxy.title}
        </h3>
        
        <!-- Listen address -->
        <div class="mb-3">
            <h4 class="text-sm font-medium text-gray-700 dark:text-gray-300">Listen address</h4>
            <div class="font-mono text-sm mt-1 bg-gray-100 dark:bg-gray-700 px-2 py-1 rounded inline-block">
                {proxy.listen}
            </div>
        </div>
        
        <!-- TLS status -->
        <div class="mb-3">
            <h4 class="text-sm font-medium text-gray-700 dark:text-gray-300">TLS</h4>
            <div class="mt-1">
                {#if proxy.useTls}
                    <span class="px-2 py-1 text-xs bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-300 rounded-full">
                        {proxy.autoTls ? 'Auto SSL' : 'Manual SSL'}
                    </span>
                {:else}
                    <span class="px-2 py-1 text-xs bg-gray-100 text-gray-800 dark:bg-gray-700 dark:text-gray-300 rounded-full">
                        Disabled
                    </span>
                {/if}
            </div>
        </div>
        
        <!-- Domain (if set) -->
        {#if proxy.domain}
            <div class="mb-3">
                <h4 class="text-sm font-medium text-gray-700 dark:text-gray-300">Domain</h4>
                <div class="font-mono text-sm mt-1 bg-gray-100 dark:bg-gray-700 px-2 py-1 rounded inline-block">
                    {proxy.domain}
                </div>
            </div>
        {/if}
    </div>
</div>