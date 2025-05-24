<!-- Desktop Sidebar Navigation Component -->
<script lang="ts">
    import { Home, BarChart2, Users, Server, Share2, Menu } from 'lucide-svelte';
    export let username: string;
    
    // Function to handle logout - will be passed from parent
    export let onLogout: () => void;
</script>

<div class="flex h-full w-full">
    <!-- Desktop Sidebar -->
    <div class="hidden md:flex flex-col w-64 bg-white dark:bg-[#161b22] border-r border-gray-200 dark:border-gray-700 h-screen fixed left-0 top-0 z-30">
        <!-- Brand/logo -->
        <div class="flex items-center px-4 py-5 border-b border-gray-200 dark:border-gray-700">
            <img src="/logo.png" alt="Logo" class="h-8 w-auto mr-4" />
            <span class="text-xl font-normal">Mini Gateway</span>
        </div>
        
        <!-- Navigation Links -->
        <nav class="mt-5 flex-1 px-4 space-y-1">
            <a href="/" class="group flex items-center px-2 py-3 text-base font-normal hover:bg-gray-50 dark:hover:bg-gray-900 transition-colors border-l-2 border-transparent hover:border-indigo-500">
                <Home class="mr-3 h-6 w-6 text-gray-500 group-hover:text-gray-900 dark:text-gray-400 dark:group-hover:text-white" />
                Dashboard
            </a>
            <a href="/stats" class="group flex items-center px-2 py-3 text-base font-normal hover:bg-gray-50 dark:hover:bg-gray-900 transition-colors border-l-2 border-transparent hover:border-indigo-500">
                <BarChart2 class="mr-3 h-6 w-6 text-gray-500 group-hover:text-gray-900 dark:text-gray-400 dark:group-hover:text-white" />
                Stats
            </a>
            <a href="/users" class="group flex items-center px-2 py-3 text-base font-normal hover:bg-gray-50 dark:hover:bg-gray-900 transition-colors border-l-2 border-transparent hover:border-indigo-500">
                <Users class="mr-3 h-6 w-6 text-gray-500 group-hover:text-gray-900 dark:text-gray-400 dark:group-hover:text-white" />
                Users
            </a>
            <a href="/proxy" class="group flex items-center px-2 py-3 text-base font-normal hover:bg-gray-50 dark:hover:bg-gray-900 transition-colors border-l-2 border-transparent hover:border-indigo-500">
                <Server class="mr-3 h-6 w-6 text-gray-500 group-hover:text-gray-900 dark:text-gray-400 dark:group-hover:text-white" />
                Proxy
            </a>
            <a href="/gwnode" class="group flex items-center px-2 py-3 text-base font-normal hover:bg-gray-50 dark:hover:bg-gray-900 transition-colors border-l-2 border-transparent hover:border-indigo-500">
                <Share2 class="mr-3 h-6 w-6 text-gray-500 group-hover:text-gray-900 dark:text-gray-400 dark:group-hover:text-white" />
                Gateway Nodes
            </a>
        </nav>
        
        <!-- User info and logout -->
        <div class="border-t border-gray-200 dark:border-gray-700 p-4">
            <div class="flex items-center justify-between">
                <span class="text-sm">Welcome, {username}</span>
                <button 
                    on:click={onLogout}
                    class="px-3 py-2 text-sm font-normal bg-red-600 hover:bg-red-700 text-white transition-colors"
                >
                    Logout
                </button>
            </div>
        </div>
    </div>

    <!-- Mobile Header -->
    <div class="md:hidden flex h-16 justify-between items-center w-full ">
        <div class="flex items-center">
            <img src="/logo.png" alt="Logo" class="h-8 w-auto mr-4" />
            <span class="text-xl font-normal">Mini Gateway</span>
        </div>
        
        <!-- Mobile Menu Button -->
        <div class="flex items-center">
            <button 
                id="menu-toggle"
                class="inline-flex items-center justify-center p-2 text-gray-700 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white hover:bg-gray-100 dark:hover:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-inset focus:ring-indigo-500 transition-colors"
                aria-expanded={false}
                aria-label="Main menu"
                on:click
            >
                <Menu class="h-6 w-6" />
            </button>
        </div>
    </div>
</div>