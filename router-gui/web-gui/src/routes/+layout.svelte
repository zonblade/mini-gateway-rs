<script lang="ts">
    import "../app.css";
    import { onMount } from "svelte";
    import { user } from "$lib/stores/userStore";

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

{#if isLoggedIn}
    <div class="flex flex-col w-full h-screen bg-gray-50 dark:bg-[#0d1117] text-gray-900 dark:text-gray-100">
        <!-- Header/Navigation -->
        <header class="bg-white dark:bg-[#161b22] shadow-sm">
            <div class="mx-auto px-4 sm:px-6 lg:px-8">
                <div class="flex h-16 justify-between items-center">
                    <div class="flex items-center">
                        <img src="/logo.png" alt="Logo" class="h-8 w-auto mr-4" />
                        <span class="text-xl font-semibold">Mini Gateway</span>
                    </div>
                    
                    <nav class="hidden md:flex space-x-8">
                        <a href="/" class="px-3 py-2 rounded-md text-sm font-medium hover:bg-gray-100 dark:hover:bg-gray-800">Dashboard</a>
                        <a href="/users" class="px-3 py-2 rounded-md text-sm font-medium hover:bg-gray-100 dark:hover:bg-gray-800">Users</a>
                        <a href="/settings" class="px-3 py-2 rounded-md text-sm font-medium hover:bg-gray-100 dark:hover:bg-gray-800">Settings</a>
                    </nav>
                    
                    <div class="flex items-center">
                        <span class="mr-4 text-sm">Welcome, {username}</span>
                        <button 
                            on:click={handleLogout}
                            class="px-3 py-2 rounded-md text-sm font-medium bg-red-600 hover:bg-red-700 text-white"
                        >
                            Logout
                        </button>
                    </div>
                </div>
            </div>
        </header>
        
        <!-- Main content -->
        <main class="flex-1">
            <slot />
        </main>
        
        <!-- Footer -->
        <footer class="bg-white dark:bg-[#161b22] shadow-sm py-4">
            <div class="mx-auto px-4 sm:px-6 lg:px-8 text-center text-sm text-gray-500 dark:text-gray-400">
                &copy; 2023 Mini Gateway. All rights reserved.
            </div>
        </footer>
    </div>
{:else}
    <div class="flex flex-col w-full h-screen">
        <slot />
    </div>
{/if}
