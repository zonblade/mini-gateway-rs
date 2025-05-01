<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import ProxyCard from "./ProxyCard.svelte";
    import ProxyModal from "./ProxyModal.svelte";
    import LoadingSpinner from "$lib/components/common/LoadingSpinner.svelte";
    import SearchInput from "$lib/components/common/SearchInput.svelte";
    import EmptyState from "$lib/components/common/EmptyState.svelte";
    import Button from "$lib/components/common/Button.svelte";
    import { proxyStore } from "$lib/stores/proxyStore";
    import type { Proxy } from "$lib/types/proxy";
    import gwnodeActions from "$lib/actions/gwnodeActions";
    
    // For adapting our API Proxy type to match the UI's expected format
    interface UIProxy {
        id: string; // Changed to string
        title: string;
        listen: string;
        useTls: boolean;
        autoTls: boolean;
        certPem: string;
        certKey: string;
        domain: string;
        target: string;
        highSpeed: boolean;
        highSpeedAddr: string;
    }
    
    // Function to convert API Proxy to UI Proxy format
    function apiToUiProxy(proxy: Proxy): UIProxy {
        return {
            id: proxy.id || '', // Use string ID, default to empty string
            title: proxy.title,
            listen: proxy.addr_listen,
            useTls: proxy.tls,
            autoTls: proxy.tls_autron,
            certPem: proxy.tls_pem || "",
            certKey: proxy.tls_key || "",
            domain: proxy.sni || "",
            target: proxy.addr_target || "",
            highSpeed: proxy.high_speed || false,
            highSpeedAddr: proxy.high_speed_addr || ""
        };
    }
    
    // Function to convert UI Proxy back to API format
    function uiToApiProxy(uiProxy: UIProxy): Proxy {
        // For new proxies (id === ''), send empty string as ID to ensure the backend treats it as new
        return {
            id: uiProxy.id, // Directly use the string ID
            title: uiProxy.title,
            addr_listen: uiProxy.listen,
            addr_target: "", // Always send empty string as addr_target
            tls: uiProxy.useTls,
            tls_pem: uiProxy.certPem || null,
            tls_key: uiProxy.certKey || null,
            tls_autron: uiProxy.autoTls,
            sni: uiProxy.domain || null,
            high_speed: uiProxy.highSpeed || false,
            high_speed_addr: uiProxy.highSpeedAddr || null
        };
    }
    
    // For search functionality
    export let searchTerm = "";
    
    // Store subscriptions
    let apiProxies: Proxy[] = [];
    let uiProxies: UIProxy[] = [];
    let isLoading = true;
    
    // Subscribe to the store
    const unsubProxy = proxyStore.subscribe(state => {
        apiProxies = state.proxies;
        uiProxies = apiProxies.map(apiToUiProxy);
        isLoading = state.loading;
    });
    
    // Fetch proxies on component mount
    onMount(() => {
        proxyStore.fetchProxies();
    });
    
    // Cleanup subscriptions on component destroy
    onDestroy(() => {
        unsubProxy();
    });
    
    // Filtered proxies based on search term
    $: filteredProxies = uiProxies.filter(proxy => 
        proxy.title.toLowerCase().includes(searchTerm.toLowerCase()) ||
        proxy.listen.toLowerCase().includes(searchTerm.toLowerCase()) ||
        (proxy.domain && proxy.domain.toLowerCase().includes(searchTerm.toLowerCase())) ||
        (proxy.target && proxy.target.toLowerCase().includes(searchTerm.toLowerCase()))
    );
    
    // For "load more" functionality
    let visibleCount = 6;
    $: hasMoreToLoad = filteredProxies.length > visibleCount;
    $: visibleProxies = filteredProxies.slice(0, visibleCount);
    
    function loadMore(): void {
        visibleCount += 6;
    }
    
    // For add/edit proxy modal
    let showProxyModal = false;
    let isEditMode = false;
    let currentProxy: UIProxy = { 
        id: '', // Changed default ID to empty string
        title: "",
        listen: "",
        useTls: false,
        autoTls: false,
        certPem: "",
        certKey: "",
        domain: "",
        target: "",
        highSpeed: false,
        highSpeedAddr: ""
    };
    
    // Function to open modal for adding a new proxy
    function addProxy(): void {
        currentProxy = { 
            id: '', // Changed default ID to empty string
            title: "", 
            listen: "", 
            useTls: false, 
            autoTls: false, 
            certPem: "", 
            certKey: "",
            domain: "",
            target: "",
            highSpeed: false,
            highSpeedAddr: ""
        };
        isEditMode = false;
        showProxyModal = true;
        console.log("Opening modal for new proxy, isEditMode:", isEditMode);
    }
    
    // Function to open modal for editing an existing proxy
    function editProxy(proxy: UIProxy): void {
        currentProxy = { ...proxy };
        isEditMode = true;
        showProxyModal = true;
        console.log("Opening modal for editing proxy ID:", proxy.id, "isEditMode:", isEditMode);
    }
    
    // Function to save proxy (create or update)
    async function saveProxy(): Promise<void> {
        // Convert UI proxy to API format
        const apiProxy = uiToApiProxy(currentProxy);
        
        try {
            await proxyStore.saveProxy(apiProxy);
            showProxyModal = false;
        } catch (error) {
            console.error('Error saving proxy:', error);
            alert('Failed to save proxy: ' + (error instanceof Error ? error.message : String(error)));
        }
    }
    
    // Function to delete a proxy
    async function deleteProxy(id: string): Promise<void> { // id is already string
        if (confirm("Are you sure you want to delete this proxy?")) {
            try {
                await proxyStore.deleteProxy(id);
                // Refetch the list after successful deletion
                await proxyStore.fetchProxies(); 
            } catch (error) {
                console.error('Error deleting proxy:', error);
                alert('Failed to delete proxy: ' + (error instanceof Error ? error.message : String(error)));
            }
        }
    }
    
    // Function to sync proxies with the server
    async function syncProxies(): Promise<void> {
        try {
            const result = await proxyStore.syncProxies();
            await gwnodeActions.syncGatewayNodes();
            alert(result.message);
        } catch (error) {
            console.error('Error syncing proxies:', error);
            alert('Failed to sync proxies: ' + (error instanceof Error ? error.message : String(error)));
        }
    }
    
    // Close modal
    function closeModal(): void {
        showProxyModal = false;
    }
