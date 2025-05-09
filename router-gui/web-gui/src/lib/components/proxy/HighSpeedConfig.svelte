<script lang="ts">
    interface GwNode {
        id: string;
        proxy_id: string;
        title: string;
        alt_target: string;
    }
    
    export let highSpeed = false;
    export let highSpeedAddr = "";
    export let selectedGwNodeId = "";
    export let gwNodes: GwNode[] = [];
    export let loadingGwNodes = false;
    export let errorLoadingGwNodes: string | null = null;
    
    $: showHighSpeedWarning = highSpeed && gwNodes.length === 0 && !loadingGwNodes;

    // When selecting a gateway node, store both the ID and the address
    function selectGwNode(event: Event) {
        const selectElement = event.target as HTMLSelectElement;
        const selectedValue = selectElement.value;
        
        if (selectedValue) {
            const selectedOption = selectElement.options[selectElement.selectedIndex];
            selectedGwNodeId = selectedOption.getAttribute('data-id') || "";
            highSpeedAddr = selectedValue;
        } else {
            selectedGwNodeId = "";
            highSpeedAddr = "";
        }
    }
</script>

<div class="space-y-3 mt-4 border-t pt-4 dark:border-gray-700">
    <h3 class="text-sm font-semibold text-gray-700 dark:text-gray-300">High-Speed Mode</h3>
    
    <div class="flex items-center">
        <input 
            type="checkbox" 
            id="highSpeed" 
            bind:checked={highSpeed}
            class="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
        />
        <label for="highSpeed" class="ml-2 block text-sm text-gray-700 dark:text-gray-300">
            Enable High-Speed Mode
        </label>
    </div>
    
    {#if showHighSpeedWarning}
        <div class="pl-6 text-sm text-yellow-700 dark:text-yellow-500 bg-yellow-100 dark:bg-yellow-900/30 p-2 rounded">
            <p>No gateway nodes available for this proxy. High-speed mode will use the default target address.</p>
        </div>
    {/if}
    
    {#if highSpeed}
        <div class="pl-6">
            <label for="highSpeedAddr" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                Gateway Node <span class="text-red-500">*</span>
            </label>
            
            {#if loadingGwNodes}
                <div class="text-sm text-gray-500 dark:text-gray-400">
                    Loading gateway nodes...
                </div>
            {:else if errorLoadingGwNodes}
                <div class="text-sm text-red-600 dark:text-red-400">
                    {errorLoadingGwNodes}
                </div>
            {:else}
                <select
                    id="highSpeedAddr"
                    value={highSpeedAddr}
                    on:change={selectGwNode}
                    class="w-full p-2 rounded-md border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 {!highSpeedAddr && highSpeed ? 'border-red-500 dark:border-red-500' : ''}"
                    required={highSpeed}
                >
                    <option value="">Select a gateway node</option>
                    {#each gwNodes as node (node.id)}
                        <option value={node.alt_target} data-id={node.id}>{node.title} ({node.alt_target})</option>
                    {/each}
                </select>
                
                <p class="mt-1 text-xs {highSpeed && !highSpeedAddr ? 'text-red-600 dark:text-red-500 font-medium' : 'text-gray-500 dark:text-gray-400'}">
                    {#if gwNodes.length === 0}
                        {highSpeed ? "No gateway nodes available. Create gateway nodes for this proxy to use them in high-speed mode." : "No gateway nodes available."}
                    {:else if highSpeed && !highSpeedAddr}
                        Please select a gateway node to use high-speed mode.
                    {:else}
                        {gwNodes.length} gateway node(s) available for high-speed mode.
                    {/if}
                </p>
            {/if}
        </div>
    {/if}
    
    {#if highSpeed && !highSpeedAddr && gwNodes.length > 0}
        <div class="text-sm text-red-600 dark:text-red-400 bg-red-100 dark:bg-red-900/20 p-2 rounded my-2">
            Please select a gateway node for high-speed mode or disable high-speed mode.
        </div>
    {/if}
</div> 