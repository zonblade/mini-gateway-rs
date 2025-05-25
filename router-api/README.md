# Router API Documentation

The Router API provides a RESTful interface for managing and configuring the mini-gateway-rs routing system. This API allows for managing users, proxies, proxy domains, gateway nodes, gateways, and monitoring service statistics.

## Data Structure Relationships

The Router API is designed around the following core components and their relationships:

1. **Proxy**: The base configuration that handles listening on specific network addresses and forwarding traffic to target addresses. A Proxy represents a basic network endpoint listener.

2. **ProxyDomain**: Extends a Proxy by adding TLS configuration for specific domains. This allows a single Proxy to handle multiple domains with different TLS certificates and configurations. Each ProxyDomain is associated with exactly one Proxy and can optionally be linked to a GatewayNode for routing.

3. **GatewayNode**: Extends a Proxy by providing alternative routing targets. Each GatewayNode is associated with one Proxy and has a priority value that determines processing order (higher values = higher priority).

4. **Gateway**: Defines specific routing rules for a GatewayNode using pattern matching. Each Gateway is associated with one GatewayNode.

```
┌────────┐       ┌─────────────┐
│        │       │             │
│ Proxy  │───────┤ ProxyDomain │
│        │       │             │
└────┬───┘       └─────────────┘
     │
     │
     │           ┌────────────┐       ┌─────────┐
     └───────────┤            │       │         │
                 │ GatewayNode│───────┤ Gateway │
                 │            │       │         │
                 └────────────┘       └─────────┘
```

## Table of Contents

