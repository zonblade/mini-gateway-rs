<script lang="ts">
    import { fade } from "svelte/transition";
    
    // Define interfaces
    interface GwNode {
        id: number;
        title: string;
        proxyId: number;
        proxyTitle: string;
        proxyListen: string;
        target: string;
    }
    
    interface Proxy {
        id: number;
        title: string;
        listen: string;
    }
    
    export let showModal: boolean = false;
    export let isEditMode: boolean = false;
    export let gwnode: GwNode = {
        id: 0,
        title: "",
        proxyId: 0,
        proxyTitle: "",
        proxyListen: "",
        target: ""
    };
    export let proxies: Proxy[] = [];
    export let onSave: () => void;
    export let onClose: () => void;
    
    // Selected proxy information
    $: {
        if (gwnode.proxyId) {
            const selectedProxy = proxies.find(p => p.id === gwnode.proxyId);
            if (selectedProxy) {
                gwnode.proxyTitle = selectedProxy.title;
                gwnode.proxyListen = selectedProxy.listen;
            }
        }
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
                    <h2 id="modal-title" class="text-xl font-bold">{isEditMode ? 'Edit Gateway Node' : 'Add Gateway Node'}</h2>
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
                            bind:value={gwnode.title}
                            class="w-full p-2 rounded-md border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
                            required
                            placeholder="My Gateway Node"
                        />
                    </div>
                    
                    <div>
                        <label for="proxyId" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                            Proxy
                        </label>
                        <select 
                            id="proxyId" 
                            bind:value={gwnode.proxyId}
                            class="w-full p-2 rounded-md border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
                            required
                        >
                            <option value={0} disabled>Select a proxy</option>
                            {#each proxies as proxy}
                                <option value={proxy.id}>{proxy.title}</option>
                            {/each}
                        </select>
                    </div>
                    
                    {#if gwnode.proxyId && gwnode.proxyTitle && gwnode.proxyListen}
                        <div class="rounded-md bg-gray-50 dark:bg-gray-800 p-3">
                            <h3 class="text-sm font-medium text-gray-700 dark:text-gray-300">Selected Proxy</h3>
                            <div class="mt-1 text-sm text-gray-500 dark:text-gray-400">
                                <div>{gwnode.proxyTitle}</div>
                                <div class="font-mono text-xs">{gwnode.proxyListen}</div>
                            </div>
                        </div>
                    {/if}
                    
                    <div>
                        <label for="target" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                            Target (IP:Port)
                        </label>
                        <input 
                            type="text" 
                            id="target" 
                            bind:value={gwnode.target}
                            class="w-full p-2 rounded-md border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
                            required
                            placeholder="Example: 192.168.1.10:8080"
                        />
                    </div>
                    
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