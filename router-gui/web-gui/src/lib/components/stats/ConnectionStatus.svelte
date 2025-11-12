<script lang="ts">
    import { connectionStatus } from '$lib/stores/statisticsStore';
    
    let showDetails = false;
    
    function toggleDetails() {
        showDetails = !showDetails;
    }
</script>

<!-- Small, unobtrusive connection status indicator -->
<div class="fixed top-4 right-4 z-50">
    <button 
        on:click={toggleDetails}
        class="flex items-center gap-2 px-3 py-1 rounded-full text-xs
               {$connectionStatus.sseConnected 
                 ? 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200' 
                 : $connectionStatus.fallbackActive 
                   ? 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200'
                   : 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200'
               } 
               hover:opacity-80 transition-opacity"
        title="Click for connection details"
    >
        <div class="w-2 h-2 rounded-full 
                   {$connectionStatus.sseConnected 
                     ? 'bg-green-500 animate-pulse' 
                     : $connectionStatus.fallbackActive 
                       ? 'bg-yellow-500' 
                       : 'bg-red-500'
                   }">
        </div>
        <span>
            {$connectionStatus.sseConnected 
              ? 'Live' 
              : $connectionStatus.fallbackActive 
                ? 'Polling' 
                : 'Offline'
            }
        </span>
    </button>

    {#if showDetails}
        <div class="absolute top-full right-0 mt-2 p-3 bg-white dark:bg-gray-800 
                   border border-gray-200 dark:border-gray-700 rounded-lg shadow-lg 
                   text-xs min-w-48">
            <h4 class="font-medium text-gray-900 dark:text-gray-100 mb-2">Connection Status</h4>
            
            <div class="space-y-1 text-gray-600 dark:text-gray-400">
                <div class="flex justify-between">
                    <span>SSE Connected:</span>
                    <span class="{$connectionStatus.sseConnected ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'}">
                        {$connectionStatus.sseConnected ? 'Yes' : 'No'}
                    </span>
                </div>
                
                <div class="flex justify-between">
                    <span>Fallback Active:</span>
                    <span class="{$connectionStatus.fallbackActive ? 'text-yellow-600 dark:text-yellow-400' : 'text-gray-500'}">
                        {$connectionStatus.fallbackActive ? 'Yes' : 'No'}
                    </span>
                </div>
                
                <div class="flex justify-between">
                    <span>Error Count:</span>
                    <span class="{$connectionStatus.errorCount > 0 ? 'text-red-600 dark:text-red-400' : 'text-gray-500'}">
                        {$connectionStatus.errorCount}
                    </span>
                </div>
                
                {#if $connectionStatus.lastUpdate}
                    <div class="flex justify-between">
                        <span>Last Update:</span>
                        <span class="text-gray-500">
                            {$connectionStatus.lastUpdate.toLocaleTimeString()}
                        </span>
                    </div>
                {/if}
            </div>
            
            <button 
                on:click={toggleDetails}
                class="mt-2 text-xs text-indigo-600 dark:text-indigo-400 hover:underline">
                Close
            </button>
        </div>
    {/if}
</div>