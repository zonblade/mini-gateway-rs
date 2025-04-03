<script lang="ts">
    import Button from "./Button.svelte";
    
    export let title: string;
    export let hasAction = false;
    export let actionLabel = "Add";
    export let onAction: () => void = () => {};
</script>

<div class="flex justify-between items-center mb-6">
    <h1 class="text-2xl font-bold text-gray-900 dark:text-white">{title}</h1>
    
    {#if hasAction}
        <Button 
            variant="primary" 
            onClick={onAction}
        >
            {actionLabel}
        </Button>
    {/if}
</div>