<!-- Mobile Menu Component -->
<script lang="ts">
    export let isOpen: boolean;
    export let username: string;
    export let onClose: () => void;
    export let onLogout: () => void;
    
    function handleKeydown(event: KeyboardEvent) {
        if (event.key === 'Escape') {
            onClose();
        }
    }
</script>

<!-- Mobile Menu (Off-Canvas) -->
<div 
    id="mobile-menu"
    class="fixed inset-0 flex z-40 md:hidden {isOpen ? 'opacity-100' : 'opacity-0 pointer-events-none'} transition-opacity duration-300 ease-in-out"
    aria-modal="true" 
    role="dialog"
    tabindex="-1"
    on:keydown={handleKeydown}
>
    <!-- Overlay -->
    <div 
        class="fixed inset-0 backdrop-blur-sm bg-black/30  bg-opacity-75 transition-opacity {isOpen ? 'opacity-100' : 'opacity-0'}" 
        on:click={onClose}
        on:keydown={handleKeydown}
        role="button"
        tabindex="0"
        aria-label="Close menu"
    ></div>
    
    <!-- Side Menu -->
    <div class="relative flex-1 flex flex-col max-w-xs w-full pt-5 pb-4 bg-white dark:bg-[#161b22] shadow-lg transform transition-all {isOpen ? 'translate-x-0 opacity-100' : '-translate-x-4 opacity-0'}">
        <div class="absolute top-0 right-0 -mr-12 pt-2">
            <button 
                type="button" 
                class="ml-1 flex items-center justify-center h-10 w-10 rounded-full focus:outline-none focus:ring-2 focus:ring-inset focus:ring-white"
                on:click={onClose}
            >
                <span class="sr-only">Close menu</span>
                <svg class="h-6 w-6 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor" aria-hidden="true">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                </svg>
            </button>
        </div>
        
        <!-- Menu header -->
        <div class="flex-shrink-0 flex items-center px-4 border-b border-gray-200 dark:border-gray-700 pb-4 mb-4">
            <img src="/logo.png" alt="Logo" class="h-8 w-auto" />
            <span class="ml-2 text-xl font-semibold">Mini Gateway</span>
        </div>
        
        <!-- User info -->
        <div class="px-4 py-3 border-b border-gray-200 dark:border-gray-700 mb-3">
            <div class="flex items-center">
                <div class="flex-shrink-0">
                    <svg class="h-10 w-10 text-gray-400" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
                    </svg>
                </div>
                <div class="ml-3">
                    <div class="text-base font-medium">{username}</div>
                    <button 
                        on:click={onLogout}
                        class="text-sm text-red-500 hover:text-red-700 mt-1"
                    >
                        Logout
                    </button>
                </div>
            </div>
        </div>
        
        <!-- Navigation Links -->
        <nav class="mt-2 flex-1 px-4 space-y-1">
            <a href="/" class="group flex items-center px-2 py-3 text-base font-medium rounded-md hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors border-l-4 border-transparent hover:border-indigo-500">
                <svg xmlns="http://www.w3.org/2000/svg" class="mr-3 h-6 w-6 text-gray-500 group-hover:text-gray-900 dark:text-gray-400 dark:group-hover:text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6" />
                </svg>
                Dashboard
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
    </div>
</div>