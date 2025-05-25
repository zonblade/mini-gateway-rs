import type { User, ApiUser, CreateUserRequest, UpdateUserRequest } from '$lib/types/userTypes';
import { userService } from '$lib/services/userService';

// Format API user data to our User type
const formatUser = (apiUser: ApiUser): User => ({
    id: apiUser.id,
    username: apiUser.username,
    email: apiUser.email,
    role: apiUser.role,
    active: true, // Default to true as this field doesn't exist in API
    created_at: apiUser.created_at,
    updated_at: apiUser.updated_at
});

// User actions to be used by components
export const userActions = {
    // Fetch all users
    getUsers: async (): Promise<User[]> => {
        try {
            const apiUsers = await userService.getAllUsers();
            return apiUsers.map(formatUser);
        } catch (error:any) {
            console.error('Failed to fetch users [2]:', error.error??error);
            return []; // Return an empty array on error
        }
    },

    // Get a single user by ID
    getUserById: async (userId: string): Promise<User> => {
        try {
            const apiUser = await userService.getUserById(userId);
            return formatUser(apiUser);
        } catch (error:any) {
            console.error(`Failed to fetch user ${userId}:`, error.error??error);
            throw error.error??error;
        }
    },

    // Create a new user
    createUser: async (userData: CreateUserRequest): Promise<User> => {
        try {
            const apiUser = await userService.createUser(userData);
            return formatUser(apiUser);
        } catch (error:any) {
            console.error('Failed to create user:', error.error??error);
            throw error.error??error;
        }
    },

    // Update an existing user
    updateUser: async (userId: string, userData: UpdateUserRequest): Promise<User> => {
        try {
            const apiUser = await userService.updateUser(userId, userData);
            return formatUser(apiUser);
        } catch (error:any) {
            console.error(`Failed to update user ${userId}:`, error.error??error);
            throw error.error??error;
        }
    },

    // Delete a user
    deleteUser: async (userId: string): Promise<boolean> => {
        try {
            await userService.deleteUser(userId);
            // According to API docs, a successful delete returns a message
            // If no exception is thrown, we assume success
            return true;
        } catch (error:any) {
            console.error(`Failed to delete user ${userId}:`, error.error??error);
            throw error.error??error;
        }
    }
};

export default userActions;