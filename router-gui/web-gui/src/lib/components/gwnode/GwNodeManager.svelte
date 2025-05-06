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
    
    // For search functionality
    export let searchTerm = "";
    
    // Store subscriptions
    let gwnodeList: GwNode[] = [];
    let proxyList: Proxy[] = [];
    let isLoading = true;
    
    // Subscribe to stores
    const unsubGwNodes = gwNodes.subscribe(nodes => {
        gwnodeList = nodes;
    });
    
    const unsubProxies = proxies.subscribe(items => {
        proxyList = items.map(item => item.proxy);
    });
    
    // Load data when component mounts
    onMount(async () => {
        try {
            await gwnodeActions.loadAllGwNodes();
            proxyStore.fetchProxies();
            isLoading = false;
        } catch (error) {
            console.error("Failed to load gateway nodes:", error);
        }
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
        } catch (error: unknown) {
            console.error("Error saving gateway node:", error);
            alert(`Failed to save gateway node: ${error instanceof Error ? error.message : String(error)}`);
        }
    }
    
    // Function to delete a gwnode
    async function deleteGwNode(id: string): Promise<void> {
        if (confirm("Are you sure you want to delete this gateway node?")) {
            try {
                await gwnodeActions.deleteGwNode(id);
            } catch (error: unknown) {
                console.error("Error deleting gateway node:", error);
                alert(`Failed to delete gateway node: ${error instanceof Error ? error.message : String(error)}`);
            }
        }
    }
    
    // Function to sync gateway nodes with the server
    async function syncGatewayNodes(): Promise<void> {
        try {
            isLoading = true;
            const result = await gwnodeActions.syncGatewayNodes();
            isLoading = false;
            alert(result.message);
        } catch (error: unknown) {
            console.error("Error syncing gateway nodes:", error);
            alert(`Failed to sync gateway nodes: ${error instanceof Error ? error.message : String(error)}`);
            isLoading = false;
        }
    }
    
    // Close modal
    function closeModal(): void {
        showGwNodeModal = false;
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
    
    <!-- Search input -->
    <div class="mb-6">
        <SearchInput 
            bind:value={searchTerm} 
            placeholder="Search by title, target, or proxy..." 
        />
    </div>
    
    <!-- Card grid layout -->
    {#if isLoading}
        <LoadingSpinner />
    {:else if visibleGwNodes.length === 0}
        <EmptyState 
            message={searchTerm 
                ? "No gateway nodes match your search criteria" 
                : "No gateway nodes found"} 
            icon="search"
        />
    {:else}
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {#each visibleGwNodes as gwnode (gwnode.id)}
                <GwNodeCard {gwnode} onEdit={editGwNode} onDelete={deleteGwNode} />
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
    />
</div>