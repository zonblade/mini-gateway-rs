<!-- Mobile Menu Component -->
<script lang="ts">
    import { Home, BarChart2, Users, Server, Share2, X, User } from 'lucide-svelte';
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
        class="fixed inset-0 backdrop-blur-sm bg-black/30 bg-opacity-75 transition-opacity {isOpen ? 'opacity-100' : 'opacity-0'}" 
        on:click={onClose}
        on:keydown={handleKeydown}
        role="button"
        tabindex="0"
        aria-label="Close menu"
    ></div>
    
    <!-- Side Menu -->
    <div class="relative flex-1 flex flex-col max-w-xs w-full pt-5 pb-4 bg-white dark:bg-[#161b22] border-r border-gray-200 dark:border-gray-700 transform transition-all {isOpen ? 'translate-x-0 opacity-100' : '-translate-x-4 opacity-0'}">
        <div class="absolute top-0 right-0 -mr-12 pt-2">
            <button 
                type="button" 
                class="ml-1 flex items-center justify-center h-10 w-10 focus:outline-none"
                on:click={onClose}
            >
                <span class="sr-only">Close menu</span>
                <X class="h-6 w-6 text-gray-400" />
            </button>
        </div>
        
        <!-- Menu header -->
        <div class="flex-shrink-0 flex items-center px-4 border-b border-gray-200 dark:border-gray-700 pb-4 mb-4">
            <img src="/logo.png" alt="Logo" class="h-8 w-auto" />
            <span class="ml-2 text-xl font-normal">Mini Gateway</span>
        </div>
        
        <!-- User info -->
        <div class="px-4 py-3 border-b border-gray-200 dark:border-gray-700 mb-3">
            <div class="flex items-center">
                <div class="flex-shrink-0">
                    <User class="h-10 w-10 text-gray-400" />
                </div>
                <div class="ml-3">
                    <div class="text-base font-normal">{username}</div>
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
    </div>
</div>