<script lang="ts">
    import { gwNodes, type GwNode } from "$lib/stores/gwnodeStore";
    import { proxies, type Proxy } from "$lib/stores/proxyStore";
    import GwNodeCard from "./GwNodeCard.svelte";
    import GwNodeModal from "./GwNodeModal.svelte";
    import LoadingSpinner from "$lib/components/common/LoadingSpinner.svelte";
    import SearchInput from "$lib/components/common/SearchInput.svelte";
    import PageHeader from "$lib/components/common/PageHeader.svelte";
    import EmptyState from "$lib/components/common/EmptyState.svelte";
    import Button from "$lib/components/common/Button.svelte";
    
    // For search functionality
    export let searchTerm = "";
    
    // Store subscriptions
    let gwnodeList: GwNode[] = [];
    let proxyList: Proxy[] = [];
    
    // Subscribe to stores
    const unsubGwNodes = gwNodes.subscribe(nodes => {
        gwnodeList = nodes;
    });
    
    const unsubProxies = proxies.subscribe(items => {
        proxyList = items;
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
        gwnode.target.toLowerCase().includes(searchTerm.toLowerCase()) ||
        gwnode.proxyTitle.toLowerCase().includes(searchTerm.toLowerCase())
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
        id: 0,
        title: "",
        proxyId: 0,
        proxyTitle: "",
        proxyListen: "",
        target: ""
    };
    
    // Function to open modal for adding a new gwnode
    function addGwNode(): void {
        currentGwNode = {
            id: 0,
            title: "",
            proxyId: 0,
            proxyTitle: "",
            proxyListen: "",
            target: ""
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
    function saveGwNode(): void {
        gwNodes.update(nodes => {
            if (isEditMode) {
                // Update existing gwnode
                const index = nodes.findIndex(n => n.id === currentGwNode.id);
                if (index !== -1) {
                    nodes[index] = { ...currentGwNode };
                }
            } else {
                // Add new gwnode with the next available ID
                const newId = Math.max(...nodes.map(n => n.id), 0) + 1;
                nodes = [...nodes, { ...currentGwNode, id: newId }];
            }
            return nodes;
        });
        
        // Close the modal
        showGwNodeModal = false;
    }
    
    // Function to delete a gwnode
    function deleteGwNode(id: number): void {
        if (confirm("Are you sure you want to delete this gateway node?")) {
            gwNodes.update(nodes => nodes.filter(gwnode => gwnode.id !== id));
        }
    }
    
    // Close modal
    function closeModal(): void {
        showGwNodeModal = false;
    }
</script>

<div class="w-full max-w-[900px]">
    <PageHeader 
        title="Gateway Nodes" 
        hasAction={true} 
        actionLabel="Add Gateway Node" 
        onAction={addGwNode} 
    />
    
    <!-- Search input -->
    <div class="mb-6">
        <SearchInput 
            bind:value={searchTerm} 
            placeholder="Search by title, target, or proxy..." 
        />
    </div>
    
    <!-- Card grid layout -->
    {#if !gwnodeList.length}
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
        gwnode={currentGwNode}
        proxies={proxyList}
        onSave={saveGwNode}
        onClose={closeModal}
    />
</div>