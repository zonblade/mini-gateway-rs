import { addDataPoint, connectionStatus, clearBuffer, type StatsDataPoint } from '$lib/stores/unifiedStatsStore';
import { user } from '$lib/stores/userStore';

let eventSource: EventSource | null = null;
let reconnectTimeout: ReturnType<typeof setTimeout> | null = null;

function getApiBaseUrl(): string {
    let apiUrl: string = '/api/v1';
    user.subscribe(value => {
        apiUrl = value?.api_base_url || '/api/v1';
    })();
    return apiUrl;
}

/** Connect to the unified stats SSE endpoint */
export function connectSSE(): void {
    // Cleanup any existing connection
    disconnectSSE();

    const baseUrl = getApiBaseUrl();
    const url = `${baseUrl}/statistics/stream`;

    console.log('[SSE] Connecting to:', url);

    eventSource = new EventSource(url);

    eventSource.onopen = () => {
        console.log('[SSE] Connected');
        connectionStatus.set('connected');
    };

    eventSource.onmessage = (event) => {
        try {
            const data: StatsDataPoint = JSON.parse(event.data);
            addDataPoint(data);
        } catch (e) {
            console.error('[SSE] Failed to parse data:', e);
        }
    };

    eventSource.onerror = (error) => {
        console.error('[SSE] Connection error:', error);
        connectionStatus.set('disconnected');

        // Auto-reconnect after 3 seconds
        if (!reconnectTimeout) {
            reconnectTimeout = setTimeout(() => {
                reconnectTimeout = null;
                console.log('[SSE] Attempting reconnect...');
                connectSSE();
            }, 3000);
        }
    };
}

/** Disconnect from SSE */
export function disconnectSSE(): void {
    if (reconnectTimeout) {
        clearTimeout(reconnectTimeout);
        reconnectTimeout = null;
    }

    if (eventSource) {
        eventSource.close();
        eventSource = null;
    }

    connectionStatus.set('disconnected');
}

/** Reset connection (disconnect, clear buffer, reconnect) */
export function resetConnection(): void {
    disconnectSSE();
    clearBuffer();
    connectSSE();
}

/** Check if connected */
export function isConnected(): boolean {
    return eventSource !== null && eventSource.readyState === EventSource.OPEN;
}
