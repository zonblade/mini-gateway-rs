import { writable } from 'svelte/store';
import type { Gateway } from '$lib/types/gateway';

// Create a writable store with initial empty array
export const gateways = writable<Gateway[]>([]);

// Helper function to get a specific gateway by ID
export function getGatewayById(id: string): Gateway | undefined {
    let result: Gateway | undefined;
    gateways.subscribe(items => {
        result = items.find(item => item.id === id);
    })();
    return result;
}