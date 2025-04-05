import { gatewayService } from "$lib/services/gatewayService";
import { gateways } from "$lib/stores/gatewayStore";
import type { Gateway, CreateGatewayRequest, UpdateGatewayRequest } from "$lib/types/gateway";

/**
 * Actions for managing gateways and updating the store
 */
export const gatewayActions = {
    /**
     * Load all gateways
     * @returns Array of all gateways
     */
    async loadAllGateways(): Promise<Gateway[]> {
        try {
            console.log("Loading all gateways");
            const loadedGateways = await gatewayService.getAllGateways();
            gateways.set(loadedGateways);
            return loadedGateways;
        } catch (error) {
            console.error("Failed to load gateways:", error);
            throw error;
        }
    },

    /**
     * Load gateways for a specific gateway node
     * @param gwnodeId ID of the gateway node
     * @returns Array of gateways for the specified gateway node
     */
    async loadGatewaysByGwNodeId(gwnodeId: string): Promise<Gateway[]> {
        try {
            console.log(`Loading gateways for node ${gwnodeId}`);
            const loadedGateways = await gatewayService.getGatewaysByGwNodeId(gwnodeId);
            gateways.set(loadedGateways);
            return loadedGateways;
        } catch (error) {
            console.error(`Failed to load gateways for node ${gwnodeId}:`, error);
            throw error;
        }
    },

    /**
     * Get a specific gateway by ID
     * @param id ID of the gateway to get
     * @returns The gateway or null if not found
     */
    async getGatewayById(id: string): Promise<Gateway | null> {
        try {
            return await gatewayService.getGatewayById(id);
        } catch (error) {
            console.error(`Failed to get gateway ${id}:`, error);
            throw error;
        }
    },

    /**
     * Create a new gateway
     * @param gateway The gateway data to create
     * @returns The created gateway
     */
    async createGateway(gateway: CreateGatewayRequest): Promise<Gateway> {
        try {
            console.log("Creating new gateway:", gateway);
            const createdGateway = await gatewayService.createGateway(gateway);
            
            // Update the store by adding the new gateway
            gateways.update(items => [...items, createdGateway]);
            
            return createdGateway;
        } catch (error) {
            console.error("Failed to create gateway:", error);
            throw error;
        }
    },

    /**
     * Update an existing gateway
     * @param gateway The gateway data to update
     * @returns The updated gateway
     */
    async updateGateway(gateway: UpdateGatewayRequest): Promise<Gateway> {
        try {
            console.log(`Updating gateway ${gateway.id}:`, gateway);
            const updatedGateway = await gatewayService.updateGateway(gateway);
            
            // Update the store by replacing the updated gateway
            gateways.update(items => 
                items.map(item => item.id === updatedGateway.id ? updatedGateway : item)
            );
            
            return updatedGateway;
        } catch (error) {
            console.error(`Failed to update gateway ${gateway.id}:`, error);
            throw error;
        }
    },

    /**
     * Delete a gateway
     * @param id ID of the gateway to delete
     * @returns A message indicating success
     */
    async deleteGateway(id: string): Promise<string> {
        try {
            console.log(`Deleting gateway ${id}`);
            const result = await gatewayService.deleteGateway(id);
            
            // Update the store by removing the deleted gateway
            gateways.update(items => items.filter(item => item.id !== id));
            
            return result.message;
        } catch (error) {
            console.error(`Failed to delete gateway ${id}:`, error);
            throw error;
        }
    }
};