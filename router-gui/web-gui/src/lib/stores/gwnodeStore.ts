import { writable } from 'svelte/store';

// Define interfaces
export interface GwNode {
    id: number;
    title: string;
    proxyId: string; // Changed from number to string
    proxyTitle: string;
    proxyListen: string;
    target: string;
}

// Create a writable store with mock data
export const gwNodes = writable<GwNode[]>([
    {
        id: 1,
        title: "API Gateway",
        proxyId: "1", // Changed to string
        proxyTitle: "Main Proxy",
        proxyListen: "0.0.0.0:8080",
        target: "192.168.1.100:3000",
    },
    {
        id: 2,
        title: "Web Server",
        proxyId: "2", // Changed to string
        proxyTitle: "Secure API",
        proxyListen: "0.0.0.0:443",
        target: "192.168.1.101:8080",
    },
    {
        id: 3,
        title: "Admin Panel",
        proxyId: "3", // Changed to string
        proxyTitle: "Internal Service",
        proxyListen: "127.0.0.1:9000",
        target: "192.168.1.102:8080",
    },
    {
        id: 4,
        title: "Database Access",
        proxyId: "4", // Changed to string
        proxyTitle: "Legacy App",
        proxyListen: "192.168.1.10:8000",
        target: "192.168.1.103:5432",
    },
    {
        id: 5,
        title: "Mail Server",
        proxyId: "5", // Changed to string
        proxyTitle: "Custom SSL",
        proxyListen: "0.0.0.0:8443",
        target: "192.168.1.104:25",
    },
    {
        id: 6,
        title: "File Server",
        proxyId: "1", // Changed to string
        proxyTitle: "Main Proxy",
        proxyListen: "0.0.0.0:8080",
        target: "192.168.1.105:21",
    },
    {
        id: 7,
        title: "Monitoring",
        proxyId: "3", // Changed to string
        proxyTitle: "Internal Service",
        proxyListen: "127.0.0.1:9000",
        target: "192.168.1.106:9090",
    },
    {
        id: 8,
        title: "Authentication",
        proxyId: "2", // Changed to string
        proxyTitle: "Secure API",
        proxyListen: "0.0.0.0:443",
        target: "192.168.1.107:8080",
    },
]);

// Helper function to get a specific node by ID
export function getNodeById(id: number): GwNode | undefined {
    let result: GwNode | undefined;
    gwNodes.subscribe(nodes => {
        result = nodes.find(node => node.id === id);
    })();
    return result;
}