import { gwnodeService } from '$lib/services/gwnodeService';
import { gwNodes } from "$lib/stores/gwnodeStore";
import type { CreateGwNodeRequest, UpdateGwNodeRequest, GwNode } from "$lib/types/gwnode";
import { get } from "svelte/store";


// Gateway node actions to be used by components
export const gwnodeActions = {
    // Fetch all gateway nodes
    getAllGwNodes: async (): Promise<GwNode[]> => {
        try {
            return await gwnodeService.getAllGwNodes();
        } catch (error) {
            console.error('Failed to fetch gateway nodes:', error);
            return []; // Return an empty array on error
        }
    },
    
    // Get a single gateway node by ID
    getGwNodeById: async (nodeId: string): Promise<GwNode | null> => {
        try {
            return await gwnodeService.getGwNodeById(nodeId);
        } catch (error) {
            console.error(`Failed to fetch gateway node ${nodeId}:`, error);
            return null;
        }
    },
    
    // Get gateway nodes for a specific proxy
    getGwNodesByProxyId: async (proxyId: string): Promise<GwNode[]> => {
        try {
            return await gwnodeService.getGwNodesByProxyId(proxyId);
        } catch (error) {
            console.error(`Failed to fetch gateway nodes for proxy ${proxyId}:`, error);
            return []; // Return an empty array on error
        }
    },
    
    // Get all available gateway nodes (both unbound and those assigned to a specific proxy)
    getAvailableGwNodesForProxy: async (proxyId?: string): Promise<GwNode[]> => {
        try {
            // Get all gateway nodes
            const allNodes = await gwnodeService.getAllGwNodes();
            console.log("[gwnodeActions] Raw nodes fetched:", JSON.stringify(allNodes, null, 2)); // Log raw data

            if (!proxyId) {
                // For new proxies, only unbound nodes are available
                console.log("[gwnodeActions] Filtering for unbound nodes (new proxy)");
                const filteredNodes = allNodes.filter(node => {
                    const isUnbound = node.proxy_id === "unbound";
                    // console.log(`[gwnodeActions] Node ${node.id} (${node.title}): proxy_id='${node.proxy_id}', isUnbound=${isUnbound}`); // Uncomment for detailed node logging
                    return isUnbound;
                });
                console.log("[gwnodeActions] Filtered unbound nodes:", JSON.stringify(filteredNodes, null, 2));
                return filteredNodes;
            } else {
                // For existing proxies, both unbound nodes and nodes already assigned to this proxy are available
                console.log(`[gwnodeActions] Filtering for proxy ID: ${proxyId} or unbound`);
                const filteredNodes = allNodes.filter(node => {
                    const isUnbound = node.proxy_id === "unbound";
                    const matchesProxyId = node.proxy_id === proxyId;
                    // console.log(`[gwnodeActions] Node ${node.id} (${node.title}): proxy_id='${node.proxy_id}', isUnbound=${isUnbound}, matchesProxyId=${matchesProxyId}`); // Uncomment for detailed node logging
                    return isUnbound || matchesProxyId;
                });
                console.log("[gwnodeActions] Filtered nodes for existing proxy:", JSON.stringify(filteredNodes, null, 2));
                return filteredNodes;
            }
        } catch (error) {
            console.error('[gwnodeActions] Failed to fetch available gateway nodes:', error);
            return []; // Return an empty array on error
        }
    },


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
     * Create a new gateway node
     * @param gwnode The gateway node data to create
     * @returns The created gateway node
     */
    async createGwNode(gwnode: CreateGwNodeRequest): Promise<GwNode> {
        try {
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
    },

    /**
     * Sync gateway nodes with the server
     * @returns A message with the result of the synchronization
     */
    async syncGatewayNodes(): Promise<{ status: string, message: string }> {
        try {
            const result = await gwnodeService.syncGatewayNodes();
            
            // Reload the nodes after sync to ensure the store is up to date
            await gwnodeActions.loadAllGwNodes();
            
            return result;
        } catch (error) {
            console.error("Failed to sync gateway nodes:", error);
            throw error;
        }
    }
};

export default gwnodeActions;
