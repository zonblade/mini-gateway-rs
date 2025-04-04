<script lang="ts">
    import type { GwNode } from "$lib/types/gwnode";
    
    export let gwnode: GwNode;
</script>

<div class="bg-white dark:bg-gray-800 rounded-lg shadow-md overflow-hidden">
    <div class="p-6">
        <h1 class="text-2xl font-bold text-gray-900 dark:text-white mb-4">
            {gwnode.title}
        </h1>

        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
            <!-- Proxy Information -->
            <div class="space-y-3">
                <h3 class="text-lg font-medium text-gray-700 dark:text-gray-300">
                    Proxy Information
                </h3>
                <div class="bg-gray-50 dark:bg-gray-700 p-4 rounded-md">
                    <div class="mb-1">
                        <span class="font-medium">Name:</span>
                        {gwnode.proxyTitle}
                    </div>
                    <div>
                        <span class="font-medium">Source:</span>
                        <code class="text-sm bg-gray-200 dark:bg-gray-600 px-1 py-0.5 rounded"
                            >{gwnode.source}</code
                        >
                    </div>
                </div>
            </div>

            <!-- Target Information -->
            <div class="space-y-3">
                <h3 class="text-lg font-medium text-gray-700 dark:text-gray-300">
                    Target Information
                </h3>
                <div class="bg-gray-50 dark:bg-gray-700 p-4 rounded-md">
                    <div>
                        <span class="font-medium">Address:</span>
                        <code class="text-sm bg-gray-200 dark:bg-gray-600 px-1 py-0.5 rounded"
                            >{gwnode.alt_target}</code
                        >
                    </div>
                </div>
            </div>
        </div>
    </div>
</div>