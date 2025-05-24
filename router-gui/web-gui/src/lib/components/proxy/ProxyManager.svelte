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
    import Swal from "sweetalert2";
    import DeleteConfirmationModal from "$lib/components/common/DeleteConfirmationModal.svelte";

    // For search functionality
    export let searchTerm = "";

    // Store subscriptions
    let proxiesWithDomains: ProxyWithDomains[] = [];
    let isLoading = true;
    let isLoadError = false;

    // Add these variables
    let showDeleteModal = false;
    let proxyToDelete: { id: string; listen: string } | null = null;
    let isProcessing = false;
    let errorMessage: string | null = null;

    // Subscribe to the store
    const unsubProxy = proxyStore.subscribe((state) => {
        console.debug("Proxy store state:", state);
        proxiesWithDomains = state.proxies;
        isLoading = state.loading;
        isLoadError = state.loadError;
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
        ? proxiesWithDomains.filter((item) => {
              const proxy = item.proxy;
              const domains = item.domains || [];

              // Check if proxy attributes match
              const matchesProxy =
                  proxy.title
                      .toLowerCase()
                      .includes(searchTerm.toLowerCase()) ||
                  proxy.addr_listen
                      .toLowerCase()
                      .includes(searchTerm.toLowerCase());

              // Check if any domain matches
              const matchesDomain = domains.some(
                  (domain) =>
                      domain.sni &&
                      domain.sni
                          .toLowerCase()
                          .includes(searchTerm.toLowerCase()),
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
        goto("/proxy/new");
    }

    // Function to edit an existing proxy (redirects to the edit page)
    function editProxy(proxyId: string): void {
        goto(`/proxy/${proxyId}`);
    }

    // Function to retry loading after an error
    function retryLoading(): void {
        proxyStore.fetchProxies();
    }

    // Function to delete a proxy
    async function deleteProxy(id: string, listen: string) {
        proxyToDelete = { id, listen };
        errorMessage = null; // Reset error message
        showDeleteModal = true;
    }

    function handleDeleteCancel() {
        console.log("handleDeleteCancel");
        showDeleteModal = false;
        proxyToDelete = null;
        errorMessage = null; // Reset error message
    }

    async function handleDeleteConfirm() {
        if (!proxyToDelete) return;

        try {
            isProcessing = true;
            let [ok, message] = await proxyStore.deleteProxy(proxyToDelete.id);
            await proxyStore.fetchProxies();
            if (message.length > 0) {
                throw new Error(message);
            }
            if (ok) {
                showDeleteModal = false;
                proxyToDelete = null;
                errorMessage = null; // Reset error message
            }
        } catch (error) {
            console.error("Error deleting proxy:", error);
            errorMessage = error instanceof Error ? error.message : String(error);
        } finally {
            isProcessing = false;
        }
    }

    // Function to sync proxies with the server
    async function syncProxies(): Promise<void> {
        try {
            // Show loading state
            Swal.fire({
                title: "Syncing...",
                text: "Please wait while we sync your proxies",
                allowOutsideClick: false,
                didOpen: () => {
                    Swal.showLoading();
                },
            });

            const result = await proxyStore.syncProxies();
            await gwnodeActions.syncGatewayNodes();

            await Swal.fire({
                title: "Success!",
                text: result.message,
                icon: "success",
                timer: 2000,
                showConfirmButton: false,
            });
        } catch (error) {
            console.error("Error syncing proxies:", error);
            await Swal.fire({
                title: "Sync Failed",
                text:
                    "Failed to sync proxies: " +
                    (error instanceof Error ? error.message : String(error)),
                icon: "error",
            });
        }
    }
</script>

<div class="w-full max-w-[900px]">
    <div class="flex justify-between items-center mb-6">
        <h1 class="text-2xl font-bold text-gray-900 dark:text-white">
            Proxy Management
        </h1>
        <div class="flex space-x-2">
            <Button variant="secondary" onClick={syncProxies}>
                Sync Nodes
            </Button>
            <Button variant="primary" onClick={addProxy}>Add Proxy</Button>
        </div>
    </div>

    <!-- Search input -->
    {#if !isLoading && !isLoadError}
        <div class="mb-6">
            <SearchInput
                bind:value={searchTerm}
                placeholder="Search by title, address, or domain..."
            />
        </div>
    {/if}

    <!-- Card grid layout -->
    {#if isLoading}
        <LoadingSpinner />
    {:else if isLoadError}
        <!-- Error state -->
        <div
            class="flex flex-col items-center justify-center py-12 text-center"
        >
            <div class="text-red-500 mb-4">
                <svg
                    xmlns="http://www.w3.org/2000/svg"
                    class="h-12 w-12 mx-auto mb-2"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                >
                    <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width="2"
                        d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
                    />
                </svg>
                <h3 class="text-lg font-medium">Failed to load proxy nodes</h3>
                <p class="text-sm mt-1">{isLoadError}</p>
            </div>
            <div class="flex space-x-4">
                <Button variant="secondary" onClick={retryLoading}>
                    Retry
                </Button>
                <Button variant="primary" onClick={addProxy}>
                    Create Proxy
                </Button>
            </div>
        </div>
    {:else if visibleProxies.length === 0}
        <div class="text-center py-8">
            <EmptyState
                message={searchTerm
                    ? "No proxies match your search criteria"
                    : "No proxies found"}
                icon="search"
            />
            {#if !searchTerm}
                <div class="mt-4">
                    <Button variant="primary" onClick={addProxy}>
                        Create your first proxy
                    </Button>
                </div>
            {:else}
                <div class="mt-4">
                    <Button
                        variant="secondary"
                        onClick={() => (searchTerm = "")}
                    >
                        Clear Search
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
                        onDelete={() =>
                            deleteProxy(
                                proxyWithDomains.proxy.id,
                                proxyWithDomains.proxy.addr_listen,
                            )}
                    />
                </div>
            {/each}
        </div>

        <!-- Load more button -->
        {#if hasMoreToLoad}
            <div class="mt-6 text-center">
                <Button variant="secondary" onClick={loadMore}>
                    Load more...
                </Button>
            </div>
        {/if}
    {/if}

    <!-- Delete confirmation modal -->
    <DeleteConfirmationModal
        showModal={showDeleteModal}
        type="proxy"
        {errorMessage}
        addressToVerify={proxyToDelete?.listen || ""}
        {isProcessing}
        on:confirm={handleDeleteConfirm}
        on:cancel={handleDeleteCancel}
    />
</div>
