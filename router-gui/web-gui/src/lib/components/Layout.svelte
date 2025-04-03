<!-- Layout Wrapper Component -->
<script lang="ts">
    export let isLoggedIn: boolean = false;
    // Converting unused exports to constants since they appear to be for external reference only
    export const username: string = "";
    export const onLogout: () => void = () => {};
</script>

{#if isLoggedIn}
    <div class="flex flex-col w-full h-screen bg-gray-50 dark:bg-[#0d1117] text-gray-900 dark:text-gray-100">
        <slot name="header" />
        
        <!-- Main content -->
        <main class="flex-1">
            <slot />
        </main>
        
        <slot name="footer" />
    </div>
{:else}
    <div class="flex flex-col w-full h-screen">
        <slot />
    </div>
{/if}