<script lang="ts">
    import { fade } from "svelte/transition";
    import { gwnodeActions } from "$lib/actions/gwnodeActions";
    
    // Define Proxy interface to match the expected input from ProxyManager
    interface Proxy {
        id: string; // Changed to string
        title: string;
        listen: string;
        useTls: boolean;
        autoTls: boolean;
        certPem: string;
        certKey: string;
        domain: string;
        target?: string; // Keep this property but we'll hide it from UI
        highSpeed: boolean;
        highSpeedAddr: string;
    }
    
    // Define GwNode interface
    interface GwNode {
        id: string;
        proxy_id: string;
        title: string;
        alt_target: string;
    }
    
    export let showModal: boolean = false;
    export let isEditMode: boolean = false;
    export let proxy: Proxy = { 
        id: '', // Changed default ID to empty string
        title: "", 
        listen: "", 
        useTls: false, 
        autoTls: false, 
        certPem: "", 
        certKey: "",
        domain: "",
        target: "",
        highSpeed: false,
        highSpeedAddr: ""
    };
    export let onSave: () => void;

// Function to validate form before submitting
function handleSubmit(): void {
    // Validate that if high-speed mode is enabled, a gateway node must be selected
    if (proxy.highSpeed && !proxy.highSpeedAddr) {
        // The form won't submit due to the disabled button and validation message
        return;
    }
    
    // Call the provided onSave function from parent component
    onSave();
}
    export let onClose: () => void;
    
    let gwNodes: GwNode[] = [];
    let loadingGwNodes: boolean = false;
    let errorLoadingGwNodes: string | null = null;
    let showHighSpeedWarning: boolean = false;

    // Function to load gateway nodes based on mode (edit/add)
    async function loadGwNodes() {
        // Only run if the modal is actually visible
        if (!showModal) return;

        loadingGwNodes = true;
        errorLoadingGwNodes = null; // Reset error on new load attempt
        console.log(`Loading GW nodes. isEditMode: ${isEditMode}, proxy.id: ${proxy.id}`);
        try {
            // Check if this is an edit mode with a valid ID (not empty string)
            if (isEditMode && proxy.id && proxy.id !== '') { 
                // For existing proxies, get available nodes (unbound + assigned to this proxy)
                // proxy.id is already a string, no need for .toString()
                gwNodes = await gwnodeActions.getAvailableGwNodesForProxy(proxy.id);
                console.log(`Loaded ${gwNodes.length} gateway nodes for existing proxy ID ${proxy.id}:`, gwNodes);
            } else {
                // For new proxies, get unbound nodes
                gwNodes = await gwnodeActions.getAvailableGwNodesForProxy();
                console.log(`Loaded ${gwNodes.length} unbound gateway nodes for new proxy:`, gwNodes);
            }
        } catch (error) {
            console.error("Failed to load gateway nodes:", error);
            errorLoadingGwNodes = "Failed to load gateway nodes";
            gwNodes = []; // Clear nodes on error
        } finally {
            loadingGwNodes = false;
            // Update warning state after loading finishes
            showHighSpeedWarning = proxy.highSpeed && gwNodes.length === 0 && !loadingGwNodes;
        }
    }

    // Reactive statement to reload nodes when the modal is opened
    // This automatically runs when showModal changes to true
    $: if (showModal) {
        console.log("Modal opened, triggering GW node load.");
        loadGwNodes(); 
    } else {
        // Optional: Reset state when modal closes if needed
        // gwNodes = []; 
        // errorLoadingGwNodes = null;
        console.log("Modal closed.");
    }
    
    // Handle ESC key to close modal
    function handleKeydown(event: KeyboardEvent) {
        if (event.key === 'Escape') {
            onClose();
        }
    }
    
    // Keep events from propagating outside the modal
    function handleModalKeyDown(event: KeyboardEvent) {
        event.stopPropagation();
    }
    
    // Logic for TLS options
    $: if (!proxy.useTls) {
        proxy.autoTls = false;
    }
    
    // Logic for high-speed mode
    $: if (!proxy.highSpeed) {
        proxy.highSpeedAddr = "";
        showHighSpeedWarning = false;
    } else {
        // Re-evaluate warning when highSpeed turns on, or when nodes finish loading
        showHighSpeedWarning = gwNodes.length === 0 && !loadingGwNodes;
    }
    
    // Enable/disable cert fields logic (Ensure this is still present)
    $: certFieldsDisabled = !proxy.useTls || proxy.autoTls;
