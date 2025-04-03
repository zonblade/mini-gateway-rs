<script lang="ts">
    import { user } from "$lib/stores/userStore";

    let username = "";

    // Subscribe to the user store to get the current username
    const unsubscribe = user.subscribe(value => {
        if (value) {
            username = value.username;
        }
    });

    // Clean up subscription when component is destroyed
    import { onDestroy } from "svelte";
    onDestroy(unsubscribe);
</script>

<div class="p-6 max-w-4xl mx-auto">
    <div class="bg-white dark:bg-[#161b22] shadow-sm rounded-lg p-6">
        <h1 class="text-3xl font-bold mb-6">Hello World!</h1>
        
        <div class="mb-6 p-4 bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-md">
            <p class="text-blue-800 dark:text-blue-300">
                Welcome to Mini Gateway, <span class="font-medium">{username}</span>! You have successfully logged in.
            </p>
        </div>
        
        <div class="space-y-4">
            <h2 class="text-xl font-semibold">Your Gateway Details</h2>
            
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div class="p-4 bg-gray-50 dark:bg-gray-800/50 rounded-md">
                    <h3 class="font-medium mb-2">Token</h3>
                    <p class="text-sm text-gray-500 dark:text-gray-400 break-all">
                        Secured token is stored in the browser
                    </p>
                </div>
                
                <div class="p-4 bg-gray-50 dark:bg-gray-800/50 rounded-md">
                    <h3 class="font-medium mb-2">Session</h3>
                    <p class="text-sm text-gray-500 dark:text-gray-400">
                        Active until you log out
                    </p>
                </div>
            </div>
        </div>
    </div>
</div>