import { writable } from 'svelte/store';
import type { GwNode } from '$lib/types/gwnode';

// Create a writable store with initial empty array
export const gwNodes = writable<GwNode[]>([]);

// Helper function to get a specific node by ID
export function getNodeById(id: string): GwNode | undefined {
    let result: GwNode | undefined;
    gwNodes.subscribe(nodes => {
        result = nodes.find(node => node.id === id);
    })();
    return result;
}