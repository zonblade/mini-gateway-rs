//! # Default Page Module
//! 
//! The default page module provides handlers for serving default content in specific scenarios,
//! such as error conditions (404/500) and security monitoring (TLS honeypot). These handlers
//! serve as fallback endpoints when regular routing rules don't match or when errors occur.
//! 
//! ## Module Structure
//! 
//! * `p_base`: Common base functionality for default page handlers
//! * `p404`: Handler for 404 Not Found responses
//! * `p500`: Handler for 500 Internal Server Error responses
//! * `tls_honeypot`: Security monitoring endpoint that logs suspicious TLS connections
//! 
//! ## Usage
//! 
//! These handlers are typically started as separate TCP listeners on dedicated ports,
//! and requests are routed to them when:
//! 
//! 1. No matching routing rules exist for a request (404 handler)
//! 2. An internal error occurs during request processing (500 handler)
//! 3. Suspicious TLS connection attempts are detected (TLS honeypot)
//! 
//! Each handler logs relevant information about the request and returns an appropriate
//! response to the client.

pub mod p_base;
pub mod p404;
pub mod p500;
pub mod tls_honeypot;