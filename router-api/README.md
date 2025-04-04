# Router API Documentation

The Router API provides a RESTful interface for managing and configuring the mini-gateway-rs routing system. This API allows for managing users, proxies, gateway nodes, gateways, and monitoring service statistics.

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
- [Synchronization](#synchronization)
  - [Proxy Node Sync](#proxy-node-sync)
  - [Gateway Node Sync](#gateway-node-sync)

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

Retrieves a list of all configured proxies.

**Endpoint:** `GET /api/v1/settings/proxies`

**Response:** Returns an array of proxy objects.

| Field       | Type    | Description                                |
|-------------|---------|--------------------------------------------|
| id          | string  | Unique proxy identifier                    |
| title       | string  | Human-readable proxy name                  |
| addr_listen | string  | Listen address (format: "ip:port")         |
| addr_target | string  | Target address (format: "ip:port")         |
| tls         | boolean | Whether TLS is enabled                     |
| tls_pem     | string  | PEM-encoded certificate (null if not used) |
| tls_key     | string  | Private key for certificate (null if not used) |
| tls_autron  | boolean | Whether automatic TLS provisioning is enabled |
| sni         | string  | Server Name Indication value (null if not used) |

**Example Response:**
```json
[
  {
    "id": "proxy-1",
    "title": "Main API Proxy",
    "addr_listen": "0.0.0.0:443",
    "addr_target": "127.0.0.1:8080",
    "tls": true,
    "tls_pem": "-----BEGIN CERTIFICATE-----\nMIIE...",
    "tls_key": "-----BEGIN PRIVATE KEY-----\nMIIE...",
    "tls_autron": false,
    "sni": "api.example.com"
  },
  {
    "id": "proxy-2",
    "title": "Web Frontend",
    "addr_listen": "0.0.0.0:80",
    "addr_target": "127.0.0.1:3000",
    "tls": false,
    "tls_pem": null,
    "tls_key": null,
    "tls_autron": false,
    "sni": null
  }
]
```

#### Get Proxy by ID

Retrieves a specific proxy by its ID.

**Endpoint:** `GET /api/v1/settings/proxy/{id}`

**Path Parameters:**

| Parameter | Description             |
|-----------|-------------------------|
| id        | ID of the proxy to get  |

**Response:** Returns a proxy object (same structure as in List All Proxies).

#### Create or Update Proxy

Creates a new proxy or updates an existing one.

**Endpoint:** `POST /api/v1/settings/proxy`

**Request:**

| Field       | Type    | Description                                | Required |
|-------------|---------|--------------------------------------------|----------|
| id          | string  | Unique ID (empty for new)                  | No       |
| title       | string  | Human-readable proxy name                  | Yes      |
| addr_listen | string  | Listen address (format: "ip:port")         | Yes      |
| addr_target | string  | Target address (format: "ip:port")         | Yes      |
| tls         | boolean | Whether TLS is enabled                     | Yes      |
| tls_pem     | string  | PEM-encoded certificate content            | No       |
| tls_key     | string  | Private key content                        | No       |
| tls_autron  | boolean | Whether to use automatic TLS provisioning  | No       |
| sni         | string  | Server Name Indication value for TLS       | No       |

**Response:** Returns the saved proxy object.

**Example Request:**
```json
{
  "title": "New API Proxy",
  "addr_listen": "0.0.0.0:8443",
  "addr_target": "127.0.0.1:9090",
  "tls": true,
  "tls_pem": "-----BEGIN CERTIFICATE-----\nMIIE...",
  "tls_key": "-----BEGIN PRIVATE KEY-----\nMIIE...",
  "tls_autron": false,
  "sni": "api.newservice.com"
}
```

#### Delete Proxy

Deletes a proxy by its ID.

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
  "message": "Proxy successfully deleted"
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

**Example Response:**
```json
[
  {
    "id": "7f9c24e5-1315-43a7-9f31-6eb9772cb46a",
    "proxy_id": "550e8400-e29b-41d4-a716-446655440000",
    "title": "API Backup Gateway",
    "alt_target": "http://backup-server:8080"
  },
  {
    "id": "8d4e6f7a-2c1b-43e5-9f87-12ab34cd56ef",
    "proxy_id": "550e8400-e29b-41d4-a716-446655440000",
    "title": "Product Service Gateway",
    "alt_target": "http://alternate-server:3000"
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

**Response:** Returns the saved gateway node object.

**Example Request:**
```json
{
  "proxy_id": "550e8400-e29b-41d4-a716-446655440000",
  "title": "API Backup Gateway",
  "alt_target": "http://backup-server:8080"
}
```

**Example Response:**
```json
{
  "id": "7f9c24e5-1315-43a7-9f31-6eb9772cb46a",
  "proxy_id": "550e8400-e29b-41d4-a716-446655440000",
  "title": "API Backup Gateway",
  "alt_target": "http://backup-server:8080"
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

## Synchronization

The synchronization endpoints allow you to sync the configured proxy and gateway nodes with the registry service. These operations ensure that all components of the mini-gateway-rs system are using consistent configuration data.

### Proxy Node Sync

Synchronizes all configured proxy nodes to the registry service.

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
  "message": "Successfully synchronized 2 proxy nodes"
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

## Pattern Matching

The gateway supports various pattern matching techniques:

- **Exact path matching**: `/api/users`
- **Prefix matching with wildcard**: `/api/*`
- **Regex-like patterns**: `^/users/[0-9]+`

When using capture groups in patterns, you can reference them in the target using `$n` syntax (e.g., `$1`, `$2`):

- Pattern: `^/api/(.*)$`
- Target: `/v2/api/$1`
- Result: `/api/users` → `/v2/api/users`

## Priority System

Gateways are processed in order of priority, with lower numbers having higher precedence. When multiple patterns match an incoming request, the one with the lowest priority value is used.