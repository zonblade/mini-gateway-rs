import { writable, derived } from 'svelte/store';
import type { Proxy } from '$lib/types/proxy';
import { proxyActions } from '$lib/actions/proxyActions';

// Re-export the Proxy type for convenience
export type { Proxy } from '$lib/types/proxy';

// Define the store state interface
interface ProxyState {
    proxies: Proxy[];
    loading: boolean;
    error: string | null;
    currentPage: number;
    itemsPerPage: number;
    hasMore: boolean;
    searchTerm: string;
}

// Create initial state
const initialState: ProxyState = {
    proxies: [],
    loading: false,
    error: null,
    currentPage: 1,
    itemsPerPage: 10,
    hasMore: false,
    searchTerm: ''
};

// Create the store
function createProxyStore() {
    const { subscribe, set, update } = writable<ProxyState>(initialState);

    return {
        subscribe,
        
        // Reset the store to initial state
        reset: () => set(initialState),
        
        // Set search term
        setSearchTerm: (term: string) => update(state => ({ 
            ...state, 
            searchTerm: term, 
            currentPage: 1 // Reset to first page on new search
        })),
        
        // Load more items (client-side pagination)
        loadMore: () => update(state => ({ 
            ...state, 
            currentPage: state.currentPage + 1 
        })),
        
        // Fetch all proxies from API
        fetchProxies: async () => {
            update(state => ({ ...state, loading: true, error: null }));
            
            try {
                const proxies = await proxyActions.getProxies();
                set({
                    ...initialState,
                    proxies,
                    hasMore: false,  // All data is loaded at once
                    loading: false
                });
            } catch (err) {
                console.error('Error fetching proxies:', err);
                update(state => ({ 
                    ...state, 
                    loading: false, 
                    error: err instanceof Error ? err.message : 'An unknown error occurred' 
                }));
            }
        },
        
        // Create or update a proxy
        saveProxy: async (proxy: Proxy) => {
            update(state => ({ ...state, loading: true, error: null }));
            
            try {
                const savedProxy = await proxyActions.saveProxy(proxy);
                
                update(state => {
                    // If it's an existing proxy, update it in the array
                    if (proxy.id) {
                        const index = state.proxies.findIndex(p => p.id === proxy.id);
                        if (index >= 0) {
                            const updatedProxies = [...state.proxies];
                            updatedProxies[index] = savedProxy;
                            return { ...state, proxies: updatedProxies, loading: false };
                        }
                    }
                    
                    // Otherwise add as new proxy
                    return { 
                        ...state, 
                        proxies: [...state.proxies, savedProxy], 
                        loading: false 
                    };
                });
                
                return savedProxy;
            } catch (err) {
                console.error('Error saving proxy:', err);
                update(state => ({ 
                    ...state, 
                    loading: false, 
                    error: err instanceof Error ? err.message : 'An unknown error occurred' 
                }));
                throw err;
            }
        },
        
        // Delete a proxy
        deleteProxy: async (id: string) => {
            update(state => ({ ...state, loading: true, error: null }));
            
            try {
                const success = await proxyActions.deleteProxy(id);
                
                if (success) {
                    update(state => ({
                        ...state,
                        proxies: state.proxies.filter(p => p.id !== id),
                        loading: false
                    }));
                }
                
                return success;
            } catch (err) {
                console.error('Error deleting proxy:', err);
                update(state => ({ 
                    ...state, 
                    loading: false, 
                    error: err instanceof Error ? err.message : 'An unknown error occurred' 
                }));
                return false;
            }
        },
        
        // Sync proxies with the server
        syncProxies: async () => {
            update(state => ({ ...state, loading: true, error: null }));
            
            try {
                const result = await proxyActions.syncProxies();
                update(state => ({ ...state, loading: false }));
                return result;
            } catch (err) {
                console.error('Error syncing proxies:', err);
                update(state => ({ 
                    ...state, 
                    loading: false, 
                    error: err instanceof Error ? err.message : 'An unknown error occurred' 
                }));
                throw err;
            }
        },
        
        // Get filtered and paginated proxies
        getFilteredProxies: (state: ProxyState) => {
            const { proxies, searchTerm, currentPage, itemsPerPage } = state;
            
            // Filter proxies based on search term
            const filtered = searchTerm 
                ? proxies.filter(proxy => 
                    proxy.title.toLowerCase().includes(searchTerm.toLowerCase()) ||
                    proxy.addr_listen.toLowerCase().includes(searchTerm.toLowerCase()) ||
                    proxy.addr_target.toLowerCase().includes(searchTerm.toLowerCase())
                )
                : proxies;
                
            // Calculate pagination
            const totalPages = Math.ceil(filtered.length / itemsPerPage);
            const hasMore = currentPage < totalPages;
            
            // Get current page items
            const paginatedProxies = filtered.slice(0, currentPage * itemsPerPage);
            
            return {
                proxies: paginatedProxies,
                hasMore
            };
        }
    };
}

// Export the store instance
export const proxyStore = createProxyStore();

// Create a derived store for just the proxies array
export const proxies = derived(
    proxyStore,
    $proxyStore => $proxyStore.proxies
);