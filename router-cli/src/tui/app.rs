use serde::Deserialize;
use std::collections::{HashMap, VecDeque};

const MAX_HISTORY: usize = 120;

#[derive(Debug, Clone, Deserialize)]
pub struct UnifiedStats {
    pub ts: String,
    pub gateway: TargetStats,
    pub proxy: TargetStats,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct TargetStats {
    pub req: i64,
    pub res: i64,
    pub bytes_in: i64,
    pub bytes_out: i64,
    pub status: HashMap<String, i64>,
    pub failed: i64,
    pub bytes_in_min: i64,
    pub bytes_in_max: i64,
    pub bytes_in_avg: f64,
    pub bytes_out_min: i64,
    pub bytes_out_max: i64,
    pub bytes_out_avg: f64,
    pub stalled_count: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    Gateway,
    Proxy,
}

pub struct AppState {
    pub history: VecDeque<UnifiedStats>,
    pub focus: Panel,
    pub connected: bool,
    pub status_msg: String,
    pub should_quit: bool,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            history: VecDeque::with_capacity(MAX_HISTORY),
            focus: Panel::Gateway,
            connected: false,
            status_msg: "Connecting...".to_string(),
            should_quit: false,
        }
    }

    pub fn push_stats(&mut self, stats: UnifiedStats) {
        if self.history.len() >= MAX_HISTORY {
            self.history.pop_front();
        }
        self.history.push_back(stats);
    }

    pub fn load_history(&mut self, batch: Vec<UnifiedStats>) {
        self.history.clear();
        for s in batch {
            self.push_stats(s);
        }
    }

    pub fn latest(&self) -> Option<&UnifiedStats> {
        self.history.back()
    }

    /// Aggregate status codes across all history points for a given panel.
    pub fn aggregate_status_codes(&self, panel: Panel) -> Vec<(String, i64)> {
        let mut totals: HashMap<String, i64> = HashMap::new();
        for stats in &self.history {
            let target = match panel {
                Panel::Gateway => &stats.gateway,
                Panel::Proxy => &stats.proxy,
            };
            for (code, count) in &target.status {
                *totals.entry(code.clone()).or_default() += count;
            }
        }
        let mut sorted: Vec<_> = totals.into_iter().collect();
        sorted.sort_by(|a, b| a.0.cmp(&b.0));
        sorted
    }

    /// Get bytes_in history for sparkline.
    pub fn bytes_in_series(&self, panel: Panel) -> Vec<u64> {
        self.history
            .iter()
            .map(|s| {
                let t = match panel {
                    Panel::Gateway => &s.gateway,
                    Panel::Proxy => &s.proxy,
                };
                t.bytes_in.max(0) as u64
            })
            .collect()
    }

    /// Get bytes_out history for sparkline.
    pub fn bytes_out_series(&self, panel: Panel) -> Vec<u64> {
        self.history
            .iter()
            .map(|s| {
                let t = match panel {
                    Panel::Gateway => &s.gateway,
                    Panel::Proxy => &s.proxy,
                };
                t.bytes_out.max(0) as u64
            })
            .collect()
    }

    /// Get per-second rates for the latest stats of a given panel.
    pub fn latest_rates(&self, panel: Panel, interval_secs: f64) -> Option<Rates> {
        let s = self.latest()?;
        let t = match panel {
            Panel::Gateway => &s.gateway,
            Panel::Proxy => &s.proxy,
        };
        Some(Rates {
            req_per_sec: t.req as f64 / interval_secs,
            res_per_sec: t.res as f64 / interval_secs,
            bytes_in_per_sec: t.bytes_in as f64 / interval_secs,
            bytes_out_per_sec: t.bytes_out as f64 / interval_secs,
            failed: t.failed,
            stalled: t.stalled_count,
            bytes_in_min: t.bytes_in_min,
            bytes_in_max: t.bytes_in_max,
            bytes_in_avg: t.bytes_in_avg,
            bytes_out_min: t.bytes_out_min,
            bytes_out_max: t.bytes_out_max,
            bytes_out_avg: t.bytes_out_avg,
        })
    }
}

pub struct Rates {
    pub req_per_sec: f64,
    pub res_per_sec: f64,
    pub bytes_in_per_sec: f64,
    pub bytes_out_per_sec: f64,
    pub failed: i64,
    pub stalled: i64,
    pub bytes_in_min: i64,
    pub bytes_in_max: i64,
    pub bytes_in_avg: f64,
    pub bytes_out_min: i64,
    pub bytes_out_max: i64,
    pub bytes_out_avg: f64,
}

pub fn format_bytes_rate(bytes_per_sec: f64) -> String {
    let abs = bytes_per_sec.abs();
    if abs >= 1_073_741_824.0 {
        format!("{:.1} GB/s", bytes_per_sec / 1_073_741_824.0)
    } else if abs >= 1_048_576.0 {
        format!("{:.1} MB/s", bytes_per_sec / 1_048_576.0)
    } else if abs >= 1_024.0 {
        format!("{:.1} KB/s", bytes_per_sec / 1_024.0)
    } else {
        format!("{:.0} B/s", bytes_per_sec)
    }
}