</script>

{#if showModal}
    <div 
        class="fixed inset-0 bg-black/30 backdrop-blur-md bg-opacity-50 flex items-center justify-center z-50" 
        transition:fade={{ duration: 200 }}
        on:keydown={handleKeydown}
        role="presentation"
    >
        <div 
            class="bg-white dark:bg-[#161b22] shadow-xl max-w-md w-full mx-4 max-h-screen overflow-y-auto"
            on:click|stopPropagation
            on:keydown={handleModalKeyDown}
            role="dialog"
            aria-labelledby="modal-title"
            aria-modal="true"
            tabindex="-1"
        >
            <div class="p-6">
                <div class="flex justify-between items-center mb-4">
                    <h2 id="modal-title" class="text-xl font-bold">{isEditMode ? 'Edit Proxy' : 'Add Proxy'}</h2>
                    <button 
                        on:click={onClose}
                        aria-label="Close"
                        class="text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                            <line x1="18" y1="6" x2="6" y2="18"></line>
                            <line x1="6" y1="6" x2="18" y2="18"></line>
                        </svg>
                    </button>
                </div>
                
                <form on:submit|preventDefault={handleSubmit} class="space-y-4">
                    <div>
                        <label for="title" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                            Title
                        </label>
                        <input 
                            type="text" 
                            id="title" 
                            bind:value={proxy.title}
                            class="w-full p-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
                            required
                        />
                    </div>
                    
                    <div>
                        <label for="listen" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                            Listen (IP and Port)
                        </label>
                        <input 
                            type="text" 
                            id="listen" 
                            bind:value={proxy.listen}
                            class="w-full p-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
                            required
                            placeholder="Example: 0.0.0.0:8080"
                        />
                    </div>
                    
                    <div>
                        <label for="domain" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                            Domain (SNI)
                        </label>
                        <input 
                            type="text" 
                            id="domain" 
                            bind:value={proxy.domain}
                            class="w-full p-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
                            placeholder="Example: example.com"
                        />
                        <p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
                            Server Name Indication for TLS connections
                        </p>
                    </div>
                    
                    <!-- Target field removed from UI -->
                    
                    <div class="flex items-center">
                        <input 
                            type="checkbox" 
                            id="useTls" 
                            bind:checked={proxy.useTls}
                            class="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
                        />
                        <label for="useTls" class="ml-2 block text-sm text-gray-700 dark:text-gray-300">
                            Use TLS
                        </label>
                    </div>
                    
                    {#if proxy.useTls}
                        <div class="pl-6">
                            <div class="flex items-center">
                                <input 
                                    type="checkbox" 
                                    id="autoTls" 
                                    bind:checked={proxy.autoTls}
                                    class="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
                                    disabled={true}
                                />
                                <label for="autoTls" class="ml-2 block text-sm text-gray-700 dark:text-gray-300">
                                    Auto TLS (Let's Encrypt) - upcoming feature
                                </label>
                            </div>
                        </div>
                    {/if}
                    
                    {#if proxy.useTls && !proxy.autoTls}
                        <div class="pl-6 space-y-4">
                            <div>
                                <label for="certPem" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                    Certificate PEM
                                </label>
                                <textarea 
                                    id="certPem" 
                                    bind:value={proxy.certPem}
                                    class="w-full p-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 font-mono text-sm"
                                    placeholder="this is cert"
                                    rows="4"
                                    disabled={certFieldsDisabled}
                                ></textarea>
                            </div>
                            
                            <div>
                                <label for="certKey" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                    Certificate Key
                                </label>
                                <textarea 
                                    id="certKey" 
                                    bind:value={proxy.certKey}
                                    class="w-full p-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 font-mono text-sm"
                                    placeholder="this is cert"
                                    rows="4"
                                    disabled={certFieldsDisabled}
                                ></textarea>
                            </div>
                        </div>
                    {/if}
                    
                    <!-- High-Speed Mode Section -->
                    <div class="space-y-3 mt-4 border-t pt-4 dark:border-gray-700">
                        <h3 class="text-sm font-semibold text-gray-700 dark:text-gray-300">High-Speed Mode</h3>
                        
                        <div class="flex items-center">
                            <input 
                                type="checkbox" 
                                id="highSpeed" 
                                bind:checked={proxy.highSpeed}
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
                        
                        {#if proxy.highSpeed}
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
                                        bind:value={proxy.highSpeedAddr}
                                        class="w-full p-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 {!proxy.highSpeedAddr && 'border-red-500 dark:border-red-500'}"
                                        required={proxy.highSpeed}
                                    >
                                        <option value="">Select a gateway node</option>
                                        {#each gwNodes as node (node.id)}
                                            <option value={node.alt_target}>{node.title} ({node.alt_target})</option>
                                        {/each}
                                    </select>
                                    
                                    <p class="mt-1 text-xs {proxy.highSpeed && !proxy.highSpeedAddr ? 'text-red-600 dark:text-red-500 font-medium' : 'text-gray-500 dark:text-gray-400'}">
                                        {#if gwNodes.length === 0}
                                            {proxy.highSpeed ? "No gateway nodes available. Create gateway nodes for this proxy to use them in high-speed mode." : "No gateway nodes available."}
                                        {:else if proxy.highSpeed && !proxy.highSpeedAddr}
                                            Please select a gateway node to use high-speed mode.
                                        {:else}
                                            {gwNodes.length} gateway node(s) available for high-speed mode.
                                        {/if}
                                    </p>
                                {/if}
                                
                                <!-- Debug information - Always visible during development -->
                                {#if import.meta.env.DEV}
                                    <div class="mt-4 p-2 border border-gray-300 dark:border-gray-600 bg-gray-100 dark:bg-gray-800">
                                        <h4 class="text-xs font-bold text-gray-500 dark:text-gray-400 mb-1">Debug Info:</h4>
                                        <pre class="text-xs text-gray-600 dark:text-gray-300 overflow-auto max-h-32">
Loading GW Nodes: {loadingGwNodes}
Error Loading GW Nodes: {errorLoadingGwNodes || 'None'}
gwNodes length: {gwNodes.length}
gwNodes: {JSON.stringify(gwNodes, null, 2)}
isEditMode: {isEditMode}
Current Proxy ID: {proxy.id}
highSpeedAddr: {proxy.highSpeedAddr}
showHighSpeedWarning: {showHighSpeedWarning}
                                        </pre>
                                    </div>
                                {/if}
                            </div>
                        {/if}
                    </div>
                    
                    {#if proxy.highSpeed && !proxy.highSpeedAddr && gwNodes.length > 0}
                        <div class="text-sm text-red-600 dark:text-red-400 bg-red-100 dark:bg-red-900/20 p-2 my-2">
                            Please select a gateway node for high-speed mode or disable high-speed mode.
                        </div>
                    {/if}
                    
                    <div class="flex justify-end space-x-2 pt-4">
                        <button 
                            type="button"
                            on:click={onClose}
                            class="px-4 py-2 bg-gray-200 hover:bg-gray-300 dark:bg-gray-700 dark:hover:bg-gray-600 text-sm font-medium text-gray-700 dark:text-gray-200"
                        >
                            Cancel
                        </button>
                        <button 
                            type="submit"
                            class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white text-sm font-medium"
                            disabled={proxy.highSpeed && !proxy.highSpeedAddr}
                        >
                            {isEditMode ? 'Update' : 'Create'}
                        </button>
                    </div>
                </form>
            </div>
        </div>
    </div>
{/if}