</script>

<div class="w-full max-w-[900px]">
    <div class="flex justify-between items-center mb-6">
        <h1 class="text-2xl font-bold text-gray-900 dark:text-white">Proxy Management</h1>
        <div class="flex space-x-2">
            <Button 
                variant="secondary" 
                onClick={syncProxies}
            >
                Sync Nodes
            </Button>
            <Button 
                variant="primary" 
                onClick={addProxy}
            >
                Add Proxy
            </Button>
        </div>
    </div>
    
    <!-- Search input -->
    <div class="mb-6">
        <SearchInput 
            bind:value={searchTerm} 
            placeholder="Search by title, address, or domain..." 
        />
    </div>
    
    <!-- Card grid layout -->
    {#if isLoading}
        <LoadingSpinner />
    {:else if visibleProxies.length === 0}
        <div class="text-center py-8">
            <EmptyState 
                message={searchTerm 
                    ? "No proxies match your search criteria" 
                    : "No proxies found"
                } 
                icon="search"
            />
            {#if !searchTerm}
                <div class="mt-4">
                    <Button 
                        variant="primary" 
                        onClick={addProxy}
                    >
                        Create your first proxy
                    </Button>
                </div>
            {/if}
        </div>
    {:else}
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {#each visibleProxies as proxy (proxy.id)}
                <div>
                    <ProxyCard 
                        proxy={uiToApiProxy(proxy)} 
                        onEdit={() => editProxy(proxy)} 
                        onDelete={() => deleteProxy(proxy.id)}
                    />
                </div>
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
    
    <!-- Proxy Modal component -->
    {#if showProxyModal}
        <ProxyModal 
            bind:showModal={showProxyModal}
            isEditMode={isEditMode}
            proxy={currentProxy}
            onSave={saveProxy}
            onClose={closeModal}
        />
    {/if}
</div>