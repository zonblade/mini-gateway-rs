<script lang="ts">
    import { createEventDispatcher } from 'svelte';
    import Button from './Button.svelte';

    export let showModal = false;
    export let type: 'proxy' | 'gwnode' | 'user' = 'proxy';
    export let addressToVerify = '';
    export let isProcessing = false;

    const dispatch = createEventDispatcher<{
        confirm: void;
        cancel: void;
    }>();

    let inputValue = '';

    function handleConfirm() {
        if (inputValue === addressToVerify) {
            dispatch('confirm');
        }
    }

    function handleCancel() {
        inputValue = '';
        dispatch('cancel');
    }

    $: isInputValid = inputValue === addressToVerify;
</script>

{#if showModal}
    <div class="fixed inset-0 bg-black/30 backdrop-blur-sm flex items-center justify-center z-50">
        <div class="bg-white/95 dark:bg-gray-800/95 p-6 max-w-md w-full mx-4 shadow-xl">
            <h2 class="text-xl font-semibold text-gray-900 dark:text-white mb-4">
                Confirm Deletion
            </h2>
            
            <p class="text-gray-600 dark:text-gray-300 mb-4">
                This action cannot be undone. To confirm deletion, please type the {
                    type === 'proxy' ? 'listen address' : 
                    type === 'gwnode' ? 'target address' : 
                    'email address'
                } below:
            </p>
            
            <div class="mb-4">
                <p class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                    {type === 'proxy' ? 'Listen Address' : 
                     type === 'gwnode' ? 'Target Address' : 
                     'Email Address'}:
                </p>
                <code class="block bg-gray-100/80 dark:bg-gray-700/80 p-2 text-sm font-mono">
                    {addressToVerify}
                </code>
            </div>

            <div class="mb-4">
                <label for="verification" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                    Type the {type === 'proxy' ? 'listen address' : 
                             type === 'gwnode' ? 'target address' : 
                             'email address'} to confirm:
                </label>
                <input
                    type="text"
                    id="verification"
                    bind:value={inputValue}
                    class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 shadow-sm focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700/80 dark:text-white bg-white/80"
                    placeholder={`Type the ${type === 'proxy' ? 'listen address' : 
                                         type === 'gwnode' ? 'target address' : 
                                         'email address'} to confirm`}
                />
            </div>

            <div class="flex justify-end gap-3">
                <Button
                    variant="secondary"
                    onClick={handleCancel}
                    disabled={isProcessing}
                >
                    Cancel
                </Button>
                <Button
                    variant="danger"
                    onClick={handleConfirm}
                    disabled={!isInputValid || isProcessing}
                >
                    {#if isProcessing}
                        <span class="flex items-center">
                            <svg class="animate-spin -ml-1 mr-2 h-4 w-4 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                            </svg>
                            Deleting...
                        </span>
                    {:else}
                        Delete
                    {/if}
                </Button>
            </div>
        </div>
    </div>
{/if} 