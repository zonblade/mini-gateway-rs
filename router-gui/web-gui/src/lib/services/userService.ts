import { user } from '$lib/stores/userStore';
import type { ApiUser, CreateUserRequest, UpdateUserRequest, DeleteUserResponse } from '$lib/types/userTypes';

// Helper function to get the current API base URL from the user store
function getApiBaseUrl(): string {
    let apiUrl: string = '';
    user.subscribe(value => {
        apiUrl = value?.api_base_url || '/api/v1';
    })();
    return apiUrl;
}

// Helper function to get the auth token from the store
function getAuthToken(): string | null {
    let token: string | null = null;
    user.subscribe(value => {
        token = value?.token || null;
    })();
    return token;
}

// Helper function to create request headers with auth token
function getHeaders(): Record<string, string> {
    const token = getAuthToken();
    const headers: Record<string, string> = {
        'Content-Type': 'application/json',
    };
    
    if (token) {
        headers['Authorization'] = `Bearer ${token}`;
    }
    
    return headers;
}

// User management API service
export const userService = {
    // Get all users (admin only)
    getAllUsers: async (): Promise<ApiUser[]> => {
        try {
            const baseUrl = getApiBaseUrl();
            const response = await fetch(`${baseUrl}/users/admin`, {
                method: 'GET',
                headers: getHeaders()
            });
            
            if (!response.ok) {
                throw new Error(`Error fetching users: ${response.statusText}`);
            }
            
            return await response.json();
        } catch (error) {
            console.error('Failed to fetch users:', error);
            return []; // Return an empty array on error
        }
    },
    
    // Get a single user by ID
    getUserById: async (userId: string): Promise<ApiUser> => {
        try {
            const baseUrl = getApiBaseUrl();
            const response = await fetch(`${baseUrl}/users/${userId}`, {
                method: 'GET',
                headers: getHeaders()
            });
            
            if (!response.ok) {
                throw new Error(`Error fetching user: ${response.statusText}`);
            }
            
            return await response.json();
        } catch (error) {
            console.error(`Failed to fetch user ${userId}:`, error);
            throw error;
        }
    },
    
    // Create a new user (admin only)
    createUser: async (userData: CreateUserRequest): Promise<ApiUser> => {
        try {
            const baseUrl = getApiBaseUrl();
            const response = await fetch(`${baseUrl}/users/admin`, {
                method: 'POST',
                headers: getHeaders(),
                body: JSON.stringify(userData)
            });
            
            if (!response.ok) {
                throw new Error(`Error creating user: ${response.statusText}`);
            }
            
            return await response.json();
        } catch (error) {
            console.error('Failed to create user:', error);
            throw error;
        }
    },
    
    // Update an existing user
    updateUser: async (userId: string, userData: UpdateUserRequest): Promise<ApiUser> => {
        try {
            const baseUrl = getApiBaseUrl();
            const response = await fetch(`${baseUrl}/users/${userId}`, {
                method: 'PUT',
                headers: getHeaders(),
                body: JSON.stringify(userData)
            });
            
            if (!response.ok) {
                throw new Error(`Error updating user: ${response.statusText}`);
            }
            
            return await response.json();
        } catch (error) {
            console.error(`Failed to update user ${userId}:`, error);
            throw error;
        }
    },
    
    // Delete a user
    deleteUser: async (userId: string): Promise<DeleteUserResponse> => {
        try {
            const baseUrl = getApiBaseUrl();
            const response = await fetch(`${baseUrl}/users/${userId}`, {
                method: 'DELETE',
                headers: getHeaders()
            });
            
            if (!response.ok) {
                throw new Error(`Error deleting user: ${response.statusText}`);
            }
            
            return await response.json();
        } catch (error) {
            console.error(`Failed to delete user ${userId}:`, error);
            throw error;
        }
    }
};

export default userService;