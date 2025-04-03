<!-- Desktop Navigation Bar Component -->
<script lang="ts">
    export let username: string;
    
    // Function to handle logout - will be passed from parent
    export let onLogout: () => void;
</script>

<div class="flex h-16 justify-between items-center">
    <div class="hidden md:flex items-center">
        <img src="/logo.png" alt="Logo" class="h-8 w-auto mr-4" />
        <span class="text-xl font-semibold">Mini Gateway</span>
    </div>
    
    <!-- Desktop Menu -->
    <nav class="hidden md:flex space-x-6">
        <a href="/" class="px-3 py-2 rounded-md text-sm font-medium hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors">Dashboard</a>
        <a href="/users" class="px-3 py-2 rounded-md text-sm font-medium hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors">Users</a>
        <a href="/settings" class="px-3 py-2 rounded-md text-sm font-medium hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors">Settings</a>
        <a href="/proxy" class="px-3 py-2 rounded-md text-sm font-medium hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors">Proxy</a>
        <a href="/gwnode" class="px-3 py-2 rounded-md text-sm font-medium hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors">Gwnode</a>
        <a href="/gateway" class="px-3 py-2 rounded-md text-sm font-medium hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors">Gateway</a>
    </nav>

    <!-- Mobile Menu Button -->
    <button 
        id="menu-toggle"
        class="md:hidden inline-flex items-center justify-center p-2 rounded-md text-gray-700 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white hover:bg-gray-100 dark:hover:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-inset focus:ring-indigo-500 transition-colors"
        aria-expanded={false}
        aria-label="Main menu"
        on:click
    >
        <svg class="h-6 w-6" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke="currentColor" aria-hidden="true">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
        </svg>
    </button>
    
    <div class="flex items-center">
        <span class="mr-4 text-sm hidden md:inline-block">Welcome, {username}</span>
        <button 
            on:click={onLogout}
            class="px-3 py-2 rounded-md text-sm font-medium bg-red-600 hover:bg-red-700 text-white transition-colors"
        >
            Logout
        </button>
    </div>
</div>