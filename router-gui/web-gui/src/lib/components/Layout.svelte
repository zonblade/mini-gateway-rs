<!-- Layout Wrapper Component -->
<script lang="ts">
    export let isLoggedIn: boolean = false;
    // Converting unused exports to constants since they appear to be for external reference only
    export const username: string = "";
    export const onLogout: () => void = () => {};
</script>

{#if isLoggedIn}
    <div
        class="flex flex-col w-full h-screen bg-gray-50 dark:bg-[#0d1117] text-gray-900 dark:text-gray-100"
    >
        <slot name="header" />

        <!-- Main content with proper centering -->
        <main class="flex-1 w-full">
            <div class="mx-auto max-w-7xl lg:pl-64 px-4 sm:px-6 lg:px-8">
                <div class="py-6">
                    <slot />
                </div>
            </div>
        </main>

        <slot name="footer" />
    </div>
{:else}
    <div
        class="flex flex-col items-center justify-center w-full h-screen bg-gray-50 dark:bg-[#0d1117] text-gray-900 dark:text-gray-100"
    >
        <div class="w-full max-w-[900px] px-4">
            <slot />
        </div>
    </div>
{/if}
