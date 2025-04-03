<script lang="ts">
    export let searchTerm: string = "";
</script>

<div class="mb-6">
    <div class="relative">
        <input 
            type="text" 
            bind:value={searchTerm}
            placeholder="Search..."
            class="w-full p-3 pl-10 rounded-md border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
        />
        <div class="absolute left-3 top-3 text-gray-400">
            <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <circle cx="11" cy="11" r="8"></circle>
                <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
            </svg>
        </div>
    </div>
</div>