<script lang="ts">
    import { createEventDispatcher } from 'svelte';
    import Button from '$lib/components/common/Button.svelte';
    import type { ModelType } from '$lib/types/aiSecurity';

    export let showModal = false;
    export let isUploading = false;
    export let error: string | null = null;

    const dispatch = createEventDispatcher<{
        upload: { name: string; modelType: ModelType; file: File; version?: string };
        cancel: void;
    }>();

    let name = '';
    let modelType: ModelType = 'xgboost';
    let version = '';
    let file: File | null = null;
    let fileInput: HTMLInputElement;

    function handleFileChange(event: Event) {
        const target = event.target as HTMLInputElement;
        if (target.files && target.files.length > 0) {
            file = target.files[0];
        }
    }

    function handleSubmit() {
        if (!name || !file) return;
        dispatch('upload', {
            name,
            modelType,
            file,
            version: version || undefined
        });
    }

    function handleCancel() {
        resetForm();
        dispatch('cancel');
    }

    function resetForm() {
        name = '';
        modelType = 'xgboost';
        version = '';
        file = null;
        if (fileInput) fileInput.value = '';
    }

    $: isValid = name.trim() !== '' && file !== null;
</script>

{#if showModal}
    <div class="fixed inset-0 bg-black/30 backdrop-blur-sm flex items-center justify-center z-50">
        <div class="bg-white dark:bg-gray-800 rounded-lg p-6 max-w-md w-full mx-4 shadow-xl">
            <h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-4">
                Upload AI Model
            </h2>

            {#if error}
                <div class="bg-red-50 dark:bg-red-900/20 border-l-4 border-red-500 p-3 mb-4">
                    <p class="text-sm text-red-700 dark:text-red-300">{error}</p>
                </div>
            {/if}

            <form on:submit|preventDefault={handleSubmit} class="space-y-4">
                <div>
                    <label for="name" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                        Model Name *
                    </label>
                    <input
                        type="text"
                        id="name"
                        bind:value={name}
                        class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
                        placeholder="e.g., XGBoost Anomaly Detector"
                        disabled={isUploading}
                    />
                </div>

                <div>
                    <label for="modelType" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                        Model Type *
                    </label>
                    <select
                        id="modelType"
                        bind:value={modelType}
                        class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
                        disabled={isUploading}
                    >
                        <option value="xgboost">XGBoost</option>
                        <option value="isolation">Isolation Forest</option>
                    </select>
                </div>

                <div>
                    <label for="version" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                        Version (optional)
                    </label>
                    <input
                        type="text"
                        id="version"
                        bind:value={version}
                        class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white"
                        placeholder="e.g., 1.0.0"
                        disabled={isUploading}
                    />
                </div>

                <div>
                    <label for="file" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                        ONNX File *
                    </label>
                    <input
                        type="file"
                        id="file"
                        accept=".onnx"
                        on:change={handleFileChange}
                        bind:this={fileInput}
                        class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:text-white file:mr-4 file:py-1 file:px-4 file:rounded file:border-0 file:text-sm file:font-medium file:bg-blue-50 file:text-blue-700 dark:file:bg-blue-900 dark:file:text-blue-300"
                        disabled={isUploading}
                    />
                    {#if file}
                        <p class="mt-1 text-sm text-gray-500 dark:text-gray-400">
                            Selected: {file.name} ({(file.size / 1024 / 1024).toFixed(2)} MB)
                        </p>
                    {/if}
                </div>

                <div class="flex justify-end gap-3 pt-4">
                    <Button variant="secondary" onClick={handleCancel} disabled={isUploading}>
                        Cancel
                    </Button>
                    <Button type="submit" variant="primary" disabled={!isValid || isUploading}>
                        {#if isUploading}
                            <span class="flex items-center">
                                <svg class="animate-spin -ml-1 mr-2 h-4 w-4 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                                </svg>
                                Uploading...
                            </span>
                        {:else}
                            Upload Model
                        {/if}
                    </Button>
                </div>
            </form>
        </div>
    </div>
{/if}
