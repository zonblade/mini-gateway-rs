import { writable } from 'svelte/store';

// Define interfaces
export interface Proxy {
    id: number;
    title: string;
    listen: string;
}

// Create a writable store with mock data
export const proxies = writable<Proxy[]>([
    { id: 1, title: "Main Proxy", listen: "0.0.0.0:8080" },
    { id: 2, title: "Secure API", listen: "0.0.0.0:443" },
    { id: 3, title: "Internal Service", listen: "127.0.0.1:9000" },
    { id: 4, title: "Legacy App", listen: "192.168.1.10:8000" },
    { id: 5, title: "Custom SSL", listen: "0.0.0.0:8443" },
]);

// Helper function to get a specific proxy by ID
export function getProxyById(id: number): Proxy | undefined {
    let result: Proxy | undefined;
    proxies.subscribe(items => {
        result = items.find(proxy => proxy.id === id);
    })();
    return result;
}