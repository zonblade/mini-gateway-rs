<!-- Desktop Sidebar Navigation Component -->
<script lang="ts">
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
            <span class="text-xl font-semibold">Mini Gateway</span>
        </div>
        
        <!-- Navigation Links -->
        <nav class="mt-5 flex-1 px-4 space-y-1">
            <a href="/" class="group flex items-center px-2 py-3 text-base font-medium rounded-md hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors border-l-4 border-transparent hover:border-indigo-500">
                <svg xmlns="http://www.w3.org/2000/svg" class="mr-3 h-6 w-6 text-gray-500 group-hover:text-gray-900 dark:text-gray-400 dark:group-hover:text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6" />
                </svg>
                Dashboard
            </a>
            <a href="/stats" class="group flex items-center px-2 py-3 text-base font-medium rounded-md hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors border-l-4 border-transparent hover:border-indigo-500">
                <svg xmlns="http://www.w3.org/2000/svg" class="mr-3 h-6 w-6 text-gray-500 group-hover:text-gray-900 dark:text-gray-400 dark:group-hover:text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
                </svg>
                Stats
            </a>
            <a href="/users" class="group flex items-center px-2 py-3 text-base font-medium rounded-md hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors border-l-4 border-transparent hover:border-indigo-500">
                <svg xmlns="http://www.w3.org/2000/svg" class="mr-3 h-6 w-6 text-gray-500 group-hover:text-gray-900 dark:text-gray-400 dark:group-hover:text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z" />
                </svg>
                Users
            </a>
            <a href="/proxy" class="group flex items-center px-2 py-3 text-base font-medium rounded-md hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors border-l-4 border-transparent hover:border-indigo-500">
                <svg xmlns="http://www.w3.org/2000/svg" class="mr-3 h-6 w-6 text-gray-500 group-hover:text-gray-900 dark:text-gray-400 dark:group-hover:text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                </svg>
                Proxy
            </a>
            <a href="/gwnode" class="group flex items-center px-2 py-3 text-base font-medium rounded-md hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors border-l-4 border-transparent hover:border-indigo-500">
                <svg xmlns="http://www.w3.org/2000/svg" class="mr-3 h-6 w-6 text-gray-500 group-hover:text-gray-900 dark:text-gray-400 dark:group-hover:text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
                </svg>
                Gateway Nodes
            </a>
        </nav>
        
        <!-- User info and logout -->
        <div class="border-t border-gray-200 dark:border-gray-700 p-4">
            <div class="flex items-center justify-between">
                <span class="text-sm">Welcome, {username}</span>
                <button 
                    on:click={onLogout}
                    class="px-3 py-2 rounded-md text-sm font-medium bg-red-600 hover:bg-red-700 text-white transition-colors"
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
            <span class="text-xl font-semibold">Mini Gateway</span>
        </div>
        
        <!-- Mobile Menu Button -->
        <div class="flex items-center">
            <button 
                id="menu-toggle"
                class="inline-flex items-center justify-center p-2 rounded-md text-gray-700 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white hover:bg-gray-100 dark:hover:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-inset focus:ring-indigo-500 transition-colors"
                aria-expanded={false}
                aria-label="Main menu"
                on:click
            >
                <svg class="h-6 w-6" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor" aria-hidden="true">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
                </svg>
            </button>
        </div>
    </div>
</div>