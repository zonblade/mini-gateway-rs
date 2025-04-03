<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import { goto } from "$app/navigation";
    import LoginForm from "$lib/components/LoginForm.svelte";
    import { initTheme, applyTheme, setupThemeListener } from "$lib/utils/theme";
    import { user } from "$lib/stores/userStore";

    // Form state
    let username: string = "";
    let password: string = "";
    let rememberMe: boolean = false;
    let isSubmitting: boolean = false;
    let errorMessage: string = "";

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

        isSubmitting = true;
        errorMessage = "";

        try {
            // Use our user store login function
            await user.login(username, password);
            
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
    class="min-h-screen flex items-center justify-center transition-colors duration-300 bg-gray-50 dark:bg-[#0d1117] text-gray-900 dark:text-gray-100 p-4"
>
    <div
        class="max-w-md w-full bg-white dark:bg-[#161b22] rounded-lg shadow-md overflow-hidden"
    >
        <div class="p-6 sm:p-8">
            <div class="flex justify-center mb-8">
                <img src="/logo.png" alt="Logo" class="h-[120px] w-auto" />
            </div>

            <h2 class="text-2xl font-bold text-center mb-6">
                Mini Gateway
            </h2>

            <LoginForm 
                bind:username
                bind:password
                {isSubmitting}
                {errorMessage}
                onSubmit={handleSubmit}
            />
            <p
                class="mt-8 text-center text-sm text-gray-500 dark:text-gray-400"
            >
                Ask admin to create an account for you.
            </p>
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