<script lang="ts">
    import { onMount } from "svelte";
    import { gatewayActions } from "$lib/actions/gatewayActions";
    import { gateways } from "$lib/stores/gatewayStore";
    import type { Gateway } from "$lib/types/gateway";
    import GatewayModal from "./GatewayModal.svelte";
    import Button from "$lib/components/common/Button.svelte";
    import EmptyState from "$lib/components/common/EmptyState.svelte";
    import LoadingSpinner from "$lib/components/common/LoadingSpinner.svelte";
    import SearchInput from "$lib/components/common/SearchInput.svelte";

    // Props
    export let gwnodeId: string;
    // Export as const since it's only for external reference
    export const gwnodeTitle: string = "";

    // State
    let isLoading = true;
    let gatewayList: Gateway[] = [];
    let searchTerm = "";
    let showModal = false;
    let isEditMode = false;
    let currentGateway: Partial<Gateway> = {
        id: "",
        gwnode_id: gwnodeId,
        pattern: "",
        target: "",
        priority: 100
    };

    // Subscribe to the gateways store
    const unsubGateways = gateways.subscribe(items => {
        gatewayList = items;
    });

    // Load gateways when component mounts
    onMount(() => {
        loadGateways();
        
        // Return the cleanup function
        return () => {
            unsubGateways(); // Clean up subscription
        };
    });

    // Function to load gateways
    async function loadGateways(): Promise<void> {
        try {
            await gatewayActions.loadGatewaysByGwNodeId(gwnodeId);
            isLoading = false;
        } catch (error) {
            console.error(`Failed to load gateways for node ${gwnodeId}:`, error);
            isLoading = false;
        }
    }

    // Filtered gateways based on search term
    $: filteredGateways = gatewayList.filter(gateway => 
        gateway.pattern.toLowerCase().includes(searchTerm.toLowerCase()) ||
        gateway.target.toLowerCase().includes(searchTerm.toLowerCase())
    );

    // Sorted gateways by priority (ascending)
    $: sortedGateways = [...filteredGateways].sort((a, b) => a.priority - b.priority);

    // Add gateway
    function addGateway(): void {
        currentGateway = {
            id: "",
            gwnode_id: gwnodeId,
            pattern: "",
            target: "",
            priority: 100
        };
        isEditMode = false;
        showModal = true;
    }

    // Edit gateway
    function editGateway(gateway: Gateway): void {
        currentGateway = { ...gateway };
        isEditMode = true;
        showModal = true;
    }

    // Delete gateway
    async function deleteGateway(id: string): Promise<void> {
        if (confirm("Are you sure you want to delete this routing rule?")) {
            try {
                await gatewayActions.deleteGateway(id);
            } catch (error) {
                console.error(`Error deleting gateway ${id}:`, error);
                alert(`Failed to delete routing rule: ${error instanceof Error ? error.message : String(error)}`);
            }
        }
    }

    // Save gateway from modal
    async function saveGateway(event: CustomEvent<Partial<Gateway>>): Promise<void> {
        try {
            const gatewayData = event.detail;
            
            if (isEditMode && gatewayData.id) {
                // Update existing gateway
                await gatewayActions.updateGateway({
                    id: gatewayData.id,
                    gwnode_id: gatewayData.gwnode_id || gwnodeId,
                    pattern: gatewayData.pattern || "",
                    target: gatewayData.target || "",
                    priority: gatewayData.priority || 100
                });
            } else {
                // Create new gateway
                await gatewayActions.createGateway({
                    id: "",
                    gwnode_id: gatewayData.gwnode_id || gwnodeId,
                    pattern: gatewayData.pattern || "",
                    target: gatewayData.target || "",
                    priority: gatewayData.priority || 100
                });
            }
            
            // Close modal
            showModal = false;
        } catch (error) {
            console.error("Error saving gateway:", error);
            alert(`Failed to save routing rule: ${error instanceof Error ? error.message : String(error)}`);
        }
    }

    // Close modal
    function closeModal(): void {
        showModal = false;
    }
</script>

<div class="p-6 border-t border-gray-200 dark:border-gray-700">
    <div class="flex justify-between items-center mb-4">
        <h2 class="text-xl font-bold">Gateway Routing Rules</h2>
        <Button variant="primary" onClick={addGateway}>Add Routing Rule</Button>
    </div>

    <!-- Search input -->
    <div class="mb-4">
        <SearchInput 
            bind:value={searchTerm} 
            placeholder="Search by pattern or target..."
        />
    </div>

    <!-- Gateway table -->
    {#if isLoading}
        <div class="flex justify-center items-center py-10">
            <LoadingSpinner />
        </div>
    {:else if sortedGateways.length === 0}
        <EmptyState 
            message={searchTerm 
                ? "No routing rules match your search criteria" 
                : "No routing rules found for this gateway node"} 
            icon="search"
        />
    {:else}
        <div class="overflow-x-auto">
            <table class="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
                <thead class="bg-gray-50 dark:bg-gray-800">
                    <tr>
                        <th scope="col" class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                            Priority
                        </th>
                        <th scope="col" class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                            Pattern
                        </th>
                        <th scope="col" class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                            Target
                        </th>
                        <th scope="col" class="px-4 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                            Actions
                        </th>
                    </tr>
                </thead>
                <tbody class="bg-white dark:bg-gray-900 divide-y divide-gray-200 dark:divide-gray-800">
                    {#each sortedGateways as gateway (gateway.id)}
                        <tr class="hover:bg-gray-50 dark:hover:bg-gray-800">
                            <td class="px-4 py-3 whitespace-nowrap text-sm font-medium">
                                <span class="px-2 py-1 bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200 ">
                                    {gateway.priority}
                                </span>
                            </td>
                            <td class="px-4 py-3 text-sm font-mono">
                                {gateway.pattern}
                            </td>
                            <td class="px-4 py-3 text-sm font-mono">
                                {gateway.target}
                            </td>
                            <td class="px-4 py-3 whitespace-nowrap text-right text-sm font-medium">
                                <div class="flex justify-end space-x-2">
                                    <button 
                                        class="text-blue-600 hover:text-blue-800 dark:text-blue-400 dark:hover:text-blue-300"
                                        on:click={() => editGateway(gateway)}
                                        aria-label="Edit rule"
                                    >
                                        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 0L11.828 15H9v-2.828l8.586-8.586z" />
                                        </svg>
                                    </button>
                                    <button 
                                        class="text-red-600 hover:text-red-800 dark:text-red-400 dark:hover:text-red-300"
                                        on:click={() => deleteGateway(gateway.id)}
                                        aria-label="Delete rule"
                                    >
                                        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                                        </svg>
                                    </button>
                                </div>
                            </td>
                        </tr>
                    {/each}
                </tbody>
            </table>
        </div>
    {/if}
</div>

<!-- Gateway modal -->
<GatewayModal
    {showModal}
    {isEditMode}
    gateway={currentGateway}
    {gwnodeId}
    on:save={saveGateway}
    on:close={closeModal}
/>