import { writable, derived } from 'svelte/store';
import type { Proxy, ProxyWithDomains, DomainConfig } from '$lib/types/proxy';
import { proxyActions } from '$lib/actions/proxyActions';

// Re-export the types for convenience
export type { Proxy, ProxyWithDomains, DomainConfig } from '$lib/types/proxy';

// Define the store state interface
interface ProxyState {
    proxies: ProxyWithDomains[];
    loading: boolean;
    loadError: boolean;
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
    loadError: false,
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
                if (proxies === null) {
                    throw new Error('No proxies found');
                }
                set({
                    ...initialState,
                    proxies,
                    hasMore: false,  // All data is loaded at once
                    loading: false,
                    loadError: false
                });
            } catch (err) {
                console.error('Error fetching proxies:', err);
                update(state => ({ 
                    ...state, 
                    loading: false, 
                    loadError: true,
                    error: err instanceof Error ? err.message : 'An unknown error occurred' 
                }));
            }
        },
        
        // Create or update a proxy with domains
        saveProxy: async (data: any) => {
            update(state => ({ ...state, loading: true, error: null }));
            
            try {
                // Check if data is in the new format (with proxy and domains properties)
                const isNewFormat = data && data.proxy && 'domains' in data;
                let savedProxyWithDomains;
                
                if (isNewFormat) {
                    // New format: data is already { proxy, domains }
                    savedProxyWithDomains = await proxyActions.saveProxy(data.proxy, data.domains);
                } else {
                    // Legacy format: data is just a Proxy object, possibly with tls_domains
                    const proxy = data as Proxy;
                    savedProxyWithDomains = await proxyActions.saveProxy(proxy);
                }
                
                update(state => {
                    // If it's an existing proxy (non-empty ID), update it in the array
                    const proxyId = isNewFormat ? data.proxy.id : data.id;
                    
                    if (proxyId && proxyId !== '') {
                        const index = state.proxies.findIndex(p => p.proxy.id === proxyId);
                        if (index >= 0) {
                            const updatedProxies = [...state.proxies];
                            updatedProxies[index] = savedProxyWithDomains;
                            return { ...state, proxies: updatedProxies, loading: false };
                        }
                    }
                    
                    // Otherwise add as new proxy
                    return { 
                        ...state, 
                        proxies: [...state.proxies, savedProxyWithDomains], 
                        loading: false,
                        loadError: false
                    };
                });
                
                return savedProxyWithDomains;
            } catch (err) {
                console.error('Error saving proxy:', err);
                update(state => ({ 
                    ...state, 
                    loading: false, 
                    error: err instanceof Error ? err.message : 'An unknown error occurred',
                    loadError: true
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
                        proxies: state.proxies.filter(p => p.proxy.id !== id),
                        loading: false,
                        loadError: false
                    }));
                }
                
                return success;
            } catch (err) {
                console.error('Error deleting proxy:', err);
                update(state => ({ 
                    ...state, 
                    loading: false, 
                    loadError: true,
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
                update(state => ({ ...state, loading: false, loadError: false }));
                return result;
            } catch (err) {
                console.error('Error syncing proxies:', err);
                update(state => ({ 
                    ...state, 
                    loading: false, 
                    loadError: true,
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
                ? proxies.filter(proxyWithDomains => {
                    const proxy = proxyWithDomains.proxy;
                    const domains = proxyWithDomains.domains || [];
                    
                    // Check if the search term matches proxy data
                    const matchesProxy = 
                        proxy.title.toLowerCase().includes(searchTerm.toLowerCase()) ||
                        proxy.addr_listen.toLowerCase().includes(searchTerm.toLowerCase()) ||
                        (proxy.addr_target && proxy.addr_target.toLowerCase().includes(searchTerm.toLowerCase()));
                    
                    // Check if the search term matches any domain
                    const matchesDomain = domains.some(domain => 
                        domain.sni && domain.sni.toLowerCase().includes(searchTerm.toLowerCase())
                    );
                    
                    return matchesProxy || matchesDomain;
                })
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