<script lang="ts">
    import { user } from "$lib/stores/userStore";
    import { onDestroy } from "svelte";
    import { BarChart2, Users, Server, Share2 } from 'lucide-svelte';

    let username = "";

    // Subscribe to the user store to get the current username
    const unsubscribe = user.subscribe((value) => {
        if (value) {
            username = value.username;
        }
    });

    const quickLinks = [
        { name: 'Stats', icon: BarChart2, href: '/stats', color: 'text-blue-500' },
        { name: 'Users', icon: Users, href: '/users', color: 'text-green-500' },
        { name: 'Proxy', icon: Server, href: '/proxy', color: 'text-purple-500' },
        { name: 'Gateway Nodes', icon: Share2, href: '/gwnode', color: 'text-orange-500' }
    ];

    onDestroy(unsubscribe);
</script>

<div class="max-w-[900px] mx-auto">
    <div class="bg-white dark:bg-[#161b22] p-6">
        <h1 class="text-2xl font-medium mb-6">Welcome, {username}</h1>

        <div class="space-y-6">
            <!-- Quick Access Section -->
            <div>
                <h2 class="text-sm font-medium text-gray-600 dark:text-gray-300 mb-4">Quick Access</h2>
                <div class="grid grid-cols-2 gap-4">
                    {#each quickLinks as link}
                        <a 
                            href={link.href}
                            class="p-4 bg-gray-50 dark:bg-gray-800/30 border border-transparent hover:border-gray-200 dark:hover:border-gray-700 transition-colors group"
                        >
                            <div class="flex items-center space-x-3">
                                <svelte:component this={link.icon} class="h-5 w-5 {link.color}" />
                                <span class="text-sm font-medium text-gray-700 dark:text-gray-200">{link.name}</span>
                            </div>
                        </a>
                    {/each}
                </div>
            </div>

            <!-- Status Section -->
            <div>
                <h2 class="text-sm font-medium text-gray-600 dark:text-gray-300 mb-4">Status</h2>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div class="p-4 bg-gray-50 dark:bg-gray-800/30">
                        <h3 class="text-sm font-medium text-gray-600 dark:text-gray-300 mb-1">Token</h3>
                        <p class="text-sm text-gray-500 dark:text-gray-400">Secured in browser</p>
                    </div>

                    <div class="p-4 bg-gray-50 dark:bg-gray-800/30">
                        <h3 class="text-sm font-medium text-gray-600 dark:text-gray-300 mb-1">Session</h3>
                        <p class="text-sm text-gray-500 dark:text-gray-400">Active</p>
                    </div>
                </div>
            </div>
        </div>
    </div>
</div>
