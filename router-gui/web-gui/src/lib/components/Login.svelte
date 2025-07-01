<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import { goto } from "$app/navigation";
    import LoginForm from "$lib/components/LoginForm.svelte";
    import ConnectionsList from "$lib/components/ConnectionsList.svelte";
    import { initTheme, applyTheme, setupThemeListener } from "$lib/utils/theme";
    import { user } from "$lib/stores/userStore";
    import { connections } from "$lib/stores/connectionsStore";

    // Form state
    let username: string = "";
    let password: string = "";
    let rememberMe: boolean = false;
    let isSubmitting: boolean = false;
    let errorMessage: string = "";
    let selectedConnectionId: string = "";

    // Theme state (will be controlled by OS preference)
    let isDarkMode: boolean = false;
    let removeThemeListener: (() => void) | undefined;

    export let onLoginSuccess: () => void;

    onMount(() => {
        // Initialize theme based on OS preference
        isDarkMode = initTheme();
        
        // Setup listener for OS theme preference changes
        removeThemeListener = setupThemeListener((darkMode: boolean) => {
            isDarkMode = darkMode;
            applyTheme(darkMode);
        });
        
        // Auto-fill from localStorage if available
        const rememberedUser = localStorage.getItem('rememberedUsername');
        if (rememberedUser) {
            username = rememberedUser;
            rememberMe = true;
        }
    });

    onDestroy(() => {
        // Clean up theme listener when component is destroyed
        if (removeThemeListener) {
            removeThemeListener();
        }
    });

    async function handleSubmit(): Promise<void> {
        if (!username || !password) {
            errorMessage = "Please enter both username and password";
            return;
        }

        if (!selectedConnectionId) {
            errorMessage = "Please select a connection";
            return;
        }

        isSubmitting = true;
        errorMessage = "";

        try {
            // Get the selected connection details
            const connection = connections.getConnectionById(selectedConnectionId);
            if (!connection) {
                throw new Error("Selected connection not found");
            }

            // Set API base URL based on the selected connection
            const API_BASE_URL = `${connection.protocol}://${connection.host}${connection.port ? `:${connection.port}` : ''}${connection.subpath || '/api/v1'}`;
            
            // Mock API call - replace with actual API integration
            await new Promise(resolve => setTimeout(resolve, 500));
            
            // Use our user store login function with the connection details
            await user.login(username, password, API_BASE_URL);
            
            // Handle rememberMe preference
            if (rememberMe) {
                localStorage.setItem('rememberedUsername', username);
            } else {
                localStorage.removeItem('rememberedUsername');
            }
            
            // Save the last used connection ID
            localStorage.setItem('lastConnectionId', selectedConnectionId);
            
            // Call success callback
            onLoginSuccess();
        } catch (error) {
            errorMessage =
                error instanceof Error
                    ? error.message
                    : "Login failed. Please try again.";
            isSubmitting = false;
        }
    }
</script>

<div
    class="min-h-screen flex items-center justify-center transition-colors duration-300 bg-gray-50 dark:bg-[#121212] text-gray-900 dark:text-gray-100 p-4"
>
    <div
        class="max-w-5xl w-full bg-white dark:bg-[#1a1a1a] border border-gray-200 dark:border-gray-800"
    >
        <div class="p-6 sm:p-8">
            <div class="flex justify-center mb-8">
                <img src="/logo.png" alt="Logo" class="h-[80px] w-auto opacity-90" />
            </div>

            <h2 class="text-2xl font-normal text-center mb-6">
                Mini Gateway
            </h2>

            <div class="flex flex-col md:flex-row gap-8">
                <!-- Connection List -->
                <div class="w-full md:w-1/2 border-b md:border-b-0 md:border-r border-gray-200 dark:border-gray-800 pb-6 md:pb-0 md:pr-6">
                    <ConnectionsList bind:selectedConnectionId={selectedConnectionId} />
                </div>
                
                <!-- Login Form -->
                <div class="w-full md:w-1/2 md:pl-6">
                    <LoginForm 
                        bind:username
                        bind:password
                        {isSubmitting}
                        {errorMessage}
                        onSubmit={handleSubmit}
                    />
                    
                    <div class="mt-4 flex items-center">
                        <input 
                            type="checkbox" 
                            id="rememberMe" 
                            bind:checked={rememberMe}
                            class="mr-2 border-gray-300 dark:border-gray-700 text-gray-500 focus:ring-0" 
                        />
                        <label for="rememberMe" class="text-sm">Remember username</label>
                    </div>
                    
                    <p class="mt-8 text-center text-sm text-gray-500 dark:text-gray-400">
                        Ask admin to create an account for you.
                    </p>
                </div>
            </div>
        </div>
    </div>
</div>

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