<script lang="ts">
    export let id: string = "";
    export let name: string = "";
    export let label: string = "";
    export let value: string = "";
    export let placeholder: string = "";
    export let type: string = "text";
    export let required: boolean = false;
    export let disabled: boolean = false;
    export let error: string = "";
    
    function handleInput(event: Event) {
        const target = event.target as HTMLInputElement;
        value = target.value;
    }
</script>

<div>
    {#if label}
        <label for={id} class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
            {label}
            {#if required}<span class="text-red-500">*</span>{/if}
        </label>
    {/if}
    
    <input 
        {id}
        {name}
        {type}
        {placeholder}
        {required}
        {disabled}
        value={value}
        on:input={handleInput}
        class="w-full p-2 border
            {error ? 'border-red-500 dark:border-red-400' : 'border-gray-300 dark:border-gray-600'} 
            bg-white dark:bg-gray-700 
            text-gray-900 dark:text-gray-100 
            focus:ring-2 focus:ring-blue-500 focus:border-blue-500 dark:focus:ring-blue-400 dark:focus:border-blue-400"
    />
    
    {#if error}
        <p class="mt-1 text-sm text-red-600 dark:text-red-400">{error}</p>
    {/if}
</div>