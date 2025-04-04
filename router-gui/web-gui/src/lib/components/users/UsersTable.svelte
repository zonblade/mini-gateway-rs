<script lang="ts">
    import type { User } from '$lib/types/userTypes';
    
    export let users: User[] = [];
    export let onEdit: (user: User) => void;
    export let onDelete: (id: string) => void;
    export let disabled: boolean = false;
</script>

<div class="overflow-x-auto">
    <table class="w-full text-left">
        <thead class="bg-gray-50 dark:bg-gray-800 text-gray-500 dark:text-gray-400 text-xs uppercase">
            <tr>
                <th class="py-3 px-4">Username</th>
                <th class="py-3 px-4">Email</th>
                <th class="py-3 px-4">Role</th>
                <th class="py-3 px-4">Status</th>
                <th class="py-3 px-4">Actions</th>
            </tr>
        </thead>
        <tbody class="divide-y divide-gray-200 dark:divide-gray-700">
            {#if users.length === 0}
                <tr>
                    <td colspan="5" class="py-4 px-4 text-center text-gray-500 dark:text-gray-400">
                        No users found
                    </td>
                </tr>
            {/if}
            
            {#each users as user (user.id)}
                <tr class="hover:bg-gray-50 dark:hover:bg-gray-800/50">
                    <td class="py-3 px-4">{user.username}</td>
                    <td class="py-3 px-4">{user.email}</td>
                    <td class="py-3 px-4">
                        <span class="px-2 py-1 text-xs rounded-full 
                            {user.role === 'Admin' ? 'bg-purple-100 dark:bg-purple-900/30 text-purple-800 dark:text-purple-300' : 
                            user.role === 'Staff' ? 'bg-blue-100 dark:bg-blue-900/30 text-blue-800 dark:text-blue-300' : 
                            user.role === 'Guest' ? 'bg-yellow-100 dark:bg-yellow-900/30 text-yellow-800 dark:text-yellow-300' : 
                            'bg-green-100 dark:bg-green-900/30 text-green-800 dark:text-green-300'}">
                            {user.role}
                        </span>
                    </td>
                    <td class="py-3 px-4">
                        <span class="inline-flex items-center px-2 py-1 text-xs rounded-full 
                            {user.active ? 'bg-green-100 dark:bg-green-900/30 text-green-800 dark:text-green-300' : 
                            'bg-red-100 dark:bg-red-900/30 text-red-800 dark:text-red-300'}">
                            {user.active ? 'Active' : 'Inactive'}
                        </span>
                    </td>
                    <td class="py-3 px-4">
                        <div class="flex space-x-2">
                            <button 
                                on:click={() => onEdit(user)}
                                aria-label="Edit user"
                                class="text-blue-600 hover:text-blue-900 dark:text-blue-400 dark:hover:text-blue-200"
                                disabled={disabled}
                            >
                                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                    <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path>
                                    <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path>
                                </svg>
                            </button>
                            <button 
                                on:click={() => onDelete(user.id)}
                                aria-label="Delete user"
                                class="text-red-600 hover:text-red-900 dark:text-red-400 dark:hover:text-red-200"
                                disabled={disabled}
                            >
                                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                    <polyline points="3 6 5 6 21 6"></polyline>
                                    <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
                                    <line x1="10" y1="11" x2="10" y2="17"></line>
                                    <line x1="14" y1="11" x2="14" y2="17"></line>
                                </svg>
                            </button>
                        </div>
                    </td>
                </tr>
            {/each}
        </tbody>
    </table>
</div>