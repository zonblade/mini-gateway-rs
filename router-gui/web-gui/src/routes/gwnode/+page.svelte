<script lang="ts">
    import GwNodeManager from "$lib/components/gwnode/GwNodeManager.svelte";
    import { onMount } from "svelte";
    
    let searchTerm = "";
    
    onMount(() => {
        document.title = "Gateway Nodes | Mini Gateway";
    });
</script>

<svelte:head>
    <title>Gateway Nodes | Mini Gateway</title>
</svelte:head>

<div class="container mx-auto px-4 py-8 flex flex-col items-center">
    <GwNodeManager {searchTerm} />
</div>