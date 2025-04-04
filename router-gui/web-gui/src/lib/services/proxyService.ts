import { browser } from '$app/environment';
import type { Proxy } from '$lib/types/proxy';

const API_BASE = '/api/v1';

// Helper function to handle API responses
async function handleResponse<T>(response: Response): Promise<T> {
    if (!response.ok) {
        const errorData = await response.json().catch(() => null);
        throw new Error(errorData?.message || `API Error: ${response.statusText}`);
    }
    return await response.json() as T;
}

// Helper to get auth token from local storage
function getAuthHeader(): HeadersInit {
    if (!browser) return {};
    
    const token = localStorage.getItem('auth_token');
    if (!token) return {};
    
    return {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json'
    };
}

export const proxyService = {
    /**
     * Get all proxies
     */
    getAllProxies: async (): Promise<Proxy[]> => {
        const response = await fetch(`${API_BASE}/settings/proxies`, {
            headers: getAuthHeader()
        });
        return handleResponse<Proxy[]>(response);
    },
    
    /**
     * Get a proxy by ID
     */
    getProxyById: async (id: string): Promise<Proxy> => {
        const response = await fetch(`${API_BASE}/settings/proxy/${id}`, {
            headers: getAuthHeader()
        });
        return handleResponse<Proxy>(response);
    },
    
    /**
     * Create or update a proxy
     */
    saveProxy: async (proxy: Proxy): Promise<Proxy> => {
        const response = await fetch(`${API_BASE}/settings/proxy`, {
            method: 'POST',
            headers: getAuthHeader(),
            body: JSON.stringify(proxy)
        });
        return handleResponse<Proxy>(response);
    },
    
    /**
     * Delete a proxy
     */
    deleteProxy: async (id: string): Promise<{ message: string }> => {
        const response = await fetch(`${API_BASE}/settings/proxy/${id}`, {
            method: 'DELETE',
            headers: getAuthHeader()
        });
        return handleResponse<{ message: string }>(response);
    },
    
    /**
     * Sync proxy settings with the server
     */
    syncProxyNodes: async (): Promise<{ status: string, message: string }> => {
        const response = await fetch(`${API_BASE}/sync/node`, {
            method: 'POST',
            headers: getAuthHeader()
        });
        return handleResponse<{ status: string, message: string }>(response);
    }
};

export default proxyService;