<script lang="ts">
    import { onMount } from "svelte";
    import { page } from "$app/stores";
    import { goto } from "$app/navigation";
    import { user } from "$lib/stores/userStore";
    import { gwNodes } from "$lib/stores/gwnodeStore";
    import type { GwNode } from "$lib/types/gwnode";
    import LoadingSpinner from "$lib/components/common/LoadingSpinner.svelte";
    import BackButton from "$lib/components/common/BackButton.svelte";
    import GwNodeDetail from "$lib/components/gwnode/GwNodeDetail.svelte";
    import GwNodeNotFound from "$lib/components/gwnode/GwNodeNotFound.svelte";

    // Get the gwnode ID from the route parameter
    let gwnodeId = $page.params.id;
    
    // Find the matching gwnode
    let gwnode: GwNode | undefined;
    
    // Subscribe to the gwNodes store to get the current node
    const unsubGwNodes = gwNodes.subscribe(nodes => {
        gwnode = nodes.find(n => n.id === gwnodeId);
    });
    
    // Set page title
    onMount(() => {
        document.title = gwnode
            ? `${gwnode.title} | Gateway Node`
            : "Gateway Node Not Found";
            
        return () => {
            unsubGwNodes(); // Clean up subscription
        };
    });

    // Authentication and loading states
    let isLoggedIn = false;
    let isLoading = true;

    const unsubAuthCheck = user.subscribe((value) => {
        isLoggedIn = !!value;
        isLoading = false;
    });

    onMount(() => {
        // Redirect happens after auth check is complete
        if (!isLoading && !isLoggedIn) {
            goto("/");
        }

        return () => {
            unsubAuthCheck(); // Clean up subscription
        };
    });
</script>

<svelte:head>
    <title>{gwnode ? `${gwnode.title} | Gateway Node` : "Gateway Node Not Found"}</title>
</svelte:head>

{#if isLoading}
    <LoadingSpinner />
{:else if isLoggedIn}
    <div class="py-8 px-4 flex flex-col items-center">
        <div class="w-full max-w-[900px]">
            <BackButton text="Back to Gateway Nodes" />

            {#if gwnode}
                <GwNodeDetail {gwnode} />

                <!-- under this should be table of gateway -->
            {:else}
                <GwNodeNotFound />
            {/if}
        </div>
    </div>
{/if}
