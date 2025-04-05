import type { Gateway, CreateGatewayRequest, UpdateGatewayRequest, DeleteGatewayResponse } from "$lib/types/gateway";
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
 * Service for managing gateways through the API
 */
export const gatewayService = {
    /**
     * Fetch all gateways
     * @returns Promise with array of gateways
     */
    async getAllGateways(): Promise<Gateway[]> {
        try {
            const baseUrl = getApiBaseUrl();
            const response = await fetch(`${baseUrl}/settings/gateway/list`, {
                method: 'GET',
                headers: getHeaders()
            });

            if (!response.ok) {
                throw new Error(`Failed to fetch gateways: ${response.statusText}`);
            }

            return await response.json();
        } catch (error) {
            console.error('Error fetching gateways:', error);
            throw error;
        }
    },

    /**
     * Fetch gateways associated with a specific gateway node
     * @param gwnodeId The ID of the gateway node
     * @returns Promise with array of gateways
     */
    async getGatewaysByGwNodeId(gwnodeId: string): Promise<Gateway[]> {
        try {
            const baseUrl = getApiBaseUrl();
            const response = await fetch(`${baseUrl}/settings/gateway/list/${gwnodeId}`, {
                method: 'GET',
                headers: getHeaders()
            });

            if (!response.ok) {
                throw new Error(`Failed to fetch gateways for gateway node: ${response.statusText}`);
            }

            return await response.json();
        } catch (error) {
            console.error(`Error fetching gateways for gateway node ${gwnodeId}:`, error);
            throw error;
        }
    },

    /**
     * Fetch a specific gateway by ID
     * @param id The ID of the gateway
     * @returns Promise with the gateway or null if not found
     */
    async getGatewayById(id: string): Promise<Gateway | null> {
        try {
            const baseUrl = getApiBaseUrl();
            const response = await fetch(`${baseUrl}/settings/gateway/${id}`, {
                method: 'GET',
                headers: getHeaders()
            });

            if (response.status === 404) {
                return null;
            }

            if (!response.ok) {
                throw new Error(`Failed to fetch gateway: ${response.statusText}`);
            }

            return await response.json();
        } catch (error) {
            console.error(`Error fetching gateway ${id}:`, error);
            throw error;
        }
    },

    /**
     * Create a new gateway
     * @param gateway The gateway data to create
     * @returns Promise with the created gateway
     */
    async createGateway(gateway: CreateGatewayRequest): Promise<Gateway> {
        try {
            const baseUrl = getApiBaseUrl();
            const response = await fetch(`${baseUrl}/settings/gateway/set`, {
                method: 'POST',
                headers: getHeaders(),
                body: JSON.stringify(gateway)
            });

            if (!response.ok) {
                throw new Error(`Failed to create gateway: ${response.statusText}`);
            }

            return await response.json();
        } catch (error) {
            console.error('Error creating gateway:', error);
            throw error;
        }
    },

    /**
     * Update an existing gateway
     * @param gateway The gateway data to update
     * @returns Promise with the updated gateway
     */
    async updateGateway(gateway: UpdateGatewayRequest): Promise<Gateway> {
        try {
            const baseUrl = getApiBaseUrl();
            const response = await fetch(`${baseUrl}/settings/gateway/set`, {
                method: 'POST',
                headers: getHeaders(),
                body: JSON.stringify(gateway)
            });

            if (!response.ok) {
                throw new Error(`Failed to update gateway: ${response.statusText}`);
            }

            return await response.json();
        } catch (error) {
            console.error(`Error updating gateway ${gateway.id}:`, error);
            throw error;
        }
    },

    /**
     * Delete a gateway
     * @param id The ID of the gateway to delete
     * @returns Promise indicating success or failure
     */
    async deleteGateway(id: string): Promise<DeleteGatewayResponse> {
        try {
            const baseUrl = getApiBaseUrl();
            const request = { id };
            const response = await fetch(`${baseUrl}/settings/gateway/delete`, {
                method: 'POST',
                headers: getHeaders(),
                body: JSON.stringify(request)
            });

            if (!response.ok) {
                throw new Error(`Failed to delete gateway: ${response.statusText}`);
            }

            return await response.json();
        } catch (error) {
            console.error(`Error deleting gateway ${id}:`, error);
            throw error;
        }
    }
};