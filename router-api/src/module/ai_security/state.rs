//! Global state management for AI Security module
//!
//! Uses OnceLock for lazy initialization and AtomicBool for fast flag checks.

use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::Sender,
    OnceLock,
};
use super::thread_xgboost::MlFeatureLog;

// === Global Flags (fast atomic checks) ===
static AI_ENABLED: AtomicBool = AtomicBool::new(false);
static XGBOOST_READY: AtomicBool = AtomicBool::new(false);
static ISOLATION_READY: AtomicBool = AtomicBool::new(false);

// === Lazy-Initialized Senders ===
pub static XGBOOST_TX: OnceLock<Sender<MlFeatureLog>> = OnceLock::new();
pub static ISOLATION_TX: OnceLock<Sender<MlFeatureLog>> = OnceLock::new();

// === Public API ===

/// Fast check if AI is enabled (called from gateway.rs hot path)
#[inline(always)]
pub fn is_ai_enabled() -> bool {
    AI_ENABLED.load(Ordering::Relaxed)
}

/// Enable AI inference
pub fn enable_ai() {
    AI_ENABLED.store(true, Ordering::Release);
    log::info!("AI security inference enabled");
}

/// Disable AI inference
pub fn disable_ai() {
    AI_ENABLED.store(false, Ordering::Release);
    log::info!("AI security inference disabled");
}

/// Initialize XGBoost sender (called once when thread spawns)
pub fn init_xgboost_sender(tx: Sender<MlFeatureLog>) {
    if XGBOOST_TX.set(tx).is_ok() {
        XGBOOST_READY.store(true, Ordering::Release);
        log::info!("XGBoost sender initialized");
    }
}

/// Initialize Isolation sender
pub fn init_isolation_sender(tx: Sender<MlFeatureLog>) {
    if ISOLATION_TX.set(tx).is_ok() {
        ISOLATION_READY.store(true, Ordering::Release);
        log::info!("Isolation Forest sender initialized");
    }
}

/// Send log to inference (non-blocking, fire-and-forget)
pub fn send_to_inference(log: MlFeatureLog) {
    if !is_ai_enabled() {
        return;
    }

    // Send to XGBoost if ready
    if XGBOOST_READY.load(Ordering::Relaxed) {
        if let Some(tx) = XGBOOST_TX.get() {
            let _ = tx.send(log.clone());
        }
    }

    // Send to Isolation if ready
    if ISOLATION_READY.load(Ordering::Relaxed) {
        if let Some(tx) = ISOLATION_TX.get() {
            let _ = tx.send(log);
        }
    }
}

/// Check if any model thread is ready
pub fn any_model_ready() -> bool {
    XGBOOST_READY.load(Ordering::Relaxed) || ISOLATION_READY.load(Ordering::Relaxed)
}
