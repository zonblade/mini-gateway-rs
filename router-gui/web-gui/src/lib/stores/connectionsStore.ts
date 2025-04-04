import { writable } from 'svelte/store';
import { browser } from '$app/environment';

export interface Connection {
    id: string;
    name: string;
    host: string;
    port: number;
}

const storedConnections = browser ? localStorage.getItem('connections') : null;
const initialConnections: Connection[] = storedConnections ? JSON.parse(storedConnections) : [
    {
        id: '1',
        name: 'Default Connection',
        host: 'localhost',
        port: 24042
    }
];

const connectionsStore = writable<Connection[]>(initialConnections);

export const connections = {
    subscribe: connectionsStore.subscribe,
    
    addConnection: (connection: Omit<Connection, 'id'>) => {
        const id = crypto.randomUUID();
        connectionsStore.update(connections => {
            const newConnections = [...connections, { ...connection, id }];
            if (browser) {
                localStorage.setItem('connections', JSON.stringify(newConnections));
            }
            return newConnections;
        });
    },
    
    updateConnection: (id: string, connection: Partial<Omit<Connection, 'id'>>) => {
        connectionsStore.update(connections => {
            const newConnections = connections.map(conn => 
                conn.id === id ? { ...conn, ...connection } : conn
            );
            if (browser) {
                localStorage.setItem('connections', JSON.stringify(newConnections));
            }
            return newConnections;
        });
    },
    
    removeConnection: (id: string) => {
        connectionsStore.update(connections => {
            // Don't allow removing the last connection
            if (connections.length <= 1) {
                return connections;
            }
            
            const newConnections = connections.filter(conn => conn.id !== id);
            if (browser) {
                localStorage.setItem('connections', JSON.stringify(newConnections));
            }
            return newConnections;
        });
    },
    
    getConnectionById: (id: string): Connection | undefined => {
        let foundConnection: Connection | undefined;
        connectionsStore.subscribe(connections => {
            foundConnection = connections.find(conn => conn.id === id);
        })();
        return foundConnection;
    }
};

export default connections;