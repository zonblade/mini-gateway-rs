<script lang="ts">
    import type { Proxy, TlsDomain } from "$lib/types/proxy";

    export let proxy: Proxy;
    export let domains: TlsDomain[] = [];
    export let onEdit: () => void;
    export let onDelete: () => void;
    
    // Get the main domain or the first domain in the list
    $: mainDomain = domains.length > 0 ? domains[0].sni : null;
</script>

<div
    class="rounded-lg shadow-md overflow-hidden border border-gray-200 dark:border-gray-700 hover:shadow-lg transition-shadow duration-200 min-h-[220px]"
>
    <div class="p-4">
        <div class="flex justify-between items-start">
            <h3
                class="text-lg font-medium text-gray-900 dark:text-white truncate"
            >
                {proxy.title}
            </h3>
            <div class="flex space-x-1">
                <button
                    on:click|preventDefault={onEdit}
                    class="text-blue-500 hover:text-blue-700 dark:text-blue-400 dark:hover:text-blue-300 p-1"
                    aria-label="Edit proxy"
                >
                    <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path>
                        <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path>
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
        <div class="mt-2 text-sm text-gray-600 dark:text-gray-400">
            <div class="flex items-center my-1">
                <span class="font-medium w-20">Listen:</span>
                <span class="truncate">{proxy.addr_listen}</span>
            </div>
            
            {#if domains.length > 0}
                <div class="flex items-center my-1">
                    <span class="font-medium w-20">Domain{domains.length > 1 ? 's' : ''}:</span>
                    <div class="flex flex-col">
                        {#each domains.slice(0, 2) as domain}
                            <span class="truncate">{domain.sni || 'No domain'}</span>
                        {/each}
                        {#if domains.length > 2}
                            <span class="truncate text-gray-500">+{domains.length - 2} more</span>
                        {/if}
                    </div>
                </div>
            {/if}
            
            <div class="flex items-center my-1">
                <span class="font-medium w-20">TLS:</span>
                <span>{domains.some(d => d.tls_pem || d.tls_key || d.tls_autron) ? 'Yes' : 'No'}</span>
            </div>
            
            <div class="flex items-center my-1">
                <span class="font-medium w-20">High-Speed:</span>
                <span>{proxy.high_speed ? 'Yes' : 'No'}</span>
            </div>
        </div>
    </div>
</div>
