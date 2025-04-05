<script lang="ts">
    export let headers: string[] = []; // Column headers
    export let data: any[] = []; // Table data
    export let keyField: string = 'id'; // Field to use as key for each row
    export let onRowClick: ((item: any) => void) | null = null; // Optional row click handler
    export let isLoading: boolean = false; // Loading state
    export let emptyMessage: string = "No data available"; // Message to display when there's no data
    export let containerClass: string = ""; // Additional class for the container
</script>

<div class={`overflow-x-auto ${containerClass}`}>
    <table class="w-full border-collapse">
        <thead>
            <tr class="bg-gray-100 dark:bg-gray-800">
                {#each headers as header}
                    <th class="py-3 px-4 text-left text-sm font-medium text-gray-700 dark:text-gray-300 border-b dark:border-gray-700">
                        {header}
                    </th>
                {/each}
            </tr>
        </thead>
        <tbody>
            {#if isLoading}
                <tr>
                    <td colspan={headers.length} class="py-4 px-4 text-center text-gray-500 dark:text-gray-400">
                        <div class="flex justify-center items-center">
                            <svg class="animate-spin h-5 w-5 mr-3 text-blue-500" viewBox="0 0 24 24">
                                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" fill="none"></circle>
                                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                            </svg>
                            Loading...
                        </div>
                    </td>
                </tr>
            {:else if data.length === 0}
                <tr>
                    <td colspan={headers.length} class="py-4 px-4 text-center text-gray-500 dark:text-gray-400">
                        {emptyMessage}
                    </td>
                </tr>
            {:else}
                {#each data as item (item[keyField])}
                    <tr 
                        class="border-b dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800/60 cursor-pointer"
                        on:click={() => onRowClick && onRowClick(item)}
                    >
                        <slot {item}></slot>
                    </tr>
                {/each}
            {/if}
        </tbody>
    </table>
</div>