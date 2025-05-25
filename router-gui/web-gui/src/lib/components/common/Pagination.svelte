<script lang="ts">
    export let currentPage: number;
    export let totalPages: number;
    export let onPageChange: (page: number) => void;
    
    // Generate an array of page numbers to display
    $: pageNumbers = generatePageNumbers(currentPage, totalPages);
    
    function generatePageNumbers(current: number, total: number): number[] {
        if (total <= 7) {
            // If we have 7 or fewer pages, show all of them
            return Array.from({ length: total }, (_, i) => i + 1);
        }
        
        // Otherwise, show first, last, current, and some pages around the current
        const result: number[] = [];
        
        // Always include page 1
        result.push(1);
        
        // Handle ellipsis before current page
        if (current > 3) {
            result.push(-1); // -1 represents ellipsis
        }
        
        // Pages around current page
        const start = Math.max(2, current - 1);
        const end = Math.min(total - 1, current + 1);
        
        for (let i = start; i <= end; i++) {
            result.push(i);
        }
        
        // Handle ellipsis after current page
        if (current < total - 2) {
            result.push(-1); // -1 represents ellipsis
        }
        
        // Always include last page
        if (total > 1) {
            result.push(total);
        }
        
        return result;
    }
    
    function changePage(page: number): void {
        if (page >= 1 && page <= totalPages) {
            onPageChange(page);
        }
    }
</script>

<div class="flex justify-center items-center space-x-1 mt-4">
    <!-- Previous button -->
    <button 
        class="px-3 py-1 text-sm border dark:border-gray-700 {currentPage === 1 ? 'text-gray-400 cursor-not-allowed' : 'hover:bg-gray-100 dark:hover:bg-gray-800'}"
        disabled={currentPage === 1}
        on:click={() => changePage(currentPage - 1)}
    >
        Previous
    </button>
    
    <!-- Page numbers -->
    {#each pageNumbers as pageNum}
        {#if pageNum === -1}
            <!-- Ellipsis -->
            <span class="px-3 py-1 text-gray-500">...</span>
        {:else}
            <button 
                class="px-3 py-1 text-sm {pageNum === currentPage ? 'bg-blue-500 text-white' : 'hover:bg-gray-100 dark:hover:bg-gray-800 border dark:border-gray-700'}"
                on:click={() => changePage(pageNum)}
            >
                {pageNum}
            </button>
        {/if}
    {/each}
    
    <!-- Next button -->
    <button 
        class="px-3 py-1 text-sm border dark:border-gray-700 {currentPage === totalPages ? 'text-gray-400 cursor-not-allowed' : 'hover:bg-gray-100 dark:hover:bg-gray-800'}"
        disabled={currentPage === totalPages}
        on:click={() => changePage(currentPage + 1)}
    >
        Next
    </button>
</div>