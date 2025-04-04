import type { Proxy } from '$lib/types/proxy';
import { proxyService } from '$lib/services/proxyService';

// Proxy actions to be used by components
export const proxyActions = {
    // Fetch all proxies
    getProxies: async (): Promise<Proxy[]> => {
        try {
            return await proxyService.getAllProxies();
        } catch (error) {
            console.error('Failed to fetch proxies:', error);
            return []; // Return an empty array on error
        }
    },
    
    // Get a single proxy by ID
    getProxyById: async (proxyId: string): Promise<Proxy> => {
        try {
            return await proxyService.getProxyById(proxyId);
        } catch (error) {
            console.error(`Failed to fetch proxy ${proxyId}:`, error);
            throw error;
        }
    },
    
    // Create or update a proxy
    saveProxy: async (proxy: Proxy): Promise<Proxy> => {
        try {
            return await proxyService.saveProxy(proxy);
        } catch (error) {
            console.error('Failed to save proxy:', error);
            throw error;
        }
    },
    
    // Delete a proxy
    deleteProxy: async (proxyId: string): Promise<boolean> => {
        try {
            await proxyService.deleteProxy(proxyId);
            return true;
        } catch (error) {
            console.error(`Failed to delete proxy ${proxyId}:`, error);
            throw error;
        }
    },
    
    // Sync proxies with server nodes
    syncProxies: async (): Promise<{ status: string, message: string }> => {
        try {
            return await proxyService.syncProxyNodes();
        } catch (error) {
            console.error('Failed to sync proxy nodes:', error);
            throw error;
        }
    }
};

export default proxyActions;