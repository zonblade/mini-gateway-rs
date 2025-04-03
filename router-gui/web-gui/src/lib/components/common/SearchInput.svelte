<script lang="ts">
    export let value = "";
    export let placeholder = "Search...";
    export let onSearch: (searchTerm: string) => void = () => {};
    
    function handleInput(event: Event) {
        const inputElement = event.target as HTMLInputElement;
        value = inputElement.value;
        onSearch(value);
    }
    
    function clearSearch() {
        value = "";
        onSearch("");
    }
</script>

<div class="relative w-full">
    <input 
        type="text" 
        {placeholder}
        value={value}
        on:input={handleInput}
        class="w-full p-2 pr-10 rounded-md border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
    />
    
    {#if value}
        <button 
            on:click={clearSearch}
            class="absolute inset-y-0 right-0 flex items-center pr-3 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
            aria-label="Clear search"
        >
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <circle cx="12" cy="12" r="10"></circle>
                <line x1="15" y1="9" x2="9" y2="15"></line>
                <line x1="9" y1="9" x2="15" y2="15"></line>
            </svg>
        </button>
    {:else}
        <div class="absolute inset-y-0 right-0 flex items-center pr-3 text-gray-400 dark:text-gray-500">
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <circle cx="11" cy="11" r="8"></circle>
                <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
            </svg>
        </div>
    {/if}
</div>