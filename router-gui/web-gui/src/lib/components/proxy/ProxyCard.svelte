<script lang="ts">
    import type { Proxy } from '$lib/types/proxy';
    
    export let proxy: Proxy;
    export let onEdit: () => void;
    export let onDelete: () => void;
</script>

<div class="rounded-lg shadow-md overflow-hidden border border-gray-200 dark:border-gray-700 hover:shadow-lg transition-shadow duration-200 min-h-[220px]">
    <div class="p-4">
        <div class="flex justify-between items-start">
            <h3 class="text-lg font-medium text-gray-900 dark:text-white truncate">{proxy.title}</h3>
            <div class="flex space-x-1">
                <button 
                    on:click|preventDefault={onEdit}
                    class="text-blue-500 hover:text-blue-700 dark:text-blue-400 dark:hover:text-blue-300 p-1"
                    aria-label="Edit proxy"
                >
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                        <path d="M13.586 3.586a2 2 0 112.828 2.828l-.793.793-2.828-2.828.793-.793zM11.379 5.793L3 14.172V17h2.828l8.38-8.379-2.83-2.828z" />
                    </svg>
                </button>
                <button 
                    on:click={onDelete}
                    class="text-red-500 hover:text-red-700 dark:text-red-400 dark:hover:text-red-300 p-1"
                    aria-label="Delete proxy"
                >
                    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <polyline points="3 6 5 6 21 6"></polyline>
                        <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
                        <line x1="10" y1="11" x2="10" y2="17"></line>
                        <line x1="14" y1="11" x2="14" y2="17"></line>
                    </svg>
                </button>
            </div>
        </div>
        
        <div class="mt-3">
            <div class="flex items-center text-sm">
                <span class="font-medium text-gray-600 dark:text-gray-300 mr-2">Listen:</span>
                <code class="text-xs bg-gray-100 dark:bg-gray-700 px-1 py-0.5 rounded font-mono">
                    {proxy.addr_listen}
                </code>
            </div>
            
            {#if proxy.tls}
                <div class="flex items-center mt-2">
                    <span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-300">
                        <svg class="h-3 w-3 mr-1" fill="currentColor" viewBox="0 0 20 20">
                            <path fill-rule="evenodd" d="M5 9V7a5 5 0 0110 0v2a2 2 0 012 2v5a2 2 0 01-2 2H5a2 2 0 01-2-2v-5a2 2 0 012-2zm8-2v2H7V7a3 3 0 016 0z" clip-rule="evenodd" />
                        </svg>
                        TLS Enabled
                    </span>
                    
                    {#if proxy.tls_autron}
                        <span class="ml-2 inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-300">
                            Auto Renew
                        </span>
                    {/if}
                </div>
            {/if}
            
            {#if proxy.sni}
                <div class="flex items-center text-sm mt-2">
                    <span class="font-medium text-gray-600 dark:text-gray-300 mr-2">Domain:</span>
                    <code class="text-xs bg-gray-100 dark:bg-gray-700 px-1 py-0.5 rounded font-mono">
                        {proxy.sni}
                    </code>
                </div>
            {/if}
            
            {#if proxy.high_speed}
                <div class="flex items-center mt-2">
                    <span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-300">
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3 mr-1" viewBox="0 0 20 20" fill="currentColor">
                            <path fill-rule="evenodd" d="M11.3 1.046A1 1 0 0112 2v5h4a1 1 0 01.82 1.573l-7 10A1 1 0 018 18v-5H4a1 1 0 01-.82-1.573l7-10a1 1 0 011.12-.38z" clip-rule="evenodd" />
                        </svg>
                        High-Speed Mode
                    </span>
                </div>
                
                {#if proxy.high_speed_addr}
                    <div class="flex items-center text-sm mt-2">
                        <span class="font-medium text-gray-600 dark:text-gray-300 mr-2">Gateway:</span>
                        <code class="text-xs bg-gray-100 dark:bg-gray-700 px-1 py-0.5 rounded font-mono">
                            {proxy.high_speed_addr}
                        </code>
                    </div>
                {/if}
            {/if}
        </div>
    </div>
</div>