- [Authentication](#authentication)
- [User Management](#user-management)
  - [Implementation Notes](#implementation-notes)
  - [Get All Users](#get-all-users)
  - [Get User by ID](#get-user-by-id)
  - [Create User](#create-user)
  - [Update User](#update-user)
  - [Delete User](#delete-user)
- [Settings Management](#settings-management)
  - [Proxy Management](#proxy-management)
  - [Gateway Node Management](#gateway-node-management)
  - [Gateway Management](#gateway-management)
  - [Proxy Domain Management](#proxy-domain-management)
- [Synchronization](#synchronization)
- [Proxy Node Sync](#proxy-node-sync)
- [Gateway Node Sync](#gateway-node-sync)
- [Statistics](#statistics)
  - [Statistics Endpoints](#statistics-endpoints)
    - [Get Default Statistics](#get-default-statistics)
    - [Get Statistics by Status Code](#get-statistics-by-status-code)
    - [Get Bytes Statistics](#get-bytes-statistics)

## Authentication

All API endpoints (except the login endpoint) require JWT authentication. Include the JWT token in the `Authorization` header of each request using the Bearer scheme.

### Login

Authenticates a user and returns a JWT token for subsequent API requests.

**Endpoint:** `POST /api/v1/users/login`

**Request:**

| Field    | Type   | Description                 | Required |
|----------|--------|-----------------------------|----------|
| username | string | User's login username       | Yes      |
| password | string | User's password (plaintext) | Yes      |

**Response:**

| Field     | Type   | Description                        |
|-----------|--------|------------------------------------|
| success   | boolean| Whether login was successful       |
| token     | string | JWT token for authentication       |
| user_id   | string | UUID of the authenticated user     |
| username  | string | Username of the authenticated user |
| role      | string | User role (admin, staff, user)     |
| message   | string | Status message                     |

**Example Request:**
```json
{
  "username": "admin",
  "password": "adminpassword"
}
```

**Example Successful Response:**
```json
{
  "success": true,
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "user_id": "a1b2c3d4-e5f6-7890-abcd-1234567890ab",
  "username": "admin",
  "role": "admin",
  "message": "Login successful"
}
```

**Example Failed Response:**
```json
{
  "success": false,
  "token": null,
  "user_id": null,
  "username": null,
  "role": null,
  "message": "Invalid username or password"
}
```

## User Management

### Implementation Notes

The User Management API does not provide server-side pagination or search functionality. Client applications should implement these features on the frontend using the complete list of users returned by the API. This approach is suitable for deployments with a moderate number of users.

For larger deployments, consider implementing custom filtering by:
- Retrieving all users with `GET /api/v1/users`
- Performing client-side filtering, sorting, and pagination

### Get All Users

Retrieves a list of all users (admin only).

**Endpoint:** `GET /api/v1/users/admin`

**Request:** No parameters required.

**Response:**

| Field       | Type       | Description                   |
|-------------|------------|-------------------------------|
| id          | string     | User's unique UUID            |
| username    | string     | User's login username         |
| email       | string     | User's email address          |
| role        | string     | User's role                   |
| created_at  | string     | Timestamp of account creation |
| updated_at  | string     | Timestamp of last update      |

**Example Response:**
```json
[
  {
    "id": "a1b2c3d4-e5f6-7890-abcd-1234567890ab",
    "username": "admin",
    "email": "admin@example.com",
    "role": "admin",
    "created_at": "2023-01-01T12:00:00Z",
    "updated_at": "2023-01-01T12:00:00Z"
  },
  {
    "id": "b2c3d4e5-f6a7-8901-bcde-2345678901bc",
    "username": "staffuser",
    "email": "staff@example.com",
    "role": "staff",
    "created_at": "2023-01-02T12:00:00Z",
    "updated_at": "2023-01-02T12:00:00Z"
  }
]
```

### Get User by ID

Retrieves a specific user by their ID.

**Endpoint:** `GET /api/v1/users/{user_id}`

**Path Parameters:**

| Parameter | Description              |
|-----------|--------------------------|
| user_id   | UUID of the user to get  |

**Response:** Same as in Get All Users endpoint.

**Example Response:**
```json
{
  "id": "a1b2c3d4-e5f6-7890-abcd-1234567890ab",
  "username": "admin",
  "email": "admin@example.com",
  "role": "admin",
  "created_at": "2023-01-01T12:00:00Z",
  "updated_at": "2023-01-01T12:00:00Z"
}
```

### Create User

Creates a new user (admin only).

**Endpoint:** `POST /api/v1/users/admin`

**Request:**

| Field    | Type   | Description               | Required |
|----------|--------|---------------------------|----------|
| username | string | User's login username     | Yes      |
| email    | string | User's email address      | Yes      |
| password | string | User's initial password   | Yes      |
| role     | string | User's role (admin, staff, user) | No - defaults to "user" |

**Response:** Returns the created user object (same structure as Get User).

**Example Request:**
```json
{
  "username": "newstaff",
  "email": "newstaff@example.com",
  "password": "password123",
  "role": "staff"
}
```

**Example Response:**
```json
{
  "id": "c3d4e5f6-a7b8-9012-cdef-3456789012de",
  "username": "newstaff",
  "email": "newstaff@example.com",
  "role": "staff",
  "created_at": "2023-03-15T14:30:45Z",
  "updated_at": "2023-03-15T14:30:45Z"
}
```

### Update User

Updates an existing user. Users can only update their own profile unless they are admins.

**Endpoint:** `PUT /api/v1/users/{user_id}`

**Path Parameters:**

| Parameter | Description                |
|-----------|----------------------------|
| user_id   | UUID of the user to update |

**Request:**

| Field    | Type   | Description               | Required |
|----------|--------|---------------------------|----------|
| username | string | New username              | No       |
| email    | string | New email address         | No       |
| password | string | New password              | No       |
| role     | string | New role (admin only)     | No       |

**Response:** Returns the updated user object (same structure as Get User).

**Example Request:**
```json
{
  "email": "updated.email@example.com"
}
```

**Example Response:**
```json
{
  "id": "b2c3d4e5-f6a7-8901-bcde-2345678901bc",
  "username": "staffuser",
  "email": "updated.email@example.com",
  "role": "staff",
  "created_at": "2023-01-02T12:00:00Z",
  "updated_at": "2023-03-15T15:45:22Z"
}
```

### Delete User

Deletes a user. Users can only delete their own account unless they are admins.

**Endpoint:** `DELETE /api/v1/users/{user_id}`

**Path Parameters:**

| Parameter | Description                |
|-----------|----------------------------|
| user_id   | UUID of the user to delete |

**Response:**

| Field   | Type   | Description               |
|---------|--------|---------------------------|
| message | string | Success confirmation message |

**Example Response:**
```json
{
  "message": "User successfully deleted"
}
```

## Settings Management

All settings endpoints require staff or admin role to access.

### Proxy Management

#### List All Proxies

Retrieves a list of all configured proxies with their associated domains (simplified).

**Endpoint:** `GET /api/v1/settings/proxies`

**Response:** Returns an array of objects, each containing a proxy and its domains.

| Field           | Type    | Description                                |
|---------------|---------|--------------------------------------------|
| proxy         | object  | The proxy configuration object             |
| domains       | array   | Array of simplified domain objects (ID, SNI, TLS status only) |
| warning       | string  | Optional warning message if domain fetch failed |

**Example Response:**
```json
[
  {
    "proxy": {
      "id": "proxy-1",
      "title": "Main API Proxy",
      "addr_listen": "0.0.0.0:443",
      "addr_target": "127.0.0.1:8080",
      "high_speed": true,
      "high_speed_addr": "10.0.0.1:8081",
      "high_speed_gwid": "gwnode-2"
    },
    "domains": [
      {
        "id": "domain-1", 
        "sni": "api.example.com",
        "tls": true
      }
    ]
  },
  {
    "proxy": {
      "id": "proxy-2",
      "title": "Web Frontend",
      "addr_listen": "0.0.0.0:80",
      "addr_target": "127.0.0.1:3000",
      "high_speed": false,
      "high_speed_addr": null,
      "high_speed_gwid": null
    },
    "domains": []
  }
]
```

#### Get Proxy by ID

Retrieves a specific proxy by its ID, along with all its associated domains.

**Endpoint:** `GET /api/v1/settings/proxy/{id}`

**Path Parameters:**

| Parameter | Description             |
|-----------|-------------------------|
| id        | ID of the proxy to get  |

**Response:** Returns an object containing the proxy and its domains.

| Field           | Type    | Description                                |
|---------------|---------|--------------------------------------------|
| proxy         | object  | The proxy configuration object             |
| domains       | array   | Array of associated domain objects         |
| warning       | string  | Optional warning message if domain fetch failed |

**Example Response:**
```json
{
  "proxy": {
    "id": "proxy-1",
    "title": "Main API Proxy",
    "addr_listen": "0.0.0.0:443",
    "addr_target": "127.0.0.1:8080",
    "high_speed": true,
    "high_speed_addr": "10.0.0.1:8081",
    "high_speed_gwid": "gwnode-5"
  },
  "domains": [
    {
      "id": "domain-1",
      "proxy_id": "proxy-1",
      "gwnode_id": "gwnode-1",
      "tls": true,
      "tls_pem": "-----BEGIN CERTIFICATE-----\nMIIE...",
      "tls_key": "-----BEGIN PRIVATE KEY-----\nMIIE...",
      "sni": "api.example.com"
    },
    {
      "id": "domain-2",
      "proxy_id": "proxy-1",
      "gwnode_id": null,
      "tls": true,
      "tls_pem": "-----BEGIN CERTIFICATE-----\nMIIE...",
      "tls_key": "-----BEGIN PRIVATE KEY-----\nMIIE...",
      "sni": "admin.example.com"
    }
  ]
}
```

#### Create or Update Proxy

Creates a new proxy or updates an existing one. This endpoint supports submitting both the proxy configuration and its associated domains in a single request.

**Endpoint:** `POST /api/v1/settings/proxy`

**Request Format:**

| Field          | Type    | Description                                | Required |
|----------------|---------|--------------------------------------------|----------|
| proxy          | object  | The proxy configuration object             | Yes      |
| domains        | array   | Array of proxy domain objects              | No       |

**Proxy Object Fields:**

| Field          | Type    | Description                                | Required |
|----------------|---------|--------------------------------------------|----------|
| id             | string  | Unique ID (empty for new)                  | No       |
| title          | string  | Human-readable proxy name                  | Yes      |
| addr_listen    | string  | Listen address (format: "ip:port")         | Yes      |
| addr_target    | string  | Target address (automatically generated)   | No       |
| high_speed     | boolean | Whether high speed mode is enabled         | No       |
| high_speed_addr| string  | Specific address to use for high speed mode| No       |
| high_speed_gwid| string  | Gateway node ID to use for high speed mode | No       |

**Note:** When `high_speed_gwid` is provided, the system automatically uses the gateway node's alternative target as the `high_speed_addr`. Clients can set either `high_speed_addr` directly or specify a `high_speed_gwid` to have the address derived from a gateway node. When both are provided, the gateway node ID takes precedence.

**Response:** Returns the saved proxy object along with its associated domains.

**Example Request:**
```json
{
  "proxy": {
    "title": "New API Proxy with Domains",
    "addr_listen": "0.0.0.0:8443",
    "high_speed": true,
    "high_speed_gwid": "gwnode-3"
  },
  "domains": [
    {
      "tls": true,
      "tls_pem": "-----BEGIN CERTIFICATE-----\nMIIE...",
      "tls_key": "-----BEGIN PRIVATE KEY-----\nMIIE...",
      "sni": "api.example.com",
      "gwnode_id": "gwnode-1"
    },
    {
      "tls": true,
      "tls_pem": "-----BEGIN CERTIFICATE-----\nMIIE...",
      "tls_key": "-----BEGIN PRIVATE KEY-----\nMIIE...",
      "sni": "admin.example.com"
    }
  ]
}
```

**Example Response:**
```json
{
  "proxy": {
    "id": "new-proxy-id",
    "title": "New API Proxy with Domains",
    "addr_listen": "0.0.0.0:8443",
    "addr_target": "127.0.0.1:45023",
    "high_speed": true,
    "high_speed_addr": "http://backup-server.internal:8080",
    "high_speed_gwid": "gwnode-3"
  },
  "domains": [
    {
      "id": "domain-1",
      "proxy_id": "new-proxy-id",
      "gwnode_id": "gwnode-1",
      "tls": true,
      "tls_pem": "-----BEGIN CERTIFICATE-----\nMIIE...",
      "tls_key": "-----BEGIN PRIVATE KEY-----\nMIIE...",
      "sni": "api.example.com"
    },
    {
      "id": "domain-2",
      "proxy_id": "new-proxy-id",
      "gwnode_id": null,
      "tls": true,
      "tls_pem": "-----BEGIN CERTIFICATE-----\nMIIE...",
      "tls_key": "-----BEGIN PRIVATE KEY-----\nMIIE...",
      "sni": "admin.example.com"
    }
  ]
}
```

#### Delete Proxy

Deletes a proxy by its ID. When a proxy is deleted, the following actions occur:
1. All associated proxy domains are automatically deleted
2. Any gateway nodes associated with the proxy are unbound (but not deleted)

**Endpoint:** `DELETE /api/v1/settings/proxy/{id}`

**Path Parameters:**

| Parameter | Description               |
|-----------|---------------------------|
| id        | ID of the proxy to delete |

**Response:**

| Field   | Type   | Description              |
|---------|--------|--------------------------|
| message | string | Success message          |

**Example Response:**
```json
{
  "message": "Proxy with ID proxy-123 deleted. 2 proxy domains were removed. 1 gateway nodes were unbound."
}
```

### Gateway Node Management

#### List All Gateway Nodes

Retrieves a list of all gateway nodes.

**Endpoint:** `GET /api/v1/settings/gwnode/list`

**Response:** Returns an array of gateway node objects.

| Field      | Type   | Description                         |
|------------|--------|-------------------------------------|
| id         | string | Unique node identifier              |
| proxy_id   | string | ID of the proxy this node uses      |
| title      | string | Human-readable name for the node    |
| alt_target | string | Alternative target URL for routing  |
| priority   | number | Processing priority (default: 100, higher values = higher priority) |

**Example Response:**
```json
[
  {
    "id": "7f9c24e5-1315-43a7-9f31-6eb9772cb46a",
    "proxy_id": "550e8400-e29b-41d4-a716-446655440000",
    "title": "API Backup Gateway",
    "alt_target": "http://backup-server:8080",
    "priority": 150
  },
  {
    "id": "8d4e6f7a-2c1b-43e5-9f87-12ab34cd56ef",
    "proxy_id": "550e8400-e29b-41d4-a716-446655440000",
    "title": "Product Service Gateway",
    "alt_target": "http://alternate-server:3000",
    "priority": 100
  }
]
```

#### List Gateway Nodes by Proxy ID

Retrieves all gateway nodes associated with a specific proxy.

**Endpoint:** `GET /api/v1/settings/gwnode/list/{proxy_id}`

**Path Parameters:**

| Parameter | Description                             |
|-----------|-----------------------------------------|
| proxy_id  | ID of the proxy to list nodes for       |

**Response:** Returns an array of gateway node objects (same structure as List All Gateway Nodes).

#### Get Gateway Node by ID

Retrieves a specific gateway node by its ID.

**Endpoint:** `GET /api/v1/settings/gwnode/{id}`

**Path Parameters:**

| Parameter | Description                  |
|-----------|------------------------------|
| id        | ID of the gateway node to get|

**Response:** Returns a gateway node object (same structure as in List All Gateway Nodes).

#### Create or Update Gateway Node

Creates a new gateway node or updates an existing one.

**Endpoint:** `POST /api/v1/settings/gwnode/set`

**Request:**

| Field      | Type   | Description                           | Required |
|------------|--------|---------------------------------------|----------|
| id         | string | Unique ID (empty for new)             | No       |
| proxy_id   | string | ID of the proxy this node uses        | Yes      |
| title      | string | Human-readable name for the node      | No       |
| alt_target | string | Alternative target URL for routing    | Yes      |
| priority   | number | Processing priority (default: 100)    | No       |

**Response:** Returns the saved gateway node object.

**Example Request:**
```json
{
  "proxy_id": "550e8400-e29b-41d4-a716-446655440000",
  "title": "API Backup Gateway",
  "alt_target": "http://backup-server:8080",
  "priority": 150
}
```

**Example Response:**
```json
{
  "id": "7f9c24e5-1315-43a7-9f31-6eb9772cb46a",
  "proxy_id": "550e8400-e29b-41d4-a716-446655440000",
  "title": "API Backup Gateway",
  "alt_target": "http://backup-server:8080",
  "priority": 150
}
```

#### Delete Gateway Node

Deletes a gateway node and all associated gateways.

**Endpoint:** `POST /api/v1/settings/gwnode/delete`

**Request:**

| Field | Type   | Description                     | Required |
|-------|--------|---------------------------------|----------|
| id    | string | ID of the gateway node to delete| Yes      |

**Response:**

| Field   | Type   | Description              |
|---------|--------|--------------------------|
| message | string | Success message          |

**Example Request:**
```json
{
  "id": "7f9c24e5-1315-43a7-9f31-6eb9772cb46a"
}
```

**Example Response:**
```json
{
  "message": "Gateway node deleted successfully along with 3 associated gateways"
}
```

### Gateway Management

#### List All Gateways

Retrieves a list of all gateway routing rules.

**Endpoint:** `GET /api/v1/settings/gateway/list`

**Response:** Returns an array of gateway objects, ordered by priority.

| Field     | Type   | Description                               |
|-----------|--------|-------------------------------------------|
| id        | string | Unique gateway identifier                 |
| gwnode_id | string | ID of the gateway node this gateway uses  |
| pattern   | string | URL matching pattern                      |
| target    | string | Target URL for matched requests           |
| priority  | number | Priority level (lower = higher priority)  |

**Example Response:**
```json
[
  {
    "id": "gw-1",
    "gwnode_id": "gwnode-1",
    "pattern": "/api/users/*",
    "target": "http://user-service:8080",
    "priority": 10
  },
  {
    "id": "gw-2",
    "gwnode_id": "gwnode-1",
    "pattern": "/api/products/*",
    "target": "http://product-service:8080",
    "priority": 20
  }
]
```

#### List Gateways by Gateway Node ID

Retrieves all gateway routing rules associated with a specific gateway node.

**Endpoint:** `GET /api/v1/settings/gateway/list/{gwnode_id}`

**Path Parameters:**

| Parameter | Description                        |
|-----------|------------------------------------|
| gwnode_id | ID of the gateway node to list for |

**Response:** Returns an array of gateway objects (same structure as List All Gateways).

#### Get Gateway by ID

Retrieves a specific gateway routing rule by its ID.

**Endpoint:** `GET /api/v1/settings/gateway/{id}`

**Path Parameters:**

| Parameter | Description              |
|-----------|--------------------------|
| id        | ID of the gateway to get |

**Response:** Returns a gateway object (same structure as in List All Gateways).

#### Create or Update Gateway

Creates a new gateway routing rule or updates an existing one.

**Endpoint:** `POST /api/v1/settings/gateway/set`

**Request:**

| Field     | Type   | Description                               | Required |
|-----------|--------|-------------------------------------------|----------|
| id        | string | Unique ID (empty for new)                 | No       |
| gwnode_id | string | ID of the gateway node this gateway uses  | Yes      |
| pattern   | string | URL matching pattern                      | Yes      |
| target    | string | Target URL for matched requests           | Yes      |
| priority  | number | Priority level (lower = higher priority)  | Yes      |

**Response:** Returns the saved gateway object.

**Example Request:**
```json
{
  "gwnode_id": "gwnode-1",
  "pattern": "^/api/auth/(.*)$",
  "target": "/v2/auth/$1",
  "priority": 5
}
```

#### Delete Gateway

Deletes a gateway routing rule.

**Endpoint:** `POST /api/v1/settings/gateway/delete`

**Request:**

| Field | Type   | Description                  | Required |
|-------|--------|------------------------------|----------|
| id    | string | ID of the gateway to delete  | Yes      |

**Response:**

| Field   | Type   | Description     |
|---------|--------|-----------------|
| message | string | Success message |

**Example Request:**
```json
{
  "id": "gw-2"
}
```

**Example Response:**
```json
{
  "message": "Gateway deleted successfully"
}
```

### Proxy Domain Management

**Note:** Proxy domain management is now integrated with the proxy management endpoints. When creating or updating a proxy using the `POST /api/v1/settings/proxy` endpoint, you can include domain configurations in the same request. The standalone proxy domain endpoints are maintained for compatibility but new implementations should prefer using the combined proxy endpoints.

#### List All Proxy Domains

Retrieves a list of all proxy domains.

**Endpoint:** `GET /api/v1/settings/proxydomain/list`

**Response:** Returns an array of proxy domain objects.

| Field      | Type    | Description                                      |
|------------|---------|--------------------------------------------------|
| id         | string  | Unique proxy domain identifier                   |
| proxy_id   | string  | ID of the proxy this domain is associated with   |
| gwnode_id  | string  | ID of the gateway node for routing (optional)    |
| tls        | boolean | Whether TLS is enabled for this domain           |
| tls_pem    | string  | PEM-encoded certificate (null if not used)       |
| tls_key    | string  | Private key for certificate (null if not used)   |
| sni        | string  | Server Name Indication value (null if not used)  |

**Example Response:**
```json
[
  {
    "id": "domain-1",
    "proxy_id": "proxy-1",
    "gwnode_id": "gwnode-1",
    "tls": true,
    "tls_pem": "-----BEGIN CERTIFICATE-----\nMIIE...",
    "tls_key": "-----BEGIN PRIVATE KEY-----\nMIIE...",
    "sni": "api.example.com"
  },
  {
    "id": "domain-2",
    "proxy_id": "proxy-1",
    "gwnode_id": null,
    "tls": true,
    "tls_pem": "-----BEGIN CERTIFICATE-----\nMIIE...",
    "tls_key": "-----BEGIN PRIVATE KEY-----\nMIIE...",
    "sni": "admin.example.com"
  }
]
```

#### List Proxy Domains by Proxy ID

Retrieves all proxy domains associated with a specific proxy.

**Endpoint:** `GET /api/v1/settings/proxydomain/list/{proxy_id}`

**Path Parameters:**

| Parameter | Description                             |
|-----------|-----------------------------------------|
| proxy_id  | ID of the proxy to list domains for     |

**Response:** Returns an array of proxy domain objects (same structure as List All Proxy Domains).

#### List Proxy Domains by Gateway Node ID

Retrieves all proxy domains associated with a specific gateway node.

**Endpoint:** `GET /api/v1/settings/proxydomain/list/gwnode/{gwnode_id}`

**Path Parameters:**

| Parameter | Description                                  |
|-----------|----------------------------------------------|
| gwnode_id | ID of the gateway node to list domains for   |

**Response:** Returns an array of proxy domain objects (same structure as List All Proxy Domains).

#### Get Proxy Domain by ID

Retrieves a specific proxy domain by its ID.

**Endpoint:** `GET /api/v1/settings/proxydomain/{id}`

**Path Parameters:**

| Parameter | Description                     |
|-----------|---------------------------------|
| id        | ID of the proxy domain to get   |

**Response:** Returns a proxy domain object (same structure as in List All Proxy Domains).

#### Create or Update Proxy Domain

Creates a new proxy domain or updates an existing one.

**Endpoint:** `POST /api/v1/settings/proxydomain/set`

**Request:**

| Field      | Type    | Description                                  | Required |
|------------|---------|----------------------------------------------|----------|
| id         | string  | Unique ID (empty for new)                    | No       |
| proxy_id   | string  | ID of the proxy this domain uses             | Yes      |
| gwnode_id  | string  | ID of the gateway node for routing           | No       |
| tls        | boolean | Whether TLS is enabled for this domain       | Yes      |
| tls_pem    | string  | PEM-encoded certificate content              | No       |
| tls_key    | string  | Private key content                          | No       |
| sni        | string  | Server Name Indication value for TLS         | No       |

**Response:** Returns the saved proxy domain object.

**Example Request:**
```json
{
  "proxy_id": "proxy-1",
  "gwnode_id": "gwnode-3",
  "tls": true,
  "tls_pem": "-----BEGIN CERTIFICATE-----\nMIIE...",
  "tls_key": "-----BEGIN PRIVATE KEY-----\nMIIE...",
  "sni": "new.example.com"
}
```

**Example Response:**
```json
{
  "id": "domain-3",
  "proxy_id": "proxy-1",
  "gwnode_id": "gwnode-3",
  "tls": true,
  "tls_pem": "-----BEGIN CERTIFICATE-----\nMIIE...",
  "tls_key": "-----BEGIN PRIVATE KEY-----\nMIIE...",
  "sni": "new.example.com"
}
```

#### Delete Proxy Domain

Deletes a proxy domain by its ID.

**Endpoint:** `POST /api/v1/settings/proxydomain/delete`

**Request:**

| Field | Type   | Description                      | Required |
|-------|--------|----------------------------------|----------|
| id    | string | ID of the proxy domain to delete | Yes      |

**Response:**

| Field   | Type   | Description       |
|---------|--------|-------------------|
| message | string | Success message   |

**Example Request:**
```json
{
  "id": "domain-2"
}
```

**Example Response:**
```json
{
  "message": "Proxy domain deleted successfully"
}
```

## Synchronization

The synchronization endpoints allow you to sync the configured proxy, proxy domains, and gateway nodes with the registry service. These operations ensure that all components of the mini-gateway-rs system are using consistent configuration data.

### Proxy Node Sync

Synchronizes all configured proxy nodes and their associated proxy domains to the registry service.

**Endpoint:** `POST /api/v1/sync/node`

**Request:** No parameters required.

**Response:** Returns the result of the synchronization.

| Field   | Type    | Description                            |
|---------|---------|----------------------------------------|
| status  | string  | Status of the operation ("success" or "error") |
| message | string  | Descriptive message about the operation |

**Example Response:**
```json
{
  "status": "success",
  "message": "Successfully synchronized 2 proxy nodes with 3 proxy domains"
}
```

### Gateway Node Sync

Synchronizes all configured gateway nodes to the registry service.

**Endpoint:** `POST /api/v1/sync/gateway`

**Request:** No parameters required.

**Response:** Returns the result of the synchronization.

| Field   | Type    | Description                            |
|---------|---------|----------------------------------------|
| status  | string  | Status of the operation ("success" or "error") |
| message | string  | Descriptive message about the operation |

**Example Response:**
```json
{
  "status": "success",
  "message": "Successfully synchronized 2 gateway nodes"
}
```

## Statistics

The Statistics API provides endpoints for monitoring and reporting gateway and proxy statistics.

### Statistics Endpoints

#### Get Default Statistics

Retrieves statistics for request and response counts over time.

**Endpoint:** `GET /api/v1/statistics/default`

**Query Parameters:**

| Parameter | Type   | Description                                                 | Required |
|-----------|--------|-------------------------------------------------------------|----------|
| target    | string | Data source: "domain" (default) or "proxy"                   | No       |

**Response:** Returns an array of time series data points collected in 15-second intervals over the last 120 minutes.

| Field       | Type       | Description                                            |
|-------------|------------|--------------------------------------------------------|
| date_time   | string     | ISO-8601 formatted timestamp for the data point        |
| value       | number     | Failed/unmatched requests (req_count - res_count)      |
| high        | number     | Response count                                         |
| low         | number     | Request count                                          |

**Example Response:**
```json
[
  {
    "date_time": "2023-04-15T10:00:00Z",
    "value": 2,
    "high": 48,
    "low": 50
  },
  {
    "date_time": "2023-04-15T10:00:15Z",
    "value": 0,
    "high": 52,
    "low": 52
  }
]
```

#### Get Statistics by Status Code

Retrieves statistics filtered by HTTP status code.

**Endpoint:** `GET /api/v1/statistics/status/{status}`

**Path Parameters:**

| Parameter | Description                  |
|-----------|------------------------------|
| status    | HTTP status code to filter by|

**Query Parameters:**

| Parameter | Type   | Description                                                 | Required |
|-----------|--------|-------------------------------------------------------------|----------|
| target    | string | Data source: "domain" (default) or "proxy"                   | No       |

**Response:** Returns an array of time series data points collected in 15-second intervals over the last 120 minutes.

| Field       | Type       | Description                                            |
|-------------|------------|--------------------------------------------------------|
| date_time   | string     | ISO-8601 formatted timestamp for the data point        |
| value       | number     | how many status counted in range     |
| high        | number     | not used                   |
| low         | number     | not used                   |

**Example Response:**
```json
[
  {
    "date_time": "2023-04-15T10:00:00Z",
    "value": 5,
    "high": 120,
    "low": 45
  },
  {
    "date_time": "2023-04-15T10:00:15Z",
    "value": 3,
    "high": 95,
    "low": 30
  }
]
```

#### Get Bytes Statistics

Retrieves statistics about bytes transferred.

**Endpoint:** `GET /api/v1/statistics/bytes`

**Query Parameters:**

| Parameter | Type   | Description                                                 | Required |
|-----------|--------|-------------------------------------------------------------|----------|
| target    | string | Data source: "domain" (default) or "proxy"                   | No       |

**Response:** Returns an array of time series data points collected in 15-second intervals over the last 120 minutes.

| Field       | Type       | Description                                            |
|-------------|------------|--------------------------------------------------------|
| date_time   | string     | ISO-8601 formatted timestamp for the data point        |
| value       | number     | Average bytes transferred in the 15-second interval    |
| high        | number     | Highest one-second average bytes transferred           |
| low         | number     | Lowest one-second average bytes transferred            |

**Example Response:**
```json
[
  {
    "date_time": "2023-04-15T10:00:00Z",
    "value": 8456,
    "high": 12580,
    "low": 4532
  },
  {
    "date_time": "2023-04-15T10:00:15Z",
    "value": 7245,
    "high": 10485,
    "low": 5124
  }
]
```

## Auto-Configuration

The Auto-Configuration API provides endpoints for bulk importing and exporting gateway configurations using YAML files. This allows for easier setup, backup, and migration of configuration across environments.

### Upload Configuration

Uploads a YAML configuration file and applies it to the system by creating the corresponding proxy, domain, gateway node, and gateway configurations.

**Endpoint:** `POST /api/v1/auto-config`

**Request:** The request body should be a YAML document conforming to the following structure:

```yaml
proxy:
  - name: "proxy1"
    listen: "127.0.0.1:8080"
    domains:
      - domain: "example.com"
        tls: false
        tls_cert: |
          -----BEGIN CERTIFICATE-----
          cert
          -----END CERTIFICATE-----
        tls_key: |
          -----BEGIN PRIVATE KEY-----
          key
          -----END PRIVATE KEY-----
    highspeed:
      enabled: true
      target: "gateway1"
    gateway:
      - name: "gateway1"
        domain: "example.com"
        target: "127.0.0.1:8080"
        path:
          - priority: 1
            pattern: "^(.*)$"
            target: "/$1"
```

The configuration follows this structure:
- `proxy`: Array of proxy configurations
  - `name`: Human-readable name for the proxy
  - `listen`: Address where the proxy listens (format: "ip:port")
  - `domains`: Array of domain configurations
    - `domain`: Domain name (SNI value)
    - `tls`: Whether TLS is enabled
    - `tls_cert`: TLS certificate content (optional)
    - `tls_key`: TLS private key content (optional)
  - `highspeed`: High-speed mode configuration (optional)
    - `enabled`: Whether high-speed mode is enabled
    - `target`: Target gateway name for high-speed mode
  - `gateway`: Array of gateway configurations
    - `name`: Human-readable name for the gateway
    - `domain`: Domain associated with this gateway
    - `target`: Target address for the gateway node
    - `path`: Array of path configurations
      - `priority`: Priority level (lower numbers = higher priority)
      - `pattern`: URL matching pattern
      - `target`: Target URL where matching requests should be routed

**Response:**

| Field    | Type    | Description                                 |
|----------|---------|---------------------------------------------|
| success  | boolean | Whether the operation was successful        |
| created  | object  | Summary of created resources                |

**Example Response:**
```json
{
  "success": true,
  "created": {
    "proxies": 1,
    "domains": 1,
    "gwnodes": 1,
    "gateways": 1
  }
}
```

### Download Configuration

Downloads the current configuration as a YAML file. This exports all proxy, domain, gateway node, and gateway configurations into a format that can be uploaded back through the upload endpoint.

**Endpoint:** `GET /api/v1/auto-config`

**Response:** Returns a YAML document containing the full configuration in the same format as described in the Upload Configuration section. The response includes a `Content-Disposition` header set to `attachment; filename="gateway-config.yaml"` to prompt the browser to download the file.