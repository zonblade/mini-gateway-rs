<script lang="ts">
    // Domain Config interface
    interface DomainConfig {
        id: string;
        domain: string;
        useTls: boolean;
        autoTls: boolean;
        certPem: string;
        certKey: string;
    }

    export let domainConfigs: DomainConfig[] = []; // Array of domain configurations
    
    // Track expanded/collapsed state for each domain (by ID)
    let expandedDomains: Set<string> = new Set();
    
    // Toggle a domain's expanded state
    function toggleDomain(id: string) {
        if (expandedDomains.has(id)) {
            expandedDomains.delete(id);
        } else {
            expandedDomains.add(id);
        }
        expandedDomains = expandedDomains; // Trigger reactivity
    }
    
    // Expand all domains
    function expandAll() {
        expandedDomains = new Set(domainConfigs.map(config => config.id));
    }
    
    // Collapse all domains
    function collapseAll() {
        expandedDomains = new Set();
    }
    
    // Add a new domain configuration
    function addDomainConfig() {
        const newId = crypto.randomUUID();
        domainConfigs = [...domainConfigs, {
            id: newId,
            domain: "",
            useTls: false,
            autoTls: false,
            certPem: "",
            certKey: "",
        }];
        // Auto-expand newly added domain
        expandedDomains.add(newId);
        expandedDomains = expandedDomains; // Trigger reactivity
    }
    
    // Remove a domain configuration
    function removeDomainConfig(id: string) {
        domainConfigs = domainConfigs.filter(config => config.id !== id);
        if (expandedDomains.has(id)) {
            expandedDomains.delete(id);
            expandedDomains = expandedDomains; // Trigger reactivity
        }
    }
</script>

