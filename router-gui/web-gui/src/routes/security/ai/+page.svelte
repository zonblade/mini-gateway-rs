<script lang="ts">
    import { onMount } from 'svelte';
    import { goto } from '$app/navigation';
    import { user } from '$lib/stores/userStore';
    import { aiSecurityStore } from '$lib/stores/aiSecurityStore';
    import Button from '$lib/components/common/Button.svelte';
    import LoadingSpinner from '$lib/components/common/LoadingSpinner.svelte';
    import StatsOverview from '$lib/components/ai-security/StatsOverview.svelte';
    import ModelTable from '$lib/components/ai-security/ModelTable.svelte';
    import ModelUploadModal from '$lib/components/ai-security/ModelUploadModal.svelte';
    import ModelEditModal from '$lib/components/ai-security/ModelEditModal.svelte';
    import DeleteModelModal from '$lib/components/ai-security/DeleteModelModal.svelte';
    import type { AiModel, ModelType } from '$lib/types/aiSecurity';

    // Auth check
    let isLoggedIn = false;
    let isAuthLoading = true;

    const unsubAuth = user.subscribe((value) => {
        isLoggedIn = !!value;
        isAuthLoading = false;
    });

    // Store state
    $: ({
        models,
        stats,
        isLoading,
        isUploading,
        isProcessing,
        error
    } = $aiSecurityStore);

    // Modal state
    let showUploadModal = false;
    let showEditModal = false;
    let showDeleteModal = false;
    let selectedModel: AiModel | null = null;

    onMount(() => {
        if (!isAuthLoading && !isLoggedIn) {
            goto('/');
        } else if (isLoggedIn) {
            aiSecurityStore.loadModels();
            aiSecurityStore.loadStats();
        }
        return () => unsubAuth();
    });

    $: if (!isAuthLoading && !isLoggedIn) {
        goto('/');
    }

    // Handlers
    function handleOpenUpload() {
        showUploadModal = true;
    }

    async function handleUpload(event: CustomEvent<{ name: string; modelType: ModelType; file: File; version?: string }>) {
        const { name, modelType, file, version } = event.detail;
        const success = await aiSecurityStore.uploadModel(name, modelType, file, version);
        if (success) {
            showUploadModal = false;
            aiSecurityStore.loadStats();
        }
    }

    function handleCancelUpload() {
        showUploadModal = false;
        aiSecurityStore.clearError();
    }

    async function handleToggle(event: CustomEvent<{ id: string; enabled: boolean }>) {
        const { id, enabled } = event.detail;
        await aiSecurityStore.toggleModel(id, enabled);
        aiSecurityStore.loadStats();
    }

    function handleEdit(event: CustomEvent<AiModel>) {
        selectedModel = event.detail;
        showEditModal = true;
    }

    async function handleSaveEdit(event: CustomEvent<{ id: string; name: string; version?: string }>) {
        const { id, name, version } = event.detail;
        const success = await aiSecurityStore.updateModel({ id, name, version });
        if (success) {
            showEditModal = false;
            selectedModel = null;
        }
    }

    function handleCancelEdit() {
        showEditModal = false;
        selectedModel = null;
        aiSecurityStore.clearError();
    }

    function handleDelete(event: CustomEvent<AiModel>) {
        selectedModel = event.detail;
        showDeleteModal = true;
    }

    async function handleConfirmDelete(event: CustomEvent<string>) {
        const id = event.detail;
        const success = await aiSecurityStore.deleteModel(id);
        if (success) {
            showDeleteModal = false;
            selectedModel = null;
            aiSecurityStore.loadStats();
        }
    }

    function handleCancelDelete() {
        showDeleteModal = false;
        selectedModel = null;
        aiSecurityStore.clearError();
    }
</script>

<svelte:head>
    <title>AI Security Models</title>
</svelte:head>

<div class="p-6 mx-auto max-w-6xl">
    {#if isAuthLoading || isLoading}
        <div class="flex justify-center items-center min-h-[400px]">
            <LoadingSpinner />
        </div>
    {:else if isLoggedIn}
        <!-- Header -->
        <div class="flex justify-between items-center mb-6">
            <div>
                <h1 class="text-2xl font-bold text-gray-900 dark:text-white">AI Security Models</h1>
                <p class="text-sm text-gray-600 dark:text-gray-400 mt-1">
                    Manage ONNX models for anomaly detection and security analysis
                </p>
            </div>
            <Button variant="primary" onClick={handleOpenUpload}>
                Upload Model
            </Button>
        </div>

        <!-- Error display -->
        {#if error}
            <div class="bg-red-50 dark:bg-red-900/20 border-l-4 border-red-500 p-4 mb-6">
                <div class="flex">
                    <div class="flex-shrink-0">
                        <svg class="h-5 w-5 text-red-400" viewBox="0 0 20 20" fill="currentColor">
                            <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
                        </svg>
                    </div>
                    <div class="ml-3">
                        <p class="text-sm text-red-700 dark:text-red-300">{error}</p>
                    </div>
                    <div class="ml-auto pl-3">
                        <button
                            on:click={() => aiSecurityStore.clearError()}
                            class="text-red-500 hover:text-red-700 dark:text-red-400 dark:hover:text-red-300"
                        >
                            <svg class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                                <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd" />
                            </svg>
                        </button>
                    </div>
                </div>
            </div>
        {/if}

        <!-- Stats Overview -->
        <StatsOverview
            totalModels={stats?.total_models ?? 0}
            enabledModels={stats?.enabled_models ?? 0}
            totalInferences={stats?.total_inferences ?? 0}
        />

        <!-- Models Table -->
        <ModelTable
            {models}
            {isProcessing}
            on:toggle={handleToggle}
            on:edit={handleEdit}
            on:delete={handleDelete}
        />

        <!-- Modals -->
        <ModelUploadModal
            showModal={showUploadModal}
            {isUploading}
            {error}
            on:upload={handleUpload}
            on:cancel={handleCancelUpload}
        />

        <ModelEditModal
            showModal={showEditModal}
            model={selectedModel}
            {isProcessing}
            {error}
            on:save={handleSaveEdit}
            on:cancel={handleCancelEdit}
        />

        <DeleteModelModal
            showModal={showDeleteModal}
            model={selectedModel}
            {isProcessing}
            {error}
            on:confirm={handleConfirmDelete}
            on:cancel={handleCancelDelete}
        />
    {/if}
</div>
