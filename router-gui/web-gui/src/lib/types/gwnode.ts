// Define GwNode related types

export interface GwNode {
    id: string;
    proxy_id: string;
    title: string;
    alt_target: string;
    source?: string; // Kept for backward compatibility with API
    proxyTitle?: string; // Additional field for UI display purposes
    domain_id?: string; // ID of the selected domain
    domain_name?: string; // Name of the selected domain for UI display
}

// Request types for API calls
export interface CreateGwNodeRequest {
    id?: string; // Optional for creation, server will generate if empty
    proxy_id: string;
    title: string;
    alt_target: string;
    source?: string; // Deprecated but still needed for API compatibility
    domain_id?: string; // Add domain ID support to the API request
}

export interface UpdateGwNodeRequest {
    id: string;
    proxy_id: string;
    title: string;
    alt_target: string;
    source?: string; // Deprecated but still needed for API compatibility
    domain_id?: string; // Add domain ID support to the API request
}

export interface DeleteGwNodeRequest {
    id: string;
}