<div class="space-y-6">
    <div class="flex justify-between items-center">
        <h3 class="text-sm font-semibold text-gray-700 dark:text-gray-300">Domain Configuration</h3>
        
        {#if domainConfigs.length > 1}
            <div class="flex gap-2">
                <button 
                    type="button"
                    class="text-xs text-blue-600 dark:text-blue-400 hover:underline"
                    on:click={expandAll}
                >
                    Expand All
                </button>
                <span class="text-gray-400">|</span>
                <button 
                    type="button"
                    class="text-xs text-blue-600 dark:text-blue-400 hover:underline"
                    on:click={collapseAll}
                >
                    Collapse All
                </button>
            </div>
        {/if}
    </div>

    {#if domainConfigs.length === 0}
        <div class="text-center py-4 border rounded-md p-4 border-dashed border-gray-300 dark:border-gray-600">
            <p class="text-sm text-gray-500 dark:text-gray-400">No domains configured yet.</p>
            <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">Add domains to configure routing and TLS settings.</p>
        </div>
    {:else}
        <div class="space-y-3">
            {#each domainConfigs as config (config.id)}
                <div class="border rounded-md overflow-hidden">
                    <!-- Domain Header - Always Visible -->
                    <div class="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-700/30">
                        <div class="flex items-center flex-1 min-w-0">
                            <button
                                type="button"
                                class="flex items-center text-left focus:outline-none"
                                on:click={() => toggleDomain(config.id)}
                            >
                                <span class="w-5 h-5 mr-2 flex-shrink-0">
                                    {#if expandedDomains.has(config.id)}
                                        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor">
                                            <path fill-rule="evenodd" d="M14.77 12.79a.75.75 0 01-1.06-.02L10 8.832 6.29 12.77a.75.75 0 11-1.08-1.04l4.25-4.5a.75.75 0 011.08 0l4.25 4.5a.75.75 0 01-.02 1.06z" clip-rule="evenodd" />
                                        </svg>
                                    {:else}
                                        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor">
                                            <path fill-rule="evenodd" d="M5.23 7.21a.75.75 0 011.06.02L10 11.168l3.71-3.938a.75.75 0 111.08 1.04l-4.25 4.5a.75.75 0 01-1.08 0l-4.25-4.5a.75.75 0 01.02-1.06z" clip-rule="evenodd" />
                                        </svg>
                                    {/if}
                                </span>
                                <div class="truncate">
                                    <span class="font-medium">{config.domain || 'Unnamed Domain'}</span>
                                    <div class="text-xs text-gray-500 dark:text-gray-400 flex gap-2 mt-0.5">
                                        {#if config.useTls}
                                            <span class="bg-green-100 dark:bg-green-900/30 text-green-800 dark:text-green-400 px-1.5 py-0.5 rounded-full">
                                                TLS Enabled
                                            </span>
                                        {:else}
                                            <span class="bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-400 px-1.5 py-0.5 rounded-full">
                                                TLS Disabled
                                            </span>
                                        {/if}
                                    </div>
                                </div>
                            </button>
                        </div>
                        
                        <button 
                            type="button" 
                            class="text-red-500 hover:text-red-700 p-1" 
                            on:click={() => removeDomainConfig(config.id)}
                            aria-label="Remove domain"
                        >
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                                <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd" />
                            </svg>
                        </button>
                    </div>
                    
                    <!-- Domain Details - Collapsible -->
                    {#if expandedDomains.has(config.id)}
                        <div class="p-4 border-t border-gray-200 dark:border-gray-700">
                            <div class="mb-3">
                                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                    Domain (SNI)
                                </label>
                                <input 
                                    type="text" 
                                    bind:value={config.domain}
                                    class="w-full p-2 rounded-md border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
                                    placeholder="example.com"
                                    required
                                />
                            </div>
                            
                            <div class="flex items-center mb-3">
                                <input 
                                    type="checkbox" 
                                    id={`useTls-${config.id}`}
                                    bind:checked={config.useTls}
                                    class="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
                                />
                                <label for={`useTls-${config.id}`} class="ml-2 block text-sm text-gray-700 dark:text-gray-300">
                                    Enable TLS for this domain
                                </label>
                            </div>
                            
                            {#if config.useTls}
                                <div class="pl-4 border-l-2 border-gray-300 dark:border-gray-600 space-y-3">
                                    <div class="flex items-center mb-3">
                                        <input 
                                            type="checkbox" 
                                            id={`autoTls-${config.id}`}
                                            bind:checked={config.autoTls}
                                            class="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
                                            disabled={true}
                                        />
                                        <label for={`autoTls-${config.id}`} class="ml-2 block text-sm text-gray-700 dark:text-gray-300">
                                            Auto TLS (Let's Encrypt) - upcoming feature
                                        </label>
                                    </div>
                                    
                                    {#if !config.autoTls}
                                        <div class="space-y-3">
                                            <div>
                                                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                                    Certificate PEM
                                                </label>
                                                <textarea 
                                                    bind:value={config.certPem}
                                                    class="w-full p-2 rounded-md border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 font-mono text-sm"
                                                    placeholder="-----BEGIN CERTIFICATE-----"
                                                    rows="4"
                                                    required={config.useTls && !config.autoTls}
                                                ></textarea>
                                            </div>
                                            
                                            <div>
                                                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                                    Certificate Key
                                                </label>
                                                <textarea 
                                                    bind:value={config.certKey}
                                                    class="w-full p-2 rounded-md border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 font-mono text-sm"
                                                    placeholder="-----BEGIN PRIVATE KEY-----"
                                                    rows="4"
                                                    required={config.useTls && !config.autoTls}
                                                ></textarea>
                                            </div>
                                        </div>
                                    {/if}
                                </div>
                            {/if}
                        </div>
                    {/if}
                </div>
            {/each}
        </div>
    {/if}
    
    <div class="text-center">
        <button 
            type="button"
            class="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-blue-700 bg-blue-100 hover:bg-blue-200 dark:text-blue-400 dark:bg-blue-900/30 dark:hover:bg-blue-900/50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
            on:click={addDomainConfig}
        >
            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 mr-2" viewBox="0 0 20 20" fill="currentColor">
                <path fill-rule="evenodd" d="M10 5a1 1 0 011 1v3h3a1 1 0 110 2h-3v3a1 1 0 11-2 0v-3H6a1 1 0 110-2h3V6a1 1 0 011-1z" clip-rule="evenodd" />
            </svg>
            Add Domain
        </button>
    </div>
</div> 