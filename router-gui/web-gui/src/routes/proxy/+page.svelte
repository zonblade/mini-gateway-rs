<script lang="ts">
    import { onMount } from "svelte";
    import { goto } from "$app/navigation";
    import { user } from "$lib/stores/userStore";
    import SearchBar from "$lib/components/users/SearchBar.svelte";
    import Pagination from "$lib/components/users/Pagination.svelte";
    import ProxyManager from "$lib/components/proxy/ProxyManager.svelte";
    import LoadingSpinner from "$lib/components/common/LoadingSpinner.svelte";
    
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
    <LoadingSpinner />
{:else if isLoggedIn}
    <div class="px-4 flex flex-col items-center">
        <div class="rounded-lg py-8 w-full max-w-[900px]">
            <!-- Proxy Manager component -->
            <ProxyManager 
                {searchTerm}
            />
            
            <!-- Pagination is now handled inside ProxyManager -->
        </div>
    </div>
{/if}