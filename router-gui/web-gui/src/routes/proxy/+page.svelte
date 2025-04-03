<script lang="ts">
    import { onMount } from "svelte";
    import { goto } from "$app/navigation";
    import { user } from "$lib/stores/userStore";
    import SearchBar from "$lib/components/users/SearchBar.svelte";
    import Pagination from "$lib/components/users/Pagination.svelte";
    import ProxyManager from "$lib/components/proxy/ProxyManager.svelte";
    
    // Authentication and loading states
    let isLoggedIn = false;
    let isLoading = true;
    
    const unsubAuthCheck = user.subscribe(value => {
        isLoggedIn = !!value;
        isLoading = false;
    });
    
    onMount(() => {
        // Redirect happens after auth check is complete
        if (!isLoading && !isLoggedIn) {
            goto('/');
        }
        
        return () => {
            unsubAuthCheck(); // Clean up subscription
        };
    });
    
    // Search and pagination state
    let searchTerm = "";
    let currentPage = 1;
    
    // Handle page change
    function handlePageChange(page: number): void {
        currentPage = page;
    }
    
    // Handle authentication effect
    $: if (!isLoading && !isLoggedIn) {
        goto('/');
    }
</script>

{#if isLoading}
    <div class="flex items-center justify-center h-screen">
        <div class="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-blue-600"></div>
    </div>
{:else if isLoggedIn}
    <div class="p-6 max-w-6xl mx-auto">
        <div class="bg-white dark:bg-[#161b22] shadow-sm rounded-lg p-6 w-full max-w-[1200px] flex flex-col">
            <!-- Search component -->
            <SearchBar bind:searchTerm={searchTerm} />
            
            <!-- Proxy Manager component -->
            <ProxyManager 
                {searchTerm}
                {currentPage}
                {handlePageChange}
            />
            
            <!-- Pagination is now handled inside ProxyManager -->
        </div>
    </div>
{/if}