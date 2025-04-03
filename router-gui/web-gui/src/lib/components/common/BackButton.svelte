<script lang="ts">
    export let onClick = () => window.history.back();
    export let text = "Back";
</script>

<button
    on:click={onClick}
    class="mb-4 flex items-center text-blue-600 hover:text-blue-800 dark:text-blue-400 dark:hover:text-blue-200"
>
    <svg
        xmlns="http://www.w3.org/2000/svg"
        width="16"
        height="16"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
        class="mr-1"
    >
        <path d="M19 12H5M12 19l-7-7 7-7" />
    </svg>
    {text}
</button>