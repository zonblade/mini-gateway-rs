<script lang="ts">
    import { createEventDispatcher } from 'svelte';
    import Button from '$lib/components/common/Button.svelte';
    import type { AiModel } from '$lib/types/aiSecurity';

    export let showModal = false;
    export let model: AiModel | null = null;
    export let isProcessing = false;
    export let error: string | null = null;

    const dispatch = createEventDispatcher<{
        save: { id: string; name: string; version?: string };
        cancel: void;
    }>();

    let name = '';
    let version = '';

    $: if (model) {
        name = model.name;
        version = model.version || '';
    }

    function handleSubmit() {
        if (!model || !name.trim()) return;
        dispatch('save', {
            id: model.id,
            name: name.trim(),
            version: version.trim() || undefined
        });
    }

    function handleCancel() {
        dispatch('cancel');
    }

    $: isValid = name.trim() !== '';
</script>

{#if showModal && model}
    <div class="fixed inset-0 bg-black/30 backdrop-blur-sm flex items-center justify-center z-50">
        <div class="bg-white dark:bg-gray-800 rounded-lg p-6 max-w-md w-full mx-4 shadow-xl">
            <h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-4">
                Edit Model
            </h2>

            {#if error}
                <div class="bg-red-50 dark:bg-red-900/20 border-l-4 border-red-500 p-3 mb-4">
                    <p class="text-sm text-red-700 dark:text-red-300">{error}</p>
                </div>
            {/if}

            <form on:submit|preventDefault={handleSubmit} class="space-y-4">
                <div>
                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                        Model ID
                    </label>
                    <p class="text-sm text-gray-500 dark:text-gray-400 font-mono bg-gray-100 dark:bg-gray-700 px-3 py-2 rounded">
                        {model.id}
                    </p>
                </div>

                <div>
                    <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                        Model Type
                    </label>
                    <p class="text-sm text-gray-600 dark:text-gray-300">
                        {model.model_type === 'xgboost' ? 'XGBoost' : 'Isolation Forest'}
                    </p>
                </div>

                <div>
                    <label for="editName" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                        Name *
                    </label>
                    <input
                        type="text"
                        id="editName"
                        bind:value={name}
                        class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
                        disabled={isProcessing}
                    />
                </div>

                <div>
                    <label for="editVersion" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                        Version
                    </label>
                    <input
                        type="text"
                        id="editVersion"
                        bind:value={version}
                        class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
                        placeholder="e.g., 1.0.0"
                        disabled={isProcessing}
                    />
                </div>

                <div class="flex justify-end gap-3 pt-4">
                    <Button variant="secondary" onClick={handleCancel} disabled={isProcessing}>
                        Cancel
                    </Button>
                    <Button type="submit" variant="primary" disabled={!isValid || isProcessing}>
                        {#if isProcessing}
                            <span class="flex items-center">
                                <svg class="animate-spin -ml-1 mr-2 h-4 w-4 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                                </svg>
                                Saving...
                            </span>
                        {:else}
                            Save Changes
                        {/if}
                    </Button>
                </div>
            </form>
        </div>
    </div>
{/if}
