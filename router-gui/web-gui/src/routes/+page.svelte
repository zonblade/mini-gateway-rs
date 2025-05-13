<script lang="ts">
    import { onMount } from "svelte";
    import { user } from "$lib/stores/userStore";
    import Login from "$lib/components/Login.svelte";
    import Dashboard from "$lib/components/Dashboard.svelte";

    // Authentication state
    let isLoggedIn = false;

    // Subscribe to the user store
    const unsubscribe = user.subscribe((value) => {
        isLoggedIn = !!value;
    });

    // Clean up subscription when component is destroyed
    onMount(() => {
        return unsubscribe;
    });

    // Handle successful login
    function handleLoginSuccess() {
        // Update the state (the user store subscription will handle this)
    }
</script>

{#if isLoggedIn}
    <Dashboard />
{:else}
    <Login onLoginSuccess={handleLoginSuccess} />
{/if}

<style lang="postcss">
    @import "tailwindcss";

    :global(html) {
        @apply transition-colors duration-300;
    }

    :global(html.dark) {
        color-scheme: dark;
    }

    /* Prevent flashing of unstyled content by hiding the body until theme is applied */
    :global(body) {
        visibility: visible;
    }

    :global(body.loading) {
        visibility: hidden;
    }
</style>
