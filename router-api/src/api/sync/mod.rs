//! # Synchronization API Module
//!
//! This module provides endpoints for synchronizing configuration and state between
//! the central router API and distributed gateway and proxy nodes. It enables consistent
//! configuration across the entire system and provides mechanisms for node registration,
//! heartbeat monitoring, and status reporting.
//!
//! ## Features
//!
//! The Synchronization API is designed to provide:
//!
//! - **Node Registration**: Gateway and proxy nodes register with the central API
//! - **Configuration Distribution**: Push configuration updates to nodes
//! - **Health Monitoring**: Track node status and health through heartbeats
//! - **State Reconciliation**: Ensure system-wide consistency
//! - **Automatic Recovery**: Detect and recover from node failures
//!
//! ## Endpoints (Planned)
//!
//! - `POST /api/v1/sync/register` - Register a new gateway or proxy node
//! - `GET /api/v1/sync/config/{node_id}` - Get configuration for a specific node
//! - `POST /api/v1/sync/heartbeat` - Report node status and receive commands
//! - `GET /api/v1/sync/nodes` - List all registered nodes and their status
//! - `POST /api/v1/sync/commands/{node_id}` - Send commands to a specific node
//!
//! ## Node Lifecycle
//!
//! 1. **Registration**: Nodes register with the central API on startup
//! 2. **Configuration**: Nodes receive their initial configuration
//! 3. **Heartbeat**: Nodes periodically send heartbeats with status information
//! 4. **Updates**: Configuration changes are pushed to affected nodes
//! 5. **Deregistration**: Nodes gracefully deregister when shutting down
//!
//! ## Reliability Mechanisms
//!
//! - Automatic re-registration of nodes after connection failures
//! - Configurable heartbeat intervals with failure detection
//! - Stateful recovery after node restarts
//! - Versioned configuration to prevent inconsistencies
mod gateway_node;
mod gateway_node_queries;
mod proxy_node;
mod proxy_node_queries;

pub mod gateway_node_tcp;
pub mod proxy_node_tcp;

use actix_web::web;
use serde::{Deserialize, Serialize};

use super::users::{JwtAuth, RoleAuth};

#[derive(Debug, Serialize, Deserialize)]
pub struct HTTPCResponse {
    pub status: String,
    pub message: String,
}

/// Configure synchronization API routes
///
/// This function will set up the routes for synchronization endpoints when implemented.
/// Currently a placeholder as the synchronization functionality has not yet been fully implemented.
///
/// # Arguments
///
/// * `cfg` - A mutable reference to the service configuration
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/sync")
            .wrap(JwtAuth::new())
            .wrap(RoleAuth::staff())
            .service(gateway_node::gateway)
            .service(proxy_node::gateway),
    );
}
