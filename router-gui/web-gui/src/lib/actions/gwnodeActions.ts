import { gwnodeService } from "$lib/services/gwnodeService";
import { gwNodes } from "$lib/stores/gwnodeStore";
import type { CreateGwNodeRequest, UpdateGwNodeRequest, GwNode } from "$lib/types/gwnode";
import { get } from "svelte/store";

/**
 * Actions for managing gateway nodes
 */
export const gwnodeActions = {
    /**
     * Load all gateway nodes and update the store
     */
    async loadAllGwNodes(): Promise<void> {
        try {
            const nodes = await gwnodeService.getAllGwNodes();
            gwNodes.set(nodes);
        } catch (error) {
            console.error("Failed to load gateway nodes:", error);
            throw error;
        }
    },

    /**
     * Load gateway nodes for a specific proxy and update the store
     * @param proxyId The ID of the proxy
     */
    async loadGwNodesByProxy(proxyId: string): Promise<void> {
        try {
            const nodes = await gwnodeService.getGwNodesByProxyId(proxyId);
            gwNodes.set(nodes);
        } catch (error) {
            console.error(`Failed to load gateway nodes for proxy ${proxyId}:`, error);
            throw error;
        }
    },

    /**
     * Get a specific gateway node by ID
     * @param id The ID of the gateway node
     * @returns The gateway node or null if not found
     */
    async getGwNodeById(id: string): Promise<GwNode | null> {
        try {
            // First check the store for the node
            const nodesFromStore = get(gwNodes);
            const existingNode = nodesFromStore.find(node => node.id === id);
            
            if (existingNode) {
                return existingNode;
            }
            
            // If not in store, fetch from API
            return await gwnodeService.getGwNodeById(id);
        } catch (error) {
            console.error(`Failed to get gateway node ${id}:`, error);
            throw error;
        }
    },

    /**
     * Create a new gateway node
     * @param gwnode The gateway node data to create
     * @returns The created gateway node
     */
    async createGwNode(gwnode: CreateGwNodeRequest): Promise<GwNode> {
        try {
            console.log("Creating new gateway node:", gwnode);

            const createdNode = await gwnodeService.createGwNode(gwnode);
            
            // Update the store
            gwNodes.update(nodes => [...nodes, createdNode]);
            
            return createdNode;
        } catch (error) {
            console.error("Failed to create gateway node:", error);
            throw error;
        }
    },

    /**
     * Update an existing gateway node
     * @param gwnode The gateway node data to update
     * @returns The updated gateway node
     */
    async updateGwNode(gwnode: UpdateGwNodeRequest): Promise<GwNode> {
        try {
            const updatedNode = await gwnodeService.updateGwNode(gwnode);
            
            // Update the store
            gwNodes.update(nodes => {
                const index = nodes.findIndex(n => n.id === updatedNode.id);
                if (index !== -1) {
                    nodes[index] = updatedNode;
                }
                return [...nodes];
            });
            
            return updatedNode;
        } catch (error) {
            console.error(`Failed to update gateway node ${gwnode.id}:`, error);
            throw error;
        }
    },

    /**
     * Delete a gateway node
     * @param id The ID of the gateway node to delete
     * @returns A message indicating success
     */
    async deleteGwNode(id: string): Promise<string> {
        try {
            const result = await gwnodeService.deleteGwNode(id);
            
            // Update the store
            gwNodes.update(nodes => nodes.filter(node => node.id !== id));
            
            return result.message;
        } catch (error) {
            console.error(`Failed to delete gateway node ${id}:`, error);
            throw error;
        }
    }
};