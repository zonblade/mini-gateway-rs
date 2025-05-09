/**
 * Gateway type definitions
 */

/**
 * Represents a gateway routing rule in the system
 */
export interface Gateway {
    /** Unique identifier for the gateway */
    id: string;
    /** Reference to the gateway node ID this gateway is associated with */
    gwnode_id: string;
    /** Pattern for URL matching (e.g., "/api/users/*", "^/users/[0-9]+") */
    pattern: string;
    /** Target URL where matching requests should be routed */
    target: string;
    /** Priority level (lower number = higher priority) */
    priority: number;
    /** Optional domain ID this gateway rule is associated with */
    domain_id?: string;
}

/**
 * Request to create a new gateway
 */
export interface CreateGatewayRequest {
    /** Optional ID (empty for new gateways) */
    id?: string; // Optional for creation, server will generate if empty
    /** Reference to the gateway node ID this gateway is associated with */
    gwnode_id: string;
    /** Pattern for URL matching */
    pattern: string;
    /** Target URL where matching requests should be routed */
    target: string;
    /** Priority level (lower number = higher priority) */
    priority: number;
    /** Optional domain ID this gateway rule is associated with */
    domain_id?: string; // Optional for creation, server will generate if empty
}

/**
 * Request to update an existing gateway
 */
export interface UpdateGatewayRequest {
    /** ID of the gateway to update */
    id: string;
    /** Reference to the gateway node ID this gateway is associated with */
    gwnode_id: string;
    /** Pattern for URL matching */
    pattern: string;
    /** Target URL where matching requests should be routed */
    target: string;
    /** Priority level (lower number = higher priority) */
    priority: number;
    /** Optional domain ID this gateway rule is associated with */
    domain_id?: string; // Optional for creation, server will generate if empty
}

/**
 * Response for deleting a gateway
 */
export interface DeleteGatewayResponse {
    /** Message indicating the result of the delete operation */
    message: string;
}