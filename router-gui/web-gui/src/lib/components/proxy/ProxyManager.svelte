<script lang="ts">
    import ProxyTable from "./ProxyTable.svelte";
    import ProxyModal from "./ProxyModal.svelte";
    import Pagination from "$lib/components/users/Pagination.svelte";
    
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
    
    // Pagination
    export let currentPage = 1;
    export let itemsPerPage = 5;
    $: totalPages = Math.ceil(filteredProxies.length / itemsPerPage);
    $: paginatedProxies = filteredProxies.slice(
        (currentPage - 1) * itemsPerPage,
        currentPage * itemsPerPage
    );
    
    // Reset to first page when search term changes
    $: if (searchTerm) {
        currentPage = 1;
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
            
            // If we're on a page that no longer has items, go to the previous page
            if (paginatedProxies.length === 1 && currentPage > 1) {
                currentPage--;
            }
        }
    }
    
    // Handle page change
    export function handlePageChange(page: number): void {
        currentPage = page;
    }
    
    // Close modal
    function closeModal(): void {
        showProxyModal = false;
    }
</script>

<div>
    <div class="flex justify-between items-center mb-6">
        <h1 class="text-2xl font-bold">Proxy Management</h1>
        <button 
            on:click={addProxy}
            class="bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-md text-sm font-medium"
        >
            Add Proxy
        </button>
    </div>
    
    <!-- Proxies Table component -->
    <ProxyTable 
        proxies={paginatedProxies} 
        onEdit={editProxy} 
        onDelete={deleteProxy} 
    />
    
    <!-- Pagination component -->
    <Pagination 
        currentPage={currentPage}
        totalPages={totalPages}
        totalItems={filteredProxies.length}
        itemsPerPage={itemsPerPage}
        onPageChange={handlePageChange}
    />
    
    <!-- Proxy Modal component -->
    <ProxyModal 
        showModal={showProxyModal}
        isEditMode={isEditMode}
        proxy={currentProxy}
        onSave={saveProxy}
        onClose={closeModal}
    />
</div>