<script lang="ts">
    interface Proxy {
        id: number;
        title: string;
        listen: string;
        useTls: boolean;
        autoTls: boolean;
        certPem: string;
        certKey: string;
        domain: string; // Added SNI domain field
    }
    
    export let proxies: Proxy[] = [];
    export let onEdit: (proxy: Proxy) => void;
    export let onDelete: (id: number) => void;
</script>

<div class="overflow-x-auto">
    <table class="w-full text-left">
        <thead class="bg-gray-50 dark:bg-gray-800 text-gray-500 dark:text-gray-400 text-xs uppercase">
            <tr>
                <th class="py-3 px-4">No</th>
                <th class="py-3 px-4">Title</th>
                <th class="py-3 px-4">Listen (IP and Port)</th>
                <th class="py-3 px-4">Domain (SNI)</th>
                <th class="py-3 px-4">Actions</th>
            </tr>
        </thead>
        <tbody class="divide-y divide-gray-200 dark:divide-gray-700">
            {#if proxies.length === 0}
                <tr>
                    <td colspan="5" class="py-4 px-4 text-center text-gray-500 dark:text-gray-400">
                        No proxies found
                    </td>
                </tr>
            {/if}
            
            {#each proxies as proxy, index (proxy.id)}
                <tr class="hover:bg-gray-50 dark:hover:bg-gray-800/50">
                    <td class="py-3 px-4">{index + 1}</td>
                    <td class="py-3 px-4">{proxy.title}</td>
                    <td class="py-3 px-4">
                        <span class="px-2 py-1 text-xs font-mono bg-gray-100 dark:bg-gray-800 rounded">
                            {proxy.listen}
                        </span>
                    </td>
                    <td class="py-3 px-4">
                        {#if proxy.domain}
                            <span class="px-2 py-1 text-xs font-mono bg-gray-100 dark:bg-gray-800 rounded">
                                {proxy.domain}
                            </span>
                        {:else}
                            <span class="text-gray-400 italic">Not set</span>
                        {/if}
                    </td>
                    <td class="py-3 px-4">
                        <div class="flex space-x-2">
                            <button 
                                on:click={() => onEdit(proxy)}
                                aria-label="Edit proxy"
                                class="text-blue-600 hover:text-blue-900 dark:text-blue-400 dark:hover:text-blue-200"
                            >
                                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                    <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path>
                                    <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path>
                                </svg>
                            </button>
                            <button 
                                on:click={() => onDelete(proxy.id)}
                                aria-label="Delete proxy"
                                class="text-red-600 hover:text-red-900 dark:text-red-400 dark:hover:text-red-200"
                            >
                                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                    <polyline points="3 6 5 6 21 6"></polyline>
                                    <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
                                    <line x1="10" y1="11" x2="10" y2="17"></line>
                                    <line x1="14" y1="11" x2="14" y2="17"></line>
                                </svg>
                            </button>
                        </div>
                    </td>
                </tr>
            {/each}
        </tbody>
    </table>
</div>