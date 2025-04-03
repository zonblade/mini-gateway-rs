import { writable } from 'svelte/store';
import { browser } from '$app/environment';

// Define user type
export interface User {
    username: string;
    token: string;
}

// Initialize the store with value from localStorage if available
const storedUser = browser ? localStorage.getItem('user') : null;
const initialUser = storedUser ? JSON.parse(storedUser) : null;

// Create the store
const userStore = writable<User | null>(initialUser);

// Export derived functions for more convenient use
export const user = {
    subscribe: userStore.subscribe,
    
    // Set user data and save to localStorage
    login: (username: string, password: string): Promise<void> => {
        return new Promise((resolve, reject) => {
            // Simulate API call
            setTimeout(() => {
                // For now, accept any username/password and create a token
                const token = `token_${Math.random().toString(36).substring(2)}`;
                const userData: User = { username, token };
                
                userStore.set(userData);
                
                // Save to localStorage
                if (browser) {
                    localStorage.setItem('user', JSON.stringify(userData));
                }
                
                resolve();
            }, 500);
        });
    },
    
    // Clear user data from store and localStorage
    logout: () => {
        userStore.set(null);
        if (browser) {
            localStorage.removeItem('user');
        }
    },
    
    // Check if user is logged in
    isLoggedIn: (): boolean => {
        let isLoggedIn = false;
        userStore.subscribe(value => {
            isLoggedIn = !!value;
        })();
        return isLoggedIn;
    }
};

export default user;