<script lang="ts">
    export let currentPage: number;
    export let totalPages: number;
    export let totalItems: number;
    export let itemsPerPage: number;
    export let onPageChange: (page: number) => void;
    
    function goToPage(page: number) {
        if (page >= 1 && page <= totalPages) {
            onPageChange(page);
        }
    }
    
    function previousPage() {
        if (currentPage > 1) {
            onPageChange(currentPage - 1);
        }
    }
    
    function nextPage() {
        if (currentPage < totalPages) {
            onPageChange(currentPage + 1);
        }
    }
</script>

{#if totalPages > 1}
    <div class="flex items-center justify-between mt-6">
        <div class="text-sm text-gray-500 dark:text-gray-400">
            Showing {(currentPage - 1) * itemsPerPage + 1} to {Math.min(currentPage * itemsPerPage, totalItems)} of {totalItems} users
        </div>
        <div class="flex space-x-2">
            <button 
                on:click={previousPage}
                class="px-3 py-1 rounded-md border border-gray-300 dark:border-gray-600 
                       {currentPage === 1 ? 'text-gray-400 dark:text-gray-600 cursor-not-allowed' : 'text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800'}"
                disabled={currentPage === 1}
            >
                Previous
            </button>
            
            <!-- Page numbers -->
            {#each Array(totalPages) as _, i}
                {#if totalPages <= 7 || 
                    i === 0 || 
                    i === totalPages - 1 || 
                    (i >= currentPage - 2 && i <= currentPage + 2)}
                    <button 
                        on:click={() => goToPage(i + 1)}
                        class="px-3 py-1 rounded-md 
                              {currentPage === i + 1 ? 
                                'bg-blue-600 text-white' : 
                                'border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800'}"
                    >
                        {i + 1}
                    </button>
                {:else if (i === 1 && currentPage > 4) || (i === totalPages - 2 && currentPage < totalPages - 3)}
                    <span class="px-3 py-1 text-gray-500 dark:text-gray-400">...</span>
                {/if}
            {/each}
            
            <button 
                on:click={nextPage}
                class="px-3 py-1 rounded-md border border-gray-300 dark:border-gray-600 
                       {currentPage === totalPages ? 'text-gray-400 dark:text-gray-600 cursor-not-allowed' : 'text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800'}"
                disabled={currentPage === totalPages}
            >
                Next
            </button>
        </div>
    </div>
{/if}