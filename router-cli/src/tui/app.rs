use serde::Deserialize;
use std::collections::{HashMap, VecDeque};
use std::time::Instant;

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
    pub show_help: bool,
    pub started_at: Instant,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            history: VecDeque::with_capacity(MAX_HISTORY),
            focus: Panel::Gateway,
            connected: false,
            status_msg: "Connecting...".to_string(),
            should_quit: false,
            show_help: false,
            started_at: Instant::now(),
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

    pub fn uptime_str(&self) -> String {
        let secs = self.started_at.elapsed().as_secs();
        let h = secs / 3600;
        let m = (secs % 3600) / 60;
        let s = secs % 60;
        if h > 0 {
            format!("{h}h {m:02}m {s:02}s")
        } else if m > 0 {
            format!("{m}m {s:02}s")
        } else {
            format!("{s}s")
        }
    }

    /// Aggregate totals across all history for a panel.
    pub fn aggregate_totals(&self, panel: Panel) -> AggregateTotals {
        let mut total_req: i64 = 0;
        let mut total_res: i64 = 0;
        let mut total_failed: i64 = 0;
        let mut total_bytes_in: i64 = 0;
        let mut total_bytes_out: i64 = 0;

        for stats in &self.history {
            let t = match panel {
                Panel::Gateway => &stats.gateway,
                Panel::Proxy => &stats.proxy,
            };
            total_req += t.req;
            total_res += t.res;
            total_failed += t.failed;
            total_bytes_in += t.bytes_in;
            total_bytes_out += t.bytes_out;
        }

        let error_rate = if total_req > 0 {
            (total_failed as f64 / total_req as f64) * 100.0
        } else {
            0.0
        };

        let success_rate = 100.0 - error_rate;

        AggregateTotals {
            total_req,
            total_res,
            total_failed,
            total_bytes_in,
            total_bytes_out,
            error_rate,
            success_rate,
            intervals: self.history.len(),
        }
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

    /// Get req/s history for sparkline.
    pub fn req_series(&self, panel: Panel) -> Vec<u64> {
        self.history
            .iter()
            .map(|s| {
                let t = match panel {
                    Panel::Gateway => &s.gateway,
                    Panel::Proxy => &s.proxy,
                };
                t.req.max(0) as u64
            })
            .collect()
    }

    /// Get the latest TargetStats for a panel.
    pub fn latest_target(&self, panel: Panel) -> Option<&TargetStats> {
        let s = self.latest()?;
        Some(match panel {
            Panel::Gateway => &s.gateway,
            Panel::Proxy => &s.proxy,
        })
    }

    /// Get per-second rates for the latest stats of a given panel.
    pub fn latest_rates(&self, panel: Panel, interval_secs: f64) -> Option<Rates> {
        let t = self.latest_target(panel)?;
        Some(Rates {
            req_per_sec: t.req as f64 / interval_secs,
            res_per_sec: t.res as f64 / interval_secs,
            bytes_in_per_sec: t.bytes_in as f64 / interval_secs,
            bytes_out_per_sec: t.bytes_out as f64 / interval_secs,
            failed: t.failed,
            stalled: t.stalled_count,
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
}

pub struct AggregateTotals {
    pub total_req: i64,
    pub total_res: i64,
    pub total_failed: i64,
    pub total_bytes_in: i64,
    pub total_bytes_out: i64,
    pub error_rate: f64,
    pub success_rate: f64,
    pub intervals: usize,
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

pub fn format_bytes(b: i64) -> String {
    let abs = b.unsigned_abs();
    if abs >= 1_073_741_824 {
        format!("{:.1} GB", b as f64 / 1_073_741_824.0)
    } else if abs >= 1_048_576 {
        format!("{:.1} MB", b as f64 / 1_048_576.0)
    } else if abs >= 1_024 {
        format!("{:.1} KB", b as f64 / 1_024.0)
    } else {
        format!("{b} B")
    }
}

pub fn format_count(n: i64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        format!("{n}")
    }
}
