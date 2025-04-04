<script lang="ts">
    export let size = "h-12 w-12";
    export let borderColor = "border-blue-600";
</script>

<div class="flex items-center justify-center h-[400px]">
    <div
        class={`animate-spin rounded-full border-t-2 border-b-2 ${borderColor} ${size}`}
    ></div>
</div>