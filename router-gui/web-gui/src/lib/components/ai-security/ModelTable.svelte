<script lang="ts">
    import { createEventDispatcher } from 'svelte';
    import type { AiModel } from '$lib/types/aiSecurity';

    export let models: AiModel[] = [];
    export let isProcessing = false;

    const dispatch = createEventDispatcher<{
        toggle: { id: string; enabled: boolean };
        edit: AiModel;
        delete: AiModel;
    }>();

    function formatDate(dateStr: string | null): string {
        if (!dateStr) return '-';
        return new Date(dateStr).toLocaleString();
    }

    function formatNumber(num: number): string {
        return num.toLocaleString();
    }

    function handleToggle(model: AiModel) {
        dispatch('toggle', { id: model.id, enabled: !model.enabled });
    }
</script>

<div class="bg-white dark:bg-gray-800 rounded-lg shadow overflow-hidden">
    <div class="overflow-x-auto">
        <table class="w-full">
            <thead>
                <tr class="bg-gray-100 dark:bg-gray-700">
                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">Name</th>
                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">Type</th>
                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">Version</th>
                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">Status</th>
                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">Inferences</th>
                    <th class="px-4 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">Last Run</th>
                    <th class="px-4 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">Actions</th>
                </tr>
            </thead>
            <tbody class="divide-y divide-gray-200 dark:divide-gray-700">
                {#if models.length === 0}
                    <tr>
                        <td colspan="7" class="px-4 py-8 text-center text-gray-500 dark:text-gray-400">
                            No models found. Upload a model to get started.
                        </td>
                    </tr>
                {:else}
                    {#each models as model (model.id)}
                        <tr class="hover:bg-gray-50 dark:hover:bg-gray-700/50">
                            <td class="px-4 py-3">
                                <div class="text-sm font-medium text-gray-900 dark:text-white">{model.name}</div>
                                <div class="text-xs text-gray-500 dark:text-gray-400 font-mono">{model.id.slice(0, 8)}...</div>
                            </td>
                            <td class="px-4 py-3">
                                <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium
                                    {model.model_type === 'xgboost'
                                        ? 'bg-purple-100 text-purple-800 dark:bg-purple-900/30 dark:text-purple-300'
                                        : 'bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-300'}">
                                    {model.model_type === 'xgboost' ? 'XGBoost' : 'Isolation'}
                                </span>
                            </td>
                            <td class="px-4 py-3 text-sm text-gray-600 dark:text-gray-300">
                                {model.version || '-'}
                            </td>
                            <td class="px-4 py-3">
                                <button
                                    on:click={() => handleToggle(model)}
                                    disabled={isProcessing}
                                    class="relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 disabled:opacity-50
                                        {model.enabled ? 'bg-green-500' : 'bg-gray-300 dark:bg-gray-600'}"
                                >
                                    <span
                                        class="pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out
                                            {model.enabled ? 'translate-x-5' : 'translate-x-0'}"
                                    ></span>
                                </button>
                            </td>
                            <td class="px-4 py-3 text-sm text-gray-600 dark:text-gray-300">
                                {formatNumber(model.inference_count)}
                            </td>
                            <td class="px-4 py-3 text-sm text-gray-600 dark:text-gray-300">
                                {formatDate(model.last_inference)}
                            </td>
                            <td class="px-4 py-3 text-right">
                                <div class="flex justify-end gap-2">
                                    <button
                                        on:click={() => dispatch('edit', model)}
                                        class="text-blue-600 hover:text-blue-800 dark:text-blue-400 dark:hover:text-blue-300 text-sm font-medium"
                                        disabled={isProcessing}
                                    >
                                        Edit
                                    </button>
                                    <button
                                        on:click={() => dispatch('delete', model)}
                                        class="text-red-600 hover:text-red-800 dark:text-red-400 dark:hover:text-red-300 text-sm font-medium"
                                        disabled={isProcessing}
                                    >
                                        Delete
                                    </button>
                                </div>
                            </td>
                        </tr>
                    {/each}
                {/if}
            </tbody>
        </table>
    </div>
</div>
