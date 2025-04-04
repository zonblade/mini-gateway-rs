import { writable, derived } from 'svelte/store';
import type { User, ApiUser } from '$lib/types/userTypes';
import userService from '$lib/services/userService';

// Store state interface
interface UsersState {
    users: User[];
    filteredUsers: User[];
    paginatedUsers: User[];
    isLoading: boolean;
    isProcessing: boolean;
    error: string | null;
    searchTerm: string;
    currentPage: number;
    itemsPerPage: number;
    totalPages: number;
}

// Initial state
const initialState: UsersState = {
    users: [],
    filteredUsers: [],
    paginatedUsers: [],
    isLoading: false,
    isProcessing: false,
    error: null,
    searchTerm: '',
    currentPage: 1,
    itemsPerPage: 5,
    totalPages: 0
};

// Create the writable store
function createUsersStore() {
    const { subscribe, set, update } = writable<UsersState>(initialState);

    return {
        subscribe,
        
        // Reset the store to initial state
        reset: () => set(initialState),
        
        // Load all users from the API
        loadUsers: async () => {
            update(state => ({ ...state, isLoading: true, error: null }));
            
            try {
                const apiUsers = await userService.getAllUsers();
                
                // Convert API users to frontend User type
                const users = apiUsers.map((apiUser: ApiUser) => ({
                    id: apiUser.id,
                    username: apiUser.username,
                    email: apiUser.email,
                    role: apiUser.role,
                    active: true,
                    created_at: apiUser.created_at,
                    updated_at: apiUser.updated_at
                }));
                
                update(state => {
                    // Calculate derived values
                    const filteredUsers = applyFilter(users, state.searchTerm);
                    const totalPages = Math.ceil(filteredUsers.length / state.itemsPerPage);
                    const paginatedUsers = applyPagination(
                        filteredUsers, 
                        state.currentPage, 
                        state.itemsPerPage
                    );
                    
                    return {
                        ...state,
                        users,
                        filteredUsers,
                        paginatedUsers,
                        totalPages,
                        isLoading: false
                    };
                });
            } catch (error) {
                console.error('Failed to load users:', error);
                update(state => ({
                    ...state,
                    isLoading: false,
                    error: error instanceof Error ? error.message : 'Failed to load users'
                }));
            }
        },
        
        // Create a new user
        createUser: async (userData: { username: string; email: string; password: string; role: string }) => {
            update(state => ({ ...state, isProcessing: true, error: null }));
            
            try {
                const newApiUser = await userService.createUser(userData);
                
                update(state => {
                    // Add the new user to the list
                    const newUser: User = {
                        id: newApiUser.id,
                        username: newApiUser.username,
                        email: newApiUser.email,
                        role: newApiUser.role,
                        active: true,
                        created_at: newApiUser.created_at,
                        updated_at: newApiUser.updated_at
                    };
                    
                    const users = [...state.users, newUser];
                    
                    // Calculate derived values
                    const filteredUsers = applyFilter(users, state.searchTerm);
                    const totalPages = Math.ceil(filteredUsers.length / state.itemsPerPage);
                    const paginatedUsers = applyPagination(
                        filteredUsers, 
                        state.currentPage, 
                        state.itemsPerPage
                    );
                    
                    return {
                        ...state,
                        users,
                        filteredUsers,
                        paginatedUsers,
                        totalPages,
                        isProcessing: false
                    };
                });
                
                return true;
            } catch (error) {
                console.error('Failed to create user:', error);
                update(state => ({
                    ...state,
                    isProcessing: false,
                    error: error instanceof Error ? error.message : 'Failed to create user'
                }));
                return false;
            }
        },
        
        // Update an existing user
        updateUser: async (userId: string, userData: { username?: string; email?: string; role?: string }) => {
            update(state => ({ ...state, isProcessing: true, error: null }));
            
            try {
                const updatedApiUser = await userService.updateUser(userId, userData);
                
                update(state => {
                    // Update the user in the list
                    const users = state.users.map(user => 
                        user.id === userId ? {
                            ...user,
                            username: updatedApiUser.username,
                            email: updatedApiUser.email,
                            role: updatedApiUser.role,
                            updated_at: updatedApiUser.updated_at
                        } : user
                    );
                    
                    // Calculate derived values
                    const filteredUsers = applyFilter(users, state.searchTerm);
                    const totalPages = Math.ceil(filteredUsers.length / state.itemsPerPage);
                    const paginatedUsers = applyPagination(
                        filteredUsers, 
                        state.currentPage, 
                        state.itemsPerPage
                    );
                    
                    return {
                        ...state,
                        users,
                        filteredUsers,
                        paginatedUsers,
                        totalPages,
                        isProcessing: false
                    };
                });
                
                return true;
            } catch (error) {
                console.error(`Failed to update user ${userId}:`, error);
                update(state => ({
                    ...state,
                    isProcessing: false,
                    error: error instanceof Error ? error.message : 'Failed to update user'
                }));
                return false;
            }
        },
        
        // Delete a user
        deleteUser: async (userId: string) => {
            update(state => ({ ...state, isProcessing: true, error: null }));
            
            try {
                await userService.deleteUser(userId);
                
                update(state => {
                    // Remove the user from the list
                    const users = state.users.filter(user => user.id !== userId);
                    
                    // Calculate derived values
                    const filteredUsers = applyFilter(users, state.searchTerm);
                    const totalPages = Math.ceil(filteredUsers.length / state.itemsPerPage);
                    
                    // If we're on a page that no longer has items, go to the previous page
                    let currentPage = state.currentPage;
                    if (state.paginatedUsers.length === 1 && currentPage > 1) {
                        currentPage--;
                    }
                    
                    const paginatedUsers = applyPagination(
                        filteredUsers, 
                        currentPage, 
                        state.itemsPerPage
                    );
                    
                    return {
                        ...state,
                        users,
                        filteredUsers,
                        paginatedUsers,
                        totalPages,
                        currentPage,
                        isProcessing: false
                    };
                });
                
                return true;
            } catch (error) {
                console.error(`Failed to delete user ${userId}:`, error);
                update(state => ({
                    ...state,
                    isProcessing: false,
                    error: error instanceof Error ? error.message : 'Failed to delete user'
                }));
                return false;
            }
        },
        
        // Update search term and filter users
        setSearchTerm: (searchTerm: string) => {
            update(state => {
                const filteredUsers = applyFilter(state.users, searchTerm);
                const totalPages = Math.ceil(filteredUsers.length / state.itemsPerPage);
                const paginatedUsers = applyPagination(
                    filteredUsers, 
                    1, // Reset to first page when search term changes
                    state.itemsPerPage
                );
                
                return {
                    ...state,
                    searchTerm,
                    filteredUsers,
                    paginatedUsers,
                    totalPages,
                    currentPage: 1
                };
            });
        },
        
        // Change the current page
        setPage: (page: number) => {
            update(state => {
                const paginatedUsers = applyPagination(
                    state.filteredUsers, 
                    page, 
                    state.itemsPerPage
                );
                
                return {
                    ...state,
                    currentPage: page,
                    paginatedUsers
                };
            });
        },
        
        // Change items per page
        setItemsPerPage: (itemsPerPage: number) => {
            update(state => {
                const totalPages = Math.ceil(state.filteredUsers.length / itemsPerPage);
                const currentPage = Math.min(state.currentPage, totalPages || 1);
                const paginatedUsers = applyPagination(
                    state.filteredUsers, 
                    currentPage, 
                    itemsPerPage
                );
                
                return {
                    ...state,
                    itemsPerPage,
                    totalPages,
                    currentPage,
                    paginatedUsers
                };
            });
        },
        
        // Clear error message
        clearError: () => {
            update(state => ({ ...state, error: null }));
        }
    };
}

// Helper function to filter users based on search term
function applyFilter(users: User[], searchTerm: string): User[] {
    if (!searchTerm) return users;
    
    const term = searchTerm.toLowerCase();
    return users.filter(user => 
        user.username.toLowerCase().includes(term) ||
        user.email.toLowerCase().includes(term) ||
        user.role.toLowerCase().includes(term)
    );
}

// Helper function to paginate users
function applyPagination(users: User[], page: number, itemsPerPage: number): User[] {
    const startIndex = (page - 1) * itemsPerPage;
    return users.slice(startIndex, startIndex + itemsPerPage);
}

// Create and export the store
export const usersStore = createUsersStore();