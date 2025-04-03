<script lang="ts">
    import GwNodeCard from "./GwNodeCard.svelte";
    import GwNodeModal from "./GwNodeModal.svelte";
    
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
    
    // Mock proxies data
    let proxies: Proxy[] = [
        { id: 1, title: "Main Proxy", listen: "0.0.0.0:8080" },
        { id: 2, title: "Secure API", listen: "0.0.0.0:443" },
        { id: 3, title: "Internal Service", listen: "127.0.0.1:9000" },
        { id: 4, title: "Legacy App", listen: "192.168.1.10:8000" },
        { id: 5, title: "Custom SSL", listen: "0.0.0.0:8443" },
    ];
    
    // Mock gwnode data
    let gwnodes: GwNode[] = [
        { id: 1, title: "API Gateway", proxyId: 1, proxyTitle: "Main Proxy", proxyListen: "0.0.0.0:8080", target: "192.168.1.100:3000" },
        { id: 2, title: "Web Server", proxyId: 2, proxyTitle: "Secure API", proxyListen: "0.0.0.0:443", target: "192.168.1.101:8080" },
        { id: 3, title: "Admin Panel", proxyId: 3, proxyTitle: "Internal Service", proxyListen: "127.0.0.1:9000", target: "192.168.1.102:8080" },
        { id: 4, title: "Database Access", proxyId: 4, proxyTitle: "Legacy App", proxyListen: "192.168.1.10:8000", target: "192.168.1.103:5432" },
        { id: 5, title: "Mail Server", proxyId: 5, proxyTitle: "Custom SSL", proxyListen: "0.0.0.0:8443", target: "192.168.1.104:25" },
        { id: 6, title: "File Server", proxyId: 1, proxyTitle: "Main Proxy", proxyListen: "0.0.0.0:8080", target: "192.168.1.105:21" },
        { id: 7, title: "Monitoring", proxyId: 3, proxyTitle: "Internal Service", proxyListen: "127.0.0.1:9000", target: "192.168.1.106:9090" },
        { id: 8, title: "Authentication", proxyId: 2, proxyTitle: "Secure API", proxyListen: "0.0.0.0:443", target: "192.168.1.107:8080" },
    ];
    
    // For search functionality
    export let searchTerm = "";
    $: filteredGwNodes = gwnodes.filter(gwnode => 
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
        if (isEditMode) {
            // Update existing gwnode
            const index = gwnodes.findIndex(n => n.id === currentGwNode.id);
            if (index !== -1) {
                gwnodes[index] = { ...currentGwNode };
            }
        } else {
            // Add new gwnode with the next available ID
            const newId = Math.max(...gwnodes.map(n => n.id), 0) + 1;
            gwnodes = [...gwnodes, { ...currentGwNode, id: newId }];
        }
        
        // Close the modal
        showGwNodeModal = false;
    }
    
    // Function to delete a gwnode
    function deleteGwNode(id: number): void {
        if (confirm("Are you sure you want to delete this gateway node?")) {
            gwnodes = gwnodes.filter(gwnode => gwnode.id !== id);
        }
    }
    
    // Close modal
    function closeModal(): void {
        showGwNodeModal = false;
    }
</script>

<div class="w-full max-w-[900px]">
    <div class="flex justify-between items-center mb-6">
        <h1 class="text-2xl font-bold">Gateway Nodes</h1>
        <button 
            on:click={addGwNode}
            class="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-md text-sm font-medium"
        >
            Add Gateway Node
        </button>
    </div>
    
    <!-- Search input -->
    <div class="mb-6">
        <input 
            type="text" 
            bind:value={searchTerm}
            placeholder="Search by title, target, or proxy..." 
            class="w-full p-2 rounded-md border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
        />
    </div>
    
    <!-- Card grid layout -->
    {#if visibleGwNodes.length === 0}
        <div class="text-center py-8 text-gray-500 dark:text-gray-400">
            No gateway nodes found
        </div>
    {:else}
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {#each visibleGwNodes as gwnode (gwnode.id)}
                <GwNodeCard {gwnode} onEdit={editGwNode} onDelete={deleteGwNode} />
            {/each}
        </div>
        
        <!-- Load more button -->
        {#if hasMoreToLoad}
            <div class="mt-6 text-center">
                <button 
                    on:click={loadMore}
                    class="px-4 py-2 bg-gray-200 hover:bg-gray-300 dark:bg-gray-700 dark:hover:bg-gray-600 rounded-md text-sm font-medium"
                >
                    Load more...
                </button>
            </div>
        {/if}
    {/if}
    
    <!-- GwNode Modal component -->
    <GwNodeModal 
        showModal={showGwNodeModal}
        isEditMode={isEditMode}
        gwnode={currentGwNode}
        {proxies}
        onSave={saveGwNode}
        onClose={closeModal}
    />
</div>