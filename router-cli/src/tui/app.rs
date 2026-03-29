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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_stats(req: i64, res: i64, failed: i64, bytes_in: i64, bytes_out: i64) -> UnifiedStats {
        let target = TargetStats {
            req,
            res,
            bytes_in,
            bytes_out,
            failed,
            stalled_count: 0,
            bytes_in_min: 100,
            bytes_in_max: 5000,
            bytes_in_avg: 2500.0,
            bytes_out_min: 200,
            bytes_out_max: 10000,
            bytes_out_avg: 5000.0,
            status: HashMap::from([
                ("200".to_string(), req - failed),
                ("500".to_string(), failed),
            ]),
        };
        UnifiedStats {
            ts: "2026-01-01T00:00:00Z".to_string(),
            gateway: target.clone(),
            proxy: TargetStats {
                req: req / 2,
                res: res / 2,
                bytes_in: bytes_in / 2,
                bytes_out: bytes_out / 2,
                failed: 0,
                ..target
            },
        }
    }

    #[test]
    fn push_stats_adds_to_history() {
        let mut app = AppState::new();
        assert!(app.latest().is_none());

        app.push_stats(make_stats(100, 98, 2, 1024, 2048));
        assert_eq!(app.history.len(), 1);
        assert!(app.latest().is_some());
    }

    #[test]
    fn ring_buffer_caps_at_120() {
        let mut app = AppState::new();
        for i in 0..150 {
            app.push_stats(make_stats(i, i, 0, 1024, 2048));
        }
        assert_eq!(app.history.len(), 120);
        // Oldest should be i=30 (first 30 were evicted)
        assert_eq!(app.history.front().unwrap().gateway.req, 30);
        assert_eq!(app.history.back().unwrap().gateway.req, 149);
    }

    #[test]
    fn load_history_replaces_existing() {
        let mut app = AppState::new();
        app.push_stats(make_stats(1, 1, 0, 100, 200));
        app.push_stats(make_stats(2, 2, 0, 100, 200));
        assert_eq!(app.history.len(), 2);

        let batch = vec![
            make_stats(10, 10, 0, 100, 200),
            make_stats(20, 20, 0, 100, 200),
            make_stats(30, 30, 0, 100, 200),
        ];
        app.load_history(batch);
        assert_eq!(app.history.len(), 3);
        assert_eq!(app.history.front().unwrap().gateway.req, 10);
    }

    #[test]
    fn aggregate_totals_sums_correctly() {
        let mut app = AppState::new();
        app.push_stats(make_stats(100, 98, 2, 1024, 2048));
        app.push_stats(make_stats(200, 195, 5, 2048, 4096));

        let totals = app.aggregate_totals(Panel::Gateway);
        assert_eq!(totals.total_req, 300);
        assert_eq!(totals.total_res, 293);
        assert_eq!(totals.total_failed, 7);
        assert_eq!(totals.total_bytes_in, 3072);
        assert_eq!(totals.total_bytes_out, 6144);
        assert_eq!(totals.intervals, 2);
    }

    #[test]
    fn error_rate_calculation() {
        let mut app = AppState::new();
        app.push_stats(make_stats(100, 90, 10, 0, 0));

        let totals = app.aggregate_totals(Panel::Gateway);
        assert!((totals.error_rate - 10.0).abs() < 0.01);
        assert!((totals.success_rate - 90.0).abs() < 0.01);
    }

    #[test]
    fn error_rate_zero_requests() {
        let mut app = AppState::new();
        app.push_stats(make_stats(0, 0, 0, 0, 0));

        let totals = app.aggregate_totals(Panel::Gateway);
        assert!((totals.error_rate - 0.0).abs() < 0.01);
    }

    #[test]
    fn aggregate_status_codes_merges_across_history() {
        let mut app = AppState::new();
        app.push_stats(make_stats(100, 98, 2, 0, 0));
        app.push_stats(make_stats(50, 49, 1, 0, 0));

        let codes = app.aggregate_status_codes(Panel::Gateway);
        let code_200 = codes.iter().find(|(c, _)| c == "200").map(|(_, v)| *v);
        let code_500 = codes.iter().find(|(c, _)| c == "500").map(|(_, v)| *v);

        assert_eq!(code_200, Some(98 + 49)); // (100-2) + (50-1)
        assert_eq!(code_500, Some(2 + 1));
    }

    #[test]
    fn latest_rates_computes_per_second() {
        let mut app = AppState::new();
        app.push_stats(make_stats(150, 148, 2, 15360, 30720));

        let rates = app.latest_rates(Panel::Gateway, 15.0).unwrap();
        assert!((rates.req_per_sec - 10.0).abs() < 0.01);
        assert!((rates.res_per_sec - 148.0 / 15.0).abs() < 0.01);
        assert!((rates.bytes_in_per_sec - 1024.0).abs() < 0.01);
        assert!((rates.bytes_out_per_sec - 2048.0).abs() < 0.01);
        assert_eq!(rates.failed, 2);
    }

    #[test]
    fn latest_rates_none_when_empty() {
        let app = AppState::new();
        assert!(app.latest_rates(Panel::Gateway, 15.0).is_none());
    }

    #[test]
    fn bytes_series_returns_correct_values() {
        let mut app = AppState::new();
        app.push_stats(make_stats(0, 0, 0, 1000, 2000));
        app.push_stats(make_stats(0, 0, 0, 3000, 4000));

        assert_eq!(app.bytes_in_series(Panel::Gateway), vec![1000, 3000]);
        assert_eq!(app.bytes_out_series(Panel::Gateway), vec![2000, 4000]);
    }

    #[test]
    fn req_series_returns_correct_values() {
        let mut app = AppState::new();
        app.push_stats(make_stats(10, 10, 0, 0, 0));
        app.push_stats(make_stats(20, 20, 0, 0, 0));

        assert_eq!(app.req_series(Panel::Gateway), vec![10, 20]);
    }

    #[test]
    fn panel_focus_affects_proxy_data() {
        let mut app = AppState::new();
        app.push_stats(make_stats(100, 100, 0, 1024, 2048));

        let gw = app.aggregate_totals(Panel::Gateway);
        let px = app.aggregate_totals(Panel::Proxy);

        assert_eq!(gw.total_req, 100);
        assert_eq!(px.total_req, 50); // proxy gets req/2
    }

    #[test]
    fn format_bytes_rate_scales() {
        assert_eq!(format_bytes_rate(500.0), "500 B/s");
        assert_eq!(format_bytes_rate(1024.0), "1.0 KB/s");
        assert_eq!(format_bytes_rate(1_048_576.0), "1.0 MB/s");
        assert_eq!(format_bytes_rate(1_073_741_824.0), "1.0 GB/s");
        assert_eq!(format_bytes_rate(2560.0), "2.5 KB/s");
    }

    #[test]
    fn format_bytes_scales() {
        assert_eq!(format_bytes(500), "500 B");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1_048_576), "1.0 MB");
        assert_eq!(format_bytes(1_073_741_824), "1.0 GB");
    }

    #[test]
    fn format_count_scales() {
        assert_eq!(format_count(42), "42");
        assert_eq!(format_count(1_500), "1.5K");
        assert_eq!(format_count(2_500_000), "2.5M");
    }

    #[test]
    fn latest_target_returns_correct_panel() {
        let mut app = AppState::new();
        app.push_stats(make_stats(100, 90, 10, 1024, 2048));

        let gw = app.latest_target(Panel::Gateway).unwrap();
        assert_eq!(gw.req, 100);

        let px = app.latest_target(Panel::Proxy).unwrap();
        assert_eq!(px.req, 50);
    }
}
