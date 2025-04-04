import { writable } from 'svelte/store';
import { browser } from '$app/environment';
// Import other stores that need to be reset on logout
import { gwNodes } from './gwnodeStore';
import { proxyStore } from './proxyStore';
// Do not import connections store as we want to preserve it

// Define user type based on API response
export interface User {
    user_id: string;
    username: string;
    token: string;
    role: string;
    api_base_url?: string; // Store the API base URL with the user
}

// API response interface
interface LoginResponse {
    success: boolean;
    token: string;
    user_id: string;
    username: string;
    role: string;
    message: string;
}

// Initialize the store with value from localStorage if available
const storedUser = browser ? localStorage.getItem('user') : null;
const initialUser = storedUser ? JSON.parse(storedUser) : null;

// Create the store
const userStore = writable<User | null>(initialUser);

// Default API base URL
const DEFAULT_API_BASE_URL = '/api/v1';

// Export derived functions for more convenient use
export const user = {
    subscribe: userStore.subscribe,
    
    // Set user data and save to localStorage
    login: async (username: string, password: string, apiBaseUrl = DEFAULT_API_BASE_URL): Promise<void> => {
        try {
            const loginEndpoint = `${apiBaseUrl}/users/login`;
            
            const response = await fetch(loginEndpoint, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({ username, password }),
            });
            
            const data: LoginResponse = await response.json();
            
            if (!data.success) {
                throw new Error(data.message || 'Login failed');
            }
            
            const userData: User = {
                user_id: data.user_id,
                username: data.username,
                token: data.token,
                role: data.role,
                api_base_url: apiBaseUrl // Store the API base URL with the user data
            };
            
            userStore.set(userData);
            
            // Save to localStorage
            if (browser) {
                localStorage.setItem('user', JSON.stringify(userData));
            }
        } catch (error) {
            // Re-throw the error for the component to handle
            throw error;
        }
    },
    
    // Clear user data and reset other stores except connections
    logout: () => {
        // Clear user data from the store
        userStore.set(null);
        
        // Reset other stores to their initial state
        // GwNodes store - reset to empty array
        gwNodes.set([]);
        
        // Proxies store - reset to initial state
        proxyStore.reset();
        
        // Reset any localStorage data
        if (browser) {
            // Save connection-related data before clearing
            const connections = localStorage.getItem('connections');
            const lastConnectionId = localStorage.getItem('lastConnectionId');
            const rememberedUsername = localStorage.getItem('rememberedUsername');
            
            // Clear localStorage
            localStorage.clear();
            
            // Restore connection-related data
            if (connections) localStorage.setItem('connections', connections);
            if (lastConnectionId) localStorage.setItem('lastConnectionId', lastConnectionId);
            if (rememberedUsername) localStorage.setItem('rememberedUsername', rememberedUsername);
        }
    },
    
    // Check if user is logged in
    isLoggedIn: (): boolean => {
        let isLoggedIn = false;
        userStore.subscribe(value => {
            isLoggedIn = !!value;
        })();
        return isLoggedIn;
    },
    
    // Get current user role
    getRole: (): string | null => {
        let role = null;
        userStore.subscribe(value => {
            role = value?.role || null;
        })();
        return role;
    },
    
    // Get auth token
    getToken: (): string | null => {
        let token = null;
        userStore.subscribe(value => {
            token = value?.token || null;
        })();
        return token;
    },
    
    // Get API base URL
    getApiBaseUrl: (): string => {
        let url = DEFAULT_API_BASE_URL;
        userStore.subscribe(value => {
            url = value?.api_base_url || DEFAULT_API_BASE_URL;
        })();
        return url;
    }
};

export default user;