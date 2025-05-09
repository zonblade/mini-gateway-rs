<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import { goto } from "$app/navigation";
    import ProxyCard from "./ProxyCard.svelte";
    import LoadingSpinner from "$lib/components/common/LoadingSpinner.svelte";
    import SearchInput from "$lib/components/common/SearchInput.svelte";
    import EmptyState from "$lib/components/common/EmptyState.svelte";
    import Button from "$lib/components/common/Button.svelte";
    import { proxyStore } from "$lib/stores/proxyStore";
    import type { Proxy, ProxyWithDomains, TlsDomain } from "$lib/types/proxy";
    import gwnodeActions from "$lib/actions/gwnodeActions";
    
    // For search functionality
    export let searchTerm = "";
    
    // Store subscriptions
    let proxiesWithDomains: ProxyWithDomains[] = [];
    let isLoading = true;
    
    // Subscribe to the store
    const unsubProxy = proxyStore.subscribe(state => {
        proxiesWithDomains = state.proxies;
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
    $: filteredProxies = searchTerm 
        ? proxiesWithDomains.filter(item => {
            const proxy = item.proxy;
            const domains = item.domains || [];
            
            // Check if proxy attributes match
            const matchesProxy = 
                proxy.title.toLowerCase().includes(searchTerm.toLowerCase()) ||
                proxy.addr_listen.toLowerCase().includes(searchTerm.toLowerCase());
            
            // Check if any domain matches
            const matchesDomain = domains.some(domain => 
                domain.sni && domain.sni.toLowerCase().includes(searchTerm.toLowerCase())
            );
            
            return matchesProxy || matchesDomain;
        })
        : proxiesWithDomains;
    
    // For "load more" functionality
    let visibleCount = 6;
    $: hasMoreToLoad = filteredProxies.length > visibleCount;
    $: visibleProxies = filteredProxies.slice(0, visibleCount);
    
    function loadMore(): void {
        visibleCount += 6;
    }
    
    // Function to add a new proxy (redirects to the new proxy page)
    function addProxy(): void {
        goto('/proxy/new');
    }
    
    // Function to edit an existing proxy (redirects to the edit page)
    function editProxy(proxyId: string): void {
        goto(`/proxy/${proxyId}`);
    }
    
    // Function to delete a proxy
    async function deleteProxy(id: string): Promise<void> {
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
            {#each visibleProxies as proxyWithDomains (proxyWithDomains.proxy.id)}
                <div>
                    <ProxyCard 
                        proxy={proxyWithDomains.proxy}
                        domains={proxyWithDomains.domains || []}
                        onEdit={() => editProxy(proxyWithDomains.proxy.id)} 
                        onDelete={() => deleteProxy(proxyWithDomains.proxy.id)}
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
</div>