<script lang="ts">
    import type { User } from '$lib/types/userTypes';
    import Button from '../common/Button.svelte';

    export let user: User;
    export let onEdit: (user: User) => void;
    export let onDelete: (id: string, email: string) => void;
    export let disabled: boolean = false;
</script>

<div class="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 p-4 hover:shadow-md transition-shadow">
    <div class="flex items-start justify-between">
        <div class="flex-1">
            <div class="flex items-center gap-2 mb-2">
                <h3 class="text-lg font-semibold text-gray-900 dark:text-white">
                    {user.username}
                </h3>
                <span class="px-2 py-1 text-xs
                    {user.role === 'Admin' ? 'bg-purple-100 dark:bg-purple-900/30 text-purple-800 dark:text-purple-300' : 
                    user.role === 'Staff' ? 'bg-blue-100 dark:bg-blue-900/30 text-blue-800 dark:text-blue-300' : 
                    user.role === 'Guest' ? 'bg-yellow-100 dark:bg-yellow-900/30 text-yellow-800 dark:text-yellow-300' : 
                    'bg-green-100 dark:bg-green-900/30 text-green-800 dark:text-green-300'}">
                    {user.role}
                </span>
            </div>
            
            <div class="space-y-2">
                <div class="flex items-center text-sm text-gray-600 dark:text-gray-300">
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 8l7.89 5.26a2 2 0 002.22 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
                    </svg>
                    {user.email}
                </div>
                
                <div class="flex items-center text-sm">
                    <span class="inline-flex items-center px-2 py-1 text-xs
                        {user.active ? 'bg-green-100 dark:bg-green-900/30 text-green-800 dark:text-green-300' : 
                        'bg-red-100 dark:bg-red-900/30 text-red-800 dark:text-red-300'}">
                        {user.active ? 'Active' : 'Inactive'}
                    </span>
                </div>
            </div>
        </div>

        <div class="flex items-center gap-2">
            <Button
                variant="secondary"
                size="sm"
                onClick={() => onEdit(user)}
                disabled={disabled}
            >
                <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                </svg>
            </Button>
            <Button
                variant="danger"
                size="sm"
                onClick={() => onDelete(user.id, user.email)}
                disabled={disabled}
            >
                <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                </svg>
            </Button>
        </div>
    </div>
</div> 