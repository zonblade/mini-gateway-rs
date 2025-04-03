<script lang="ts">
    import "../app.css";
    import { onMount } from "svelte";
    import { user } from "$lib/stores/userStore";
    import Layout from "$lib/components/Layout.svelte";
    import Header from "$lib/components/Header.svelte";
    import Footer from "$lib/components/Footer.svelte";

    let isLoggedIn: boolean = false;
    
    // Subscribe to user store
    const unsubscribe = user.subscribe(value => {
        isLoggedIn = !!value;
    });

    // Handle logout
    function handleLogout() {
        user.logout();
    }

    // Get current user's username
    let username: string = "";
    user.subscribe(value => {
        if (value) {
            username = value.username;
        }
    });

    onMount(() => {
        return unsubscribe;
    });
</script>

<Layout {isLoggedIn} {username} onLogout={handleLogout}>
    <svelte:fragment slot="header">
        <Header {username} onLogout={handleLogout} />
    </svelte:fragment>
    
    <slot />
    
    <svelte:fragment slot="footer">
        <Footer />
    </svelte:fragment>
</Layout>
