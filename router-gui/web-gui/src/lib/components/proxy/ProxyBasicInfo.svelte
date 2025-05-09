<script lang="ts">
    export let title = "";
    export let listen = "";
</script>

<div class="space-y-4">
    <div>
        <label for="title" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
            Title
        </label>
        <input 
            type="text" 
            id="title" 
            bind:value={title}
            class="w-full p-2 rounded-md border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
            required
        />
    </div>
    
    <div>
        <label for="listen" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
            Listen (IP and Port)
        </label>
        <input 
            type="text" 
            id="listen" 
            bind:value={listen}
            class="w-full p-2 rounded-md border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
            required
            placeholder="Example: 0.0.0.0:8080"
        />
    </div>
</div> 