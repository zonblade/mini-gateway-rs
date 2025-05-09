import { browser } from '$app/environment';
import type { Proxy, ProxyWithDomains, TlsDomain } from '$lib/types/proxy';
import { user } from '$lib/stores/userStore';

// Helper function to get the current API base URL from the user store
function getApiBaseUrl(): string {
    let apiUrl: string = '';
    user.subscribe(value => {
        apiUrl = value?.api_base_url || '/api/v1';
    })();
    return apiUrl;
}

// Helper function to handle API responses
async function handleResponse<T>(response: Response): Promise<T> {
    if (!response.ok) {
        const errorData = await response.json().catch(() => null);
        console.error('API Error:', errorData);
        throw new Error(errorData?.error || `API Error: ${response.statusText}`);
    }
    return await response.json() as T;
}

// Helper function to get the auth token from the store
function getAuthToken(): string | null {
    let token: string | null = null;
    user.subscribe(value => {
        token = value?.token || null;
    })();
    return token;
}

// Helper to get auth token from local storage
function getAuthHeader(): HeadersInit {
    if (!browser) return {};
    
    const token = getAuthToken();
    if (!token) return {};
    
    return {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json'
    };
}

export const proxyService = {
    /**
     * Get all proxies with their domains
     */
    getAllProxies: async (): Promise<ProxyWithDomains[]> => {
        const baseUrl = getApiBaseUrl();
        const response = await fetch(`${baseUrl}/settings/proxies`, {
            headers: getAuthHeader()
        });
        return handleResponse<ProxyWithDomains[]>(response);
    },
    
    /**
     * Get a proxy by ID with all its domains
     */
    getProxyById: async (id: string): Promise<ProxyWithDomains> => {
        const baseUrl = getApiBaseUrl();
        const response = await fetch(`${baseUrl}/settings/proxy/${id}`, {
            headers: getAuthHeader()
        });
        return handleResponse<ProxyWithDomains>(response);
    },
    
    /**
     * Create or update a proxy
     * @param proxy The proxy to save
     * @param domains Optional domains to associate with the proxy
     */
    saveProxy: async (proxy: Proxy, domains?: TlsDomain[]): Promise<ProxyWithDomains> => {
        const baseUrl = getApiBaseUrl();
        
        // Format the payload according to the new API structure
        const payload = { 
            proxy, 
            domains: domains || [] 
        };
        
        const response = await fetch(`${baseUrl}/settings/proxy`, {
            method: 'POST',
            headers: getAuthHeader(),
            body: JSON.stringify(payload)
        });
        
        return handleResponse<ProxyWithDomains>(response);
    },
    
    /**
     * Delete a proxy
     */
    deleteProxy: async (id: string): Promise<{ message: string }> => {
        const baseUrl = getApiBaseUrl();
        const response = await fetch(`${baseUrl}/settings/proxy/${id}`, {
            method: 'DELETE',
            headers: getAuthHeader()
        });
        return handleResponse<{ message: string }>(response);
    },
    
    /**
     * Sync proxy settings with the server
     */
    syncProxyNodes: async (): Promise<{ status: string, message: string }> => {
        const baseUrl = getApiBaseUrl();
        const response = await fetch(`${baseUrl}/sync/proxy`, {
            method: 'POST',
            headers: getAuthHeader()
        });
        return handleResponse<{ status: string, message: string }>(response);
    }
};

export default proxyService;