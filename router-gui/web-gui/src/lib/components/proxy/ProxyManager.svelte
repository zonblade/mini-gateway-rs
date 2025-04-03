<script lang="ts">
    import ProxyCard from "./ProxyCard.svelte";
    import ProxyModal from "./ProxyModal.svelte";
    
    // Define Proxy interface for type safety
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
    // Mock proxy data for demonstration
    let proxies: Proxy[] = [
        { id: 1, title: "Main Proxy", listen: "0.0.0.0:8080", useTls: false, autoTls: false, certPem: "", certKey: "", domain: "" },
        { id: 2, title: "Secure API", listen: "0.0.0.0:443", useTls: true, autoTls: true, certPem: "", certKey: "", domain: "api.example.com" },
        { id: 3, title: "Internal Service", listen: "127.0.0.1:9000", useTls: false, autoTls: false, certPem: "", certKey: "", domain: "" },
        { id: 4, title: "Legacy App", listen: "192.168.1.10:8000", useTls: false, autoTls: false, certPem: "", certKey: "", domain: "" },
        { id: 5, title: "Custom SSL", listen: "0.0.0.0:8443", useTls: true, autoTls: false, certPem: "/path/to/cert.pem", certKey: "/path/to/key.pem", domain: "secure.example.com" },
    ];
    // For add/edit proxy popup
    let showProxyModal = false;
    let isEditMode = false;
    let currentProxy: Proxy = { 
        id: 0, 
        title: "", 
        listen: "", 
        useTls: false, 
        autoTls: false, 
        certPem: "", 
        certKey: "",
        domain: "" 
    };
    
    // Search functionality
    export let searchTerm = "";
    $: filteredProxies = proxies.filter(proxy => 
        proxy.title.toLowerCase().includes(searchTerm.toLowerCase()) ||
        proxy.listen.toLowerCase().includes(searchTerm.toLowerCase()) ||
        proxy.domain.toLowerCase().includes(searchTerm.toLowerCase())
    );
    
    // For "load more" functionality
    let visibleCount = 6;
    $: hasMoreToLoad = filteredProxies.length > visibleCount;
    $: visibleProxies = filteredProxies.slice(0, visibleCount);
    
    function loadMore(): void {
        visibleCount += 6;
    }
    
    // Reset visible count when search term changes
    $: if (searchTerm) {
        visibleCount = 6;
    }
    
    // Function to open modal for adding a new proxy
    function addProxy(): void {
        currentProxy = { 
            id: 0, 
            title: "", 
            listen: "", 
            useTls: false, 
            autoTls: false, 
            certPem: "", 
            certKey: "",
            domain: "" 
        };
        isEditMode = false;
        showProxyModal = true;
    }
    
    // Function to open modal for editing an existing proxy
    function editProxy(proxy: Proxy): void {
        currentProxy = { ...proxy };
        isEditMode = true;
        showProxyModal = true;
    }
    
    // Function to save proxy (create or update)
    function saveProxy(): void {
        if (isEditMode) {
            // Update existing proxy
            const index = proxies.findIndex(p => p.id === currentProxy.id);
            if (index !== -1) {
                proxies[index] = { ...currentProxy };
            }
        } else {
            // Add new proxy with the next available ID
            const newId = Math.max(...proxies.map(p => p.id), 0) + 1;
            proxies = [...proxies, { ...currentProxy, id: newId }];
        }
        
        // Close the modal
        showProxyModal = false;
    }
    
    // Function to delete a proxy
    function deleteProxy(id: number): void {
        if (confirm("Are you sure you want to delete this proxy?")) {
            proxies = proxies.filter(proxy => proxy.id !== id);
        }
    }
    
    // Close modal
    function closeModal(): void {
        showProxyModal = false;
    }
</script>
<div class="w-full max-w-[900px]">
    <div class="flex justify-between items-center mb-6">
        <h1 class="text-2xl font-bold">Proxy Management</h1>
        <button 
            on:click={addProxy}
            class="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-md text-sm font-medium"
        >
            Add Proxy
        </button>
    </div>
    
    <!-- Search input -->
    <div class="mb-6">
        <input 
            type="text" 
            bind:value={searchTerm}
            placeholder="Search by title, listen address, or domain..." 
            class="w-full p-2 rounded-md border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
        />
    </div>
    
    <!-- Card grid layout -->
    {#if visibleProxies.length === 0}
        <div class="text-center py-8 text-gray-500 dark:text-gray-400">
            No proxies found
        </div>
    {:else}
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {#each visibleProxies as proxy (proxy.id)}
                <ProxyCard {proxy} onEdit={editProxy} onDelete={deleteProxy} />
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
    
    <!-- Proxy Modal component -->
    <ProxyModal 
        showModal={showProxyModal}
        isEditMode={isEditMode}
        proxy={currentProxy}
        onSave={saveProxy}
        onClose={closeModal}
    />
</div>