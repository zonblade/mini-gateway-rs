<script lang="ts">
    import { createEventDispatcher } from "svelte";
    import type { Gateway } from "$lib/types/gateway";
    import Button from "$lib/components/common/Button.svelte";
    
    // Props
    export let showModal: boolean = false;
    export let isEditMode: boolean = false;
    export let gateway: Partial<Gateway> = {
        id: "",
        gwnode_id: "",
        pattern: "",
        target: "",
        priority: 100
    };
    export let gwnodeId: string = "";
    
    // Computed title based on mode
    $: title = isEditMode ? "Edit Routing Rule" : "Add Routing Rule";
    
    // Create a local copy of the gateway to edit
    let localGateway = { ...gateway };
    
    // Watch for prop changes to update local state
    $: if (gateway) {
        localGateway = { ...gateway };
    }
    
    // Event dispatcher for communicating with parent
    const dispatch = createEventDispatcher<{
        save: Partial<Gateway>;
        close: void;
    }>();
    
    // Function to save gateway
    function saveGateway(): void {
        // Always ensure the gwnode_id is set correctly
        if (!localGateway.gwnode_id && gwnodeId) {
            localGateway.gwnode_id = gwnodeId;
        }
        
        // Validate required fields
        if (!localGateway.pattern || !localGateway.target || localGateway.priority === undefined) {
            alert("Please fill in all required fields");
            return;
        }
        
        dispatch("save", localGateway);
    }
    
    // Function to close the modal
    function closeModal(): void {
        dispatch("close");
    }
    
    // Function to handle keyboard events for accessibility
    function handleKeydown(event: KeyboardEvent): void {
        if (event.key === "Escape") {
            closeModal();
        }
    }

    // Function to handle keyboard events for the modal container
    function handleModalKeydown(event: KeyboardEvent): void {
        if (event.key === "Escape" || event.key === "Tab") {
            event.stopPropagation();
        }
    }
</script>

{#if showModal}
    <!-- Modal backdrop -->
    <div 
        class="fixed inset-0 bg-black/30 backdrop-blur-sm z-50 flex justify-center items-center p-4"
        on:click|self={closeModal}
        on:keydown={handleKeydown}
        tabindex="0"
        aria-label="Close modal"
        role="dialog"
    >
        <!-- Modal container -->
        <div 
            class="bg-white dark:bg-[#1c2128] shadow-xl max-w-md w-full p-6"
            role="dialog"
            aria-modal="true"
            aria-labelledby="modal-title"
            on:click|stopPropagation={() => {}}
            on:keydown={handleModalKeydown}
            tabindex="0"
        >
            <!-- Modal header -->
            <div class="flex justify-between items-center mb-4">
                <h2 id="modal-title" class="text-xl font-semibold">{title}</h2>
                <button 
                    class="text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
                    on:click={closeModal}
                    aria-label="Close dialog"
                >
                    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
                    </svg>
                </button>
            </div>
            
            <!-- Modal content -->
            <div class="space-y-4">
                {#if isEditMode}
                    <div class="text-sm text-gray-500 dark:text-gray-400 mb-2">
                        ID: {localGateway.id}
                    </div>
                {/if}
                
                <input type="hidden" bind:value={localGateway.gwnode_id} />
                
                <div class="form-group">
                    <label for="pattern" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                        Pattern*
                    </label>
                    <input 
                        type="text" 
                        id="pattern" 
                        class="w-full px-3 py-2 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-700 shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500" 
                        placeholder="Path pattern (e.g., /api/users/* or ^/users/[0-9]+)" 
                        bind:value={localGateway.pattern}
                    />
                    <p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
                        Patterns support wildcard (*) and regex-like syntax (^/path/...)
                    </p>
                </div>
                
                <div class="form-group">
                    <label for="target" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                        Target*
                    </label>
                    <input 
                        type="text" 
                        id="target" 
                        class="w-full px-3 py-2 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-700 shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500" 
                        placeholder="Target URL (e.g., /v2/api/ or /internal/)" 
                        bind:value={localGateway.target}
                    />
                    <p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
                        Use $1, $2, etc. to reference capture groups from the pattern
                    </p>
                </div>
                
                <div class="form-group">
                    <label for="priority" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                        Priority*
                    </label>
                    <input 
                        type="number" 
                        id="priority" 
                        class="w-full px-3 py-2 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-700 shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500" 
                        placeholder="Priority (lower number = higher priority)" 
                        bind:value={localGateway.priority}
                    />
                    <p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
                        Lower numbers have higher priority (e.g. 10 is processed before 20)
                    </p>
                </div>
            </div>
            
            <!-- Modal footer -->
            <div class="flex justify-end mt-6 space-x-3">
                <Button 
                    variant="secondary"
                    onClick={closeModal}
                >
                    Cancel
                </Button>
                <Button 
                    variant="primary"
                    onClick={saveGateway}
                >
                    {isEditMode ? "Update" : "Create"}
                </Button>
            </div>
        </div>
    </div>
{/if}