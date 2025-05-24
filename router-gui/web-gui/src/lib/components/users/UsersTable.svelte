<script lang="ts">
    import type { User } from '$lib/types/userTypes';
    import DeleteConfirmationModal from "$lib/components/common/DeleteConfirmationModal.svelte";
    import UserCard from './UserCard.svelte';
    
    export let users: User[] = [];
    export let onEdit: (user: User) => void;
    export let onDelete: (id: string) => void;
    export let disabled: boolean = false;

    let showDeleteModal = false;
    let userToDelete: { id: string; email: string } | null = null;
    let isProcessing = false;

    function handleDeleteClick(id: string, email: string) {
        userToDelete = { id, email };
        showDeleteModal = true;
    }

    async function handleDeleteConfirm() {
        if (!userToDelete) return;
        
        try {
            isProcessing = true;
            await onDelete(userToDelete.id);
            showDeleteModal = false;
            userToDelete = null;
        } catch (error) {
            console.error('Error deleting user:', error);
        } finally {
            isProcessing = false;
        }
    }

    function handleDeleteCancel() {
        showDeleteModal = false;
        userToDelete = null;
    }
</script>

<div class="space-y-4">
    {#if users.length === 0}
        <div class="text-center py-8 text-gray-500 dark:text-gray-400">
            No users found
        </div>
    {:else}
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {#each users as user (user.id)}
                <UserCard
                    {user}
                    {onEdit}
                    onDelete={handleDeleteClick}
                    {disabled}
                />
            {/each}
        </div>
    {/if}
</div>

<DeleteConfirmationModal
    showModal={showDeleteModal}
    type="user"
    addressToVerify={userToDelete?.email || ''}
    {isProcessing}
    on:confirm={handleDeleteConfirm}
    on:cancel={handleDeleteCancel}
/>