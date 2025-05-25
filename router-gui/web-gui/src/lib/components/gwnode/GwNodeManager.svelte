<script lang="ts">
    import { onMount } from "svelte";
    import { gwNodes } from "$lib/stores/gwnodeStore";
    import { proxies, proxyStore } from "$lib/stores/proxyStore";
    import type { Proxy } from "$lib/types/proxy";
    import { gwnodeActions } from "$lib/actions/gwnodeActions";
    import type { CreateGwNodeRequest, UpdateGwNodeRequest, GwNode } from "$lib/types/gwnode";
    import GwNodeCard from "./GwNodeCard.svelte";
    import GwNodeModal from "./GwNodeModal.svelte";
    import LoadingSpinner from "$lib/components/common/LoadingSpinner.svelte";
    import SearchInput from "$lib/components/common/SearchInput.svelte";
    import EmptyState from "$lib/components/common/EmptyState.svelte";
    import Button from "$lib/components/common/Button.svelte";
    import Swal from "sweetalert2";
    import DeleteConfirmationModal from "$lib/components/common/DeleteConfirmationModal.svelte";
    
    // For search functionality
    export let searchTerm = "";
    
    // Store subscriptions
    let gwnodeList: GwNode[] = [];
    let proxyList: Proxy[] = [];
    let isLoading = true;
    let loadError: string | null = null;
    
    // Add these variables
    let showDeleteModal = false;
    let gwnodeToDelete: { id: string; target: string } | null = null;
    let isProcessing = false;
    let errorMessage: string | null = null;
    let modalErrorMessage: string | null = null;
    
    // Subscribe to stores
    const unsubGwNodes = gwNodes.subscribe(nodes => {
        gwnodeList = nodes;
    });
    
    const unsubProxies = proxies.subscribe(items => {
        proxyList = items.map(item => item.proxy);
    });
    
    // Function to load data - can be called both on mount and for retrying
    async function loadData(): Promise<void> {
        try {
            isLoading = true;
            loadError = null;
            await gwnodeActions.loadAllGwNodes();
            await proxyStore.fetchProxies();
            isLoading = false;
        } catch (error) {
            console.error("Failed to load gateway nodes:", error);
            loadError = error instanceof Error ? error.message : "Failed to load data";
            isLoading = false; // Important: Set isLoading to false even on error
        }
    }
    
    // Call loadData when component mounts
    onMount(() => {
        loadData();
    });
    
    // Cleanup subscriptions on component destroy
    import { onDestroy } from "svelte";
    onDestroy(() => {
        unsubGwNodes();
        unsubProxies();
    });
    
    // Filtered nodes based on search term
    $: filteredGwNodes = gwnodeList.filter(gwnode => 
        gwnode.title.toLowerCase().includes(searchTerm.toLowerCase()) ||
        gwnode.alt_target.toLowerCase().includes(searchTerm.toLowerCase()) ||
        (gwnode.proxyTitle && gwnode.proxyTitle.toLowerCase().includes(searchTerm.toLowerCase()))
    );
    
    // For "load more" functionality
    let visibleCount = 6;
    $: hasMoreToLoad = filteredGwNodes.length > visibleCount;
    $: visibleGwNodes = filteredGwNodes.slice(0, visibleCount);
    
    function loadMore(): void {
        visibleCount += 6;
    }
    
    // For add/edit gwnode modal
    let showGwNodeModal = false;
    let isEditMode = false;
    let currentGwNode: GwNode = {
        id: "",
        title: "",
        proxy_id: "",
        proxyTitle: "",
        alt_target: "",
        source: "",
    };
    
    // Function to open modal for adding a new gwnode
    function addGwNode(): void {
        currentGwNode = {
            id: "",
            title: "",
            proxy_id: "",
            proxyTitle: "",
            alt_target: "",
            source: "",
        };
        isEditMode = false;
        showGwNodeModal = true;
    }
    
    // Function to open modal for editing an existing gwnode
    function editGwNode(gwnode: GwNode): void {
        currentGwNode = { ...gwnode };
        isEditMode = true;
        showGwNodeModal = true;
    }
    
    // Function to save gwnode (create or update)
    async function saveGwNode(): Promise<void> {
        try {
            modalErrorMessage = null;
            if (isEditMode) {
                // Update existing gwnode
                const updateRequest: UpdateGwNodeRequest = {
                    id: currentGwNode.id,
                    proxy_id: currentGwNode.proxy_id,
                    title: currentGwNode.title,
                    alt_target: currentGwNode.alt_target,
                    source: "", // Include empty source when updating
                    domain_id: currentGwNode.domain_id || undefined
                };
                await gwnodeActions.updateGwNode(updateRequest);
            } else {
                // Create new gwnode
                const createRequest: CreateGwNodeRequest = {
                    id: "", // Include empty ID for new nodes
                    proxy_id: currentGwNode.proxy_id,
                    title: currentGwNode.title,
                    alt_target: currentGwNode.alt_target,
                    source: "", // Include empty source for new nodes
                    domain_id: currentGwNode.domain_id || undefined
                };
                await gwnodeActions.createGwNode(createRequest);
            }
            
            // Close the modal
            showGwNodeModal = false;
        } catch (error: any) {
            console.error("Error saving gateway node:", error);
            modalErrorMessage = error.error??error;
        }
    }
    
    // Function to delete a gwnode
    async function deleteGwNode(id: string, target: string) {
        gwnodeToDelete = { id, target };
        showDeleteModal = true;
    }
    
    async function handleDeleteConfirm() {
        if (!gwnodeToDelete) return;
        
        try {
            isProcessing = true;
            await gwnodeActions.deleteGwNode(gwnodeToDelete.id);
            await loadData();
            showDeleteModal = false;
            gwnodeToDelete = null;
        } catch (error) {
            console.error('Error deleting gateway node:', error);
            errorMessage = `Failed to delete gateway node: ${error instanceof Error ? error.message : String(error)}`;
        } finally {
            isProcessing = false;
        }
    }
    
    function handleDeleteCancel() {
        showDeleteModal = false;
        gwnodeToDelete = null;
    }
    
    // Function to sync gateway nodes with the server
    async function syncGatewayNodes(): Promise<void> {
        try {
            // Show loading indicator
            Swal.fire({
                title: 'Syncing...',
                text: 'Please wait while we sync gateway nodes',
                allowOutsideClick: false,
                didOpen: () => {
                    Swal.showLoading();
                }
            });
            
            isLoading = true;
            loadError = null;
            const result = await gwnodeActions.syncGatewayNodes();
            isLoading = false;
            
            // Show success message
            await Swal.fire({
                title: 'Success!',
                text: result.message,
                icon: 'success',
                timer: 2000,
                showConfirmButton: false
            });
        } catch (error: unknown) {
            console.error("Error syncing gateway nodes:", error);
            // loadError = error instanceof Error ? error.message : "Failed to sync nodes";
            isLoading = false;
            
            // Show error message
            await Swal.fire({
                title: 'Sync Failed',
                text: `Failed to sync gateway nodes: ${error instanceof Error ? error.message : String(error)}`,
                icon: 'error'
            });
        }
    }
    
    // Function to retry loading after an error
    function retryLoading(): void {
        loadData();
    }
    
    // Close modal
    function closeModal(): void {
        showGwNodeModal = false;
        modalErrorMessage = null;
    }
