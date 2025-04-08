import type { GwNode, CreateGwNodeRequest, UpdateGwNodeRequest, DeleteGwNodeRequest } from "$lib/types/gwnode";
import { user } from '$lib/stores/userStore';

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

/**
 * Service for managing gateway nodes through the API
 */
export const gwnodeService = {
    /**
     * Fetch all gateway nodes
     * @returns Promise with array of gateway nodes
     */
    async getAllGwNodes(): Promise<GwNode[]> {
        try {
            const baseUrl = getApiBaseUrl();
            const response = await fetch(`${baseUrl}/settings/gwnode/list`, {
                method: 'GET',
                headers: getHeaders()
            });

            if (!response.ok) {
                throw new Error(`Failed to fetch gateway nodes: ${response.statusText}`);
            }

            return await response.json();
        } catch (error) {
            console.error('Error fetching gateway nodes:', error);
            throw error;
        }
    },

    /**
     * Fetch gateway nodes associated with a specific proxy
     * @param proxyId The ID of the proxy
     * @returns Promise with array of gateway nodes
     */
    async getGwNodesByProxyId(proxyId: string): Promise<GwNode[]> {
        try {
            const baseUrl = getApiBaseUrl();
            const response = await fetch(`${baseUrl}/settings/gwnode/list/${proxyId}`, {
                method: 'GET',
                headers: getHeaders()
            });

            if (!response.ok) {
                throw new Error(`Failed to fetch gateway nodes for proxy: ${response.statusText}`);
            }

            return await response.json();
        } catch (error) {
            console.error(`Error fetching gateway nodes for proxy ${proxyId}:`, error);
            throw error;
        }
    },

    /**
     * Fetch a specific gateway node by ID
     * @param id The ID of the gateway node
     * @returns Promise with the gateway node or null if not found
     */
    async getGwNodeById(id: string): Promise<GwNode | null> {
        try {
            const baseUrl = getApiBaseUrl();
            const response = await fetch(`${baseUrl}/settings/gwnode/${id}`, {
                method: 'GET',
                headers: getHeaders()
            });

            if (response.status === 404) {
                return null;
            }

            if (!response.ok) {
                throw new Error(`Failed to fetch gateway node: ${response.statusText}`);
            }

            return await response.json();
        } catch (error) {
            console.error(`Error fetching gateway node ${id}:`, error);
            throw error;
        }
    },

    /**
     * Create a new gateway node
     * @param gwnode The gateway node data to create
     * @returns Promise with the created gateway node
     */
    async createGwNode(gwnode: CreateGwNodeRequest): Promise<GwNode> {
        try {
            const baseUrl = getApiBaseUrl();
            const response = await fetch(`${baseUrl}/settings/gwnode/set`, {
                method: 'POST',
                headers: getHeaders(),
                body: JSON.stringify(gwnode)
            });

            if (!response.ok) {
                throw new Error(`Failed to create gateway node: ${response.statusText}`);
            }

            return await response.json();
        } catch (error) {
            console.error('Error creating gateway node:', error);
            throw error;
        }
    },

    /**
     * Update an existing gateway node
     * @param gwnode The gateway node data to update
     * @returns Promise with the updated gateway node
     */
    async updateGwNode(gwnode: UpdateGwNodeRequest): Promise<GwNode> {
        try {
            const baseUrl = getApiBaseUrl();
            const response = await fetch(`${baseUrl}/settings/gwnode/set`, {
                method: 'POST',
                headers: getHeaders(),
                body: JSON.stringify(gwnode)
            });

            if (!response.ok) {
                throw new Error(`Failed to update gateway node: ${response.statusText}`);
            }

            return await response.json();
        } catch (error) {
            console.error(`Error updating gateway node ${gwnode.id}:`, error);
            throw error;
        }
    },

    /**
     * Delete a gateway node
     * @param id The ID of the gateway node to delete
     * @returns Promise indicating success or failure
     */
    async deleteGwNode(id: string): Promise<{ message: string }> {
        try {
            const baseUrl = getApiBaseUrl();
            const request: DeleteGwNodeRequest = { id };
            const response = await fetch(`${baseUrl}/settings/gwnode/delete`, {
                method: 'POST',
                headers: getHeaders(),
                body: JSON.stringify(request)
            });

            if (!response.ok) {
                throw new Error(`Failed to delete gateway node: ${response.statusText}`);
            }

            return await response.json();
        } catch (error) {
            console.error(`Error deleting gateway node ${id}:`, error);
            throw error;
        }
    },
    
    /**
     * Sync gateway nodes with the server
     * @returns Promise with the result of the synchronization
     */
    async syncGatewayNodes(): Promise<{ status: string, message: string }> {
        try {
            const baseUrl = getApiBaseUrl();
            const response = await fetch(`${baseUrl}/sync/gateway`, {
                method: 'POST',
                headers: getHeaders()
            });

            if (!response.ok) {
                throw new Error(`Failed to sync gateway nodes: ${response.statusText}`);
            }

            return await response.json();
        } catch (error) {
            console.error('Error syncing gateway nodes:', error);
            throw error;
        }
    }
};