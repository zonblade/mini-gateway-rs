<script lang="ts">
    import { fade } from "svelte/transition";
    import type { User } from '$lib/types/userTypes';
    
    export let showModal: boolean = false;
    export let isEditMode: boolean = false;
    export let user: User = { id: "", username: "", email: "", role: "User", active: true };
    export let onSave: () => void;
    export let onClose: () => void;
    export let isProcessing: boolean = false;
    export let errorMessage: string | null = null;
    
    export let roles: string[] = ["Admin", "User", "Support", "Guest"];
    
    // Add password field (only used for new user creation)
    let password: string = "";
    // Password complexity check
    let passwordStrength: 'weak' | 'medium' | 'strong' | '' = '';
    
    $: {
        if (!isEditMode && password) {
            if (password.length < 6) {
                passwordStrength = 'weak';
            } else if (password.length < 10 || !/[A-Z]/.test(password) || !/[0-9]/.test(password)) {
                passwordStrength = 'medium';
            } else {
                passwordStrength = 'strong';
            }
        } else {
            passwordStrength = '';
        }
    }
    
    // Reset password when modal is opened/closed or mode changes
    $: if (showModal) {
        if (!isEditMode) {
            // Only reset password for new users
            password = "";
        }
    }
    
    function handleKeydown(event: KeyboardEvent) {
        if (event.key === 'Escape') {
            onClose();
        }
    }
    
    function handleModalKeyDown(event: KeyboardEvent) {
        // Keep events from propagating outside the modal
        event.stopPropagation();
    }
    
    // Function to get password for new user
    export function getPassword(): string {
        return password;
    }
</script>

{#if showModal}
    <div 
        class="fixed inset-0 bg-black/30 backdrop-blur-md bg-opacity-50 flex items-center justify-center z-50" 
        transition:fade={{ duration: 200 }}
        on:keydown={handleKeydown}
        role="presentation"
    >
        <div 
            class="bg-white dark:bg-[#161b22] shadow-xl max-w-md w-full mx-4"
            on:click|stopPropagation
            on:keydown={handleModalKeyDown}
            role="dialog"
            aria-labelledby="modal-title"
            aria-modal="true"
            tabindex="-1"
        >
            <div class="p-6">
                <div class="flex justify-between items-center mb-4">
                    <h2 id="modal-title" class="text-xl font-bold">{isEditMode ? 'Edit User' : 'Add User'}</h2>
                    <button 
                        on:click={onClose}
                        aria-label="Close"
                        class="text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                            <line x1="18" y1="6" x2="6" y2="18"></line>
                            <line x1="6" y1="6" x2="18" y2="18"></line>
                        </svg>
                    </button>
                </div>
                
                {#if errorMessage}
                    <div class="mb-4 p-3 bg-red-100 text-red-700">
                        <p>{errorMessage}</p>
                    </div>
                {/if}
                
                <form on:submit|preventDefault={onSave} class="space-y-4">
                    <div>
                        <label for="username" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                            Username
                        </label>
                        <input 
                            type="text" 
                            id="username" 
                            bind:value={user.username}
                            class="w-full p-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
                            required
                            disabled={isProcessing}
                        />
                    </div>
                    
                    <div>
                        <label for="email" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                            Email
                        </label>
                        <input 
                            type="email" 
                            id="email" 
                            bind:value={user.email}
                            class="w-full p-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
                            required
                            disabled={isProcessing}
                        />
                    </div>
                    
                    <!-- Password field - only shown when creating a new user -->
                    {#if !isEditMode}
                        <div>
                            <label for="password" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                Password
                            </label>
                            <input 
                                type="password" 
                                id="password" 
                                bind:value={password}
                                class="w-full p-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
                                required
                                disabled={isProcessing}
                                placeholder="Enter a secure password"
                                minlength="6"
                            />
                            {#if passwordStrength}
                                <div class="mt-1 text-sm">
                                    <div class="flex items-center">
                                        <span class="text-gray-600 dark:text-gray-400 mr-2">Strength:</span>
                                        <div class="h-1.5 w-24 bg-gray-200 dark:bg-gray-700 overflow-hidden">
                                            <div class="h-full {passwordStrength === 'weak' ? 'w-1/3 bg-red-500' : passwordStrength === 'medium' ? 'w-2/3 bg-yellow-500' : 'w-full bg-green-500'}"></div>
                                        </div>
                                        <span class="ml-2 {passwordStrength === 'weak' ? 'text-red-500' : passwordStrength === 'medium' ? 'text-yellow-500' : 'text-green-500'}">
                                            {passwordStrength}
                                        </span>
                                    </div>
                                </div>
                            {/if}
                        </div>
                    {/if}
                    
                    <div>
                        <label for="role" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                            Role
                        </label>
                        <select 
                            id="role" 
                            bind:value={user.role}
                            class="w-full p-2 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
                            disabled={isProcessing}
                        >
                            {#each roles as role}
                                <option value={role}>{role}</option>
                            {/each}
                        </select>
                    </div>
                    
                    <div class="flex items-center">
                        <input 
                            type="checkbox" 
                            id="active" 
                            bind:checked={user.active}
                            class="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300"
                            disabled={isProcessing}
                        />
                        <label for="active" class="ml-2 block text-sm text-gray-700 dark:text-gray-300">
                            Active
                        </label>
                    </div>
                    
                    <div class="flex justify-end space-x-2 pt-4">
                        <button 
                            type="button"
                            on:click={onClose}
                            class="px-4 py-2 bg-gray-200 hover:bg-gray-300 dark:bg-gray-700 dark:hover:bg-gray-600 text-sm font-medium text-gray-700 dark:text-gray-200 border border-transparent hover:border-gray-400 dark:hover:border-gray-500"
                            disabled={isProcessing}
                        >
                            Cancel
                        </button>
                        <button 
                            type="submit"
                            class="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white text-sm font-medium border border-transparent"
                            disabled={isProcessing}
                        >
                            {isEditMode ? 'Update' : 'Create'}
                            {#if isProcessing}
                                <span class="ml-2 inline-block animate-spin">&#8635;</span>
                            {/if}
                        </button>
                    </div>
                </form>
            </div>
        </div>
    </div>
{/if}