</script>

<div class="w-full max-w-[900px]">
    <div class="flex justify-between items-center mb-6">
        <h1 class="text-2xl font-bold text-gray-900 dark:text-white">Gateway Nodes</h1>
        <div class="flex space-x-2">
            <Button 
                variant="secondary" 
                onClick={syncGatewayNodes}
            >
                Sync Nodes
            </Button>
            <Button 
                variant="primary" 
                onClick={addGwNode}
            >
                Add Gateway Node
            </Button>
        </div>
    </div>
    
    <!-- Search input (show only when data is loaded and no errors) -->
    {#if !isLoading && !loadError}
        <div class="mb-6">
            <SearchInput 
                bind:value={searchTerm} 
                placeholder="Search by title, target, or proxy..." 
            />
        </div>
    {/if}
    
    <!-- Main content area -->
    {#if isLoading}
        <div class="flex justify-center items-center py-16">
            <LoadingSpinner />
        </div>
    {:else if loadError}
        <!-- Error state -->
        <div class="flex flex-col items-center justify-center py-12 text-center">
            <div class="text-red-500 mb-4">
                <svg xmlns="http://www.w3.org/2000/svg" class="h-12 w-12 mx-auto mb-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                </svg>
                <h3 class="text-lg font-medium">Failed to load gateway nodes</h3>
                <p class="text-sm mt-1">{loadError}</p>
            </div>
            <div class="flex space-x-4">
                <Button 
                    variant="secondary" 
                    onClick={retryLoading}
                >
                    Retry
                </Button>
                <Button 
                    variant="primary" 
                    onClick={addGwNode}
                >
                    Create Gateway Node
                </Button>
            </div>
        </div>
    {:else if visibleGwNodes.length === 0}
        <!-- Empty state -->
        <div class="flex flex-col items-center justify-center py-12 text-center">
            <EmptyState 
                message={searchTerm 
                    ? "No gateway nodes match your search criteria" 
                    : "No gateway nodes found"} 
                icon={searchTerm ? "search" : "search"}
            />
            {#if !searchTerm}
                <div class="mt-6">
                    <Button 
                        variant="primary" 
                        onClick={addGwNode}
                    >
                        Create Gateway Node
                    </Button>
                </div>
            {:else}
                <div class="mt-4">
                    <Button 
                        variant="secondary" 
                        onClick={() => searchTerm = ""}
                    >
                        Clear Search
                    </Button>
                </div>
            {/if}
        </div>
    {:else}
        <!-- Card grid layout -->
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {#each visibleGwNodes as gwnode (gwnode.id)}
                <GwNodeCard 
                    {gwnode} 
                    onEdit={editGwNode} 
                    onDelete={(id) => deleteGwNode(id, gwnode.alt_target)} 
                />
            {/each}
        </div>
        
        <!-- Load more button -->
        {#if hasMoreToLoad}
            <div class="mt-6 text-center">
                <Button 
                    variant="secondary" 
                    onClick={loadMore}
                >
                    Load more...
                </Button>
            </div>
        {/if}
    {/if}
    
    <!-- GwNode Modal component -->
    <GwNodeModal 
        showModal={showGwNodeModal}
        isEditMode={isEditMode}
        bind:gwnode={currentGwNode}
        proxies={proxyList}
        onSave={saveGwNode}
        onClose={closeModal}
        errorMessage={modalErrorMessage}
    />
    
    <!-- Delete Confirmation Modal component -->
    <DeleteConfirmationModal
        showModal={showDeleteModal}
        type="gwnode"
        addressToVerify={gwnodeToDelete?.target || ''}
        {isProcessing}
        on:confirm={handleDeleteConfirm}
        on:cancel={handleDeleteCancel}
    />
</div>