<script lang="ts">
    import { fade } from "svelte/transition";
    
    // Define Proxy interface for better type safety
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
    
    export let showModal: boolean = false;
    export let isEditMode: boolean = false;
    export let proxy: Proxy = { 
        id: 0, 
        title: "", 
        listen: "", 
        useTls: false, 
        autoTls: false, 
        certPem: "", 
        certKey: "",
        domain: "" 
    };
    export let onSave: () => void;
    export let onClose: () => void;
    
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
    
    // Enable/disable cert fields logic
    $: certFieldsDisabled = !proxy.useTls || proxy.autoTls;
</script>

{#if showModal}
    <div 
        class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50" 
        transition:fade={{ duration: 200 }}
        on:keydown={handleKeydown}
        role="presentation"
    >
        <div 
            class="bg-white dark:bg-[#161b22] rounded-lg shadow-xl max-w-md w-full mx-4"
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
                
                <form on:submit|preventDefault={onSave} class="space-y-4">
                    <div>
                        <label for="title" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                            Title
                        </label>
                        <input 
                            type="text" 
                            id="title" 
                            bind:value={proxy.title}
                            class="w-full p-2 rounded-md border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
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
                            class="w-full p-2 rounded-md border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
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
                            class="w-full p-2 rounded-md border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
                            placeholder="Example: example.com"
                        />
                        <p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
                            Server Name Indication for TLS connections
                        </p>
                    </div>
                    
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
                                    disabled={!proxy.useTls}
                                />
                                <label for="autoTls" class="ml-2 block text-sm text-gray-700 dark:text-gray-300">
                                    Auto TLS (Let's Encrypt)
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
                                    class="w-full p-2 rounded-md border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 font-mono text-sm"
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
                                    class="w-full p-2 rounded-md border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 font-mono text-sm"
                                    placeholder="this is cert"
                                    rows="4"
                                    disabled={certFieldsDisabled}
                                ></textarea>
                            </div>
                        </div>
                    {/if}
                    
                    <div class="flex justify-end space-x-2 pt-4">
                        <button 
                            type="button"
                            on:click={onClose}
                            class="px-4 py-2 bg-gray-200 hover:bg-gray-300 dark:bg-gray-700 dark:hover:bg-gray-600 rounded-md text-sm font-medium text-gray-700 dark:text-gray-200"
                        >
                            Cancel
                        </button>
                        <button 
                            type="submit"
                            class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-md text-sm font-medium"
                        >
                            {isEditMode ? 'Update' : 'Create'}
                        </button>
                    </div>
                </form>
            </div>
        </div>
    </div>
{/if}