<script lang="ts">
    import GwNodeManager from "$lib/components/gwnode/GwNodeManager.svelte";
    import { onMount } from "svelte";
    import { goto } from "$app/navigation";
    import { user } from "$lib/stores/userStore";

    let searchTerm = "";

    // Authentication and loading states
    let isLoggedIn = false;
    let isLoading = true;

    const unsubAuthCheck = user.subscribe((value) => {
        isLoggedIn = !!value;
        isLoading = false;
    });

    onMount(() => {
        document.title = "Gateway Nodes | Mini Gateway";
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
    <title>Gateway Nodes | Mini Gateway</title>
</svelte:head>

{#if isLoading}
    <div class="flex items-center justify-center h-screen">
        <div
            class="animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-blue-600"
        ></div>
    </div>
{:else if isLoggedIn}
    <div class="container mx-auto px-4 py-8 flex flex-col items-center">
        <GwNodeManager {searchTerm} />
    </div>
{/if}
