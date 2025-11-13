//! # Drift Analysis Module (Week 6 - B2)
//!
//! Advanced drift tracking mit rolling windows und ring buffer.
//!
//! ## Features
//! - 5-Minuten Rolling Window für Drift-Ratio
//! - Ring Buffer für effiziente Time-Series-Speicherung
//! - Automatische Aggregation und Cleanup
//! - Sliding Window Queries

use crate::orchestrator::{Verdict, VerdictPair};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::{Duration, SystemTime};

/// Time-stamped drift event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftEvent {
    /// Timestamp des Events
    pub timestamp: SystemTime,

    /// Shadow verdict
    pub shadow: Verdict,

    /// Enforced verdict
    pub enforced: Verdict,

    /// Wurde Enforcement angewendet?
    pub enforced_applied: bool,

    /// Policy ID
    pub policy_id: String,

    /// Request ID
    pub request_id: String,
}

impl DriftEvent {
    /// Erstellt DriftEvent aus VerdictPair
    pub fn from_verdict_pair(pair: &VerdictPair, policy_id: String, request_id: String) -> Self {
        Self {
            timestamp: SystemTime::now(),
            shadow: pair.shadow.clone(),
            enforced: pair.enforced.clone(),
            enforced_applied: pair.enforced_applied,
            policy_id,
            request_id,
        }
    }

    /// Prüft, ob Event Drift aufweist
    pub fn has_drift(&self) -> bool {
        self.enforced_applied && self.shadow != self.enforced
    }
}

/// Ring buffer für time-series drift events
#[derive(Debug)]
pub struct DriftRingBuffer {
    /// Events (circular buffer)
    events: VecDeque<DriftEvent>,

    /// Maximale Buffer-Größe
    max_size: usize,

    /// Maximales Event-Alter (für Rolling Window)
    max_age: Duration,
}

impl DriftRingBuffer {
    /// Erstellt neuen Ring Buffer
    ///
    /// # Parameters
    /// - `max_size`: Maximale Anzahl Events im Buffer (z.B. 10000)
    /// - `max_age`: Maximales Event-Alter (z.B. 5 Minuten)
    pub fn new(max_size: usize, max_age: Duration) -> Self {
        Self {
            events: VecDeque::with_capacity(max_size),
            max_size,
            max_age,
        }
    }

    /// Fügt Event zum Buffer hinzu
    pub fn push(&mut self, event: DriftEvent) {
        // Remove oldest if buffer is full
        if self.events.len() >= self.max_size {
            self.events.pop_front();
        }

        self.events.push_back(event);

        // Cleanup old events
        self.cleanup_old_events();
    }

    /// Entfernt Events älter als max_age
    fn cleanup_old_events(&mut self) {
        let now = SystemTime::now();

        while let Some(front) = self.events.front() {
            if let Ok(age) = now.duration_since(front.timestamp) {
                if age > self.max_age {
                    self.events.pop_front();
                } else {
                    break;
                }
            } else {
                // Timestamp in der Zukunft? Entfernen
                self.events.pop_front();
            }
        }
    }

    /// Liefert alle Events im Rolling Window
    pub fn get_events_in_window(&self, window: Duration) -> Vec<&DriftEvent> {
        let now = SystemTime::now();
        self.events
            .iter()
            .filter(|e| {
                now.duration_since(e.timestamp)
                    .map(|age| age <= window)
                    .unwrap_or(false)
            })
            .collect()
    }

    /// Berechnet Drift-Ratio für Rolling Window
    pub fn drift_ratio_window(&self, window: Duration) -> f64 {
        let events = self.get_events_in_window(window);

        if events.is_empty() {
            return 0.0;
        }

        let drift_count = events.iter().filter(|e| e.has_drift()).count();
        drift_count as f64 / events.len() as f64
    }

    /// Anzahl Events im Buffer
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Prüft, ob Buffer leer ist
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Leert den Buffer
    pub fn clear(&mut self) {
        self.events.clear();
    }
}

/// Advanced Drift Analyzer mit Rolling Windows
#[derive(Debug)]
pub struct DriftAnalyzer {
    /// Ring buffer für Events
    buffer: DriftRingBuffer,

    /// Standard-Fenster für Drift-Ratio (5 Minuten)
    default_window: Duration,
}

impl DriftAnalyzer {
    /// Erstellt neuen DriftAnalyzer
    ///
    /// # Parameters
    /// - `buffer_size`: Maximale Buffer-Größe (default: 10000)
    /// - `max_age`: Maximales Event-Alter (default: 10 Minuten)
    /// - `default_window`: Standard-Fenster für Queries (default: 5 Minuten)
    pub fn new(buffer_size: usize, max_age: Duration, default_window: Duration) -> Self {
        Self {
            buffer: DriftRingBuffer::new(buffer_size, max_age),
            default_window,
        }
    }

    /// Erstellt DriftAnalyzer mit Standard-Werten
    ///
    /// - Buffer: 10000 Events
    /// - Max Age: 10 Minuten
    /// - Default Window: 5 Minuten
    #[allow(clippy::should_implement_trait)]
    pub fn default() -> Self {
        Self::new(
            10_000,
            Duration::from_secs(600), // 10 minutes
            Duration::from_secs(300), // 5 minutes
        )
    }

    /// Zeichnet VerdictPair auf
    pub fn record_verdict_pair(
        &mut self,
        pair: &VerdictPair,
        policy_id: String,
        request_id: String,
    ) {
        let event = DriftEvent::from_verdict_pair(pair, policy_id, request_id);
        self.buffer.push(event);
    }

    /// Berechnet Drift-Ratio für Standard-Fenster (5min)
    pub fn drift_ratio_5m(&self) -> f64 {
        self.buffer.drift_ratio_window(self.default_window)
    }

    /// Berechnet Drift-Ratio für Custom-Fenster
    pub fn drift_ratio_custom(&self, window: Duration) -> f64 {
        self.buffer.drift_ratio_window(window)
    }

    /// Liefert Drift-Events im Standard-Fenster
    pub fn drift_events_5m(&self) -> Vec<&DriftEvent> {
        self.buffer
            .get_events_in_window(self.default_window)
            .into_iter()
            .filter(|e| e.has_drift())
            .collect()
    }

    /// Liefert alle Events im Standard-Fenster
    pub fn events_5m(&self) -> Vec<&DriftEvent> {
        self.buffer.get_events_in_window(self.default_window)
    }

    /// Berechnet Request-Rate (requests/sec) im Standard-Fenster
    pub fn request_rate_5m(&self) -> f64 {
        let events = self.buffer.get_events_in_window(self.default_window);

        if events.is_empty() {
            return 0.0;
        }

        // Calculate time span
        let oldest = events.first().unwrap().timestamp;
        let newest = events.last().unwrap().timestamp;

        if let Ok(duration) = newest.duration_since(oldest) {
            let seconds = duration.as_secs_f64();
            if seconds > 0.0 {
                return events.len() as f64 / seconds;
            }
        }

        0.0
    }

    /// Aggregierte Statistiken für Standard-Fenster
    pub fn stats_5m(&self) -> DriftStats {
        let events = self.buffer.get_events_in_window(self.default_window);
        let drift_events = events.iter().filter(|e| e.has_drift()).count();

        DriftStats {
            total_events: events.len(),
            drift_events,
            drift_ratio: if events.is_empty() {
                0.0
            } else {
                drift_events as f64 / events.len() as f64
            },
            window_duration: self.default_window,
        }
    }

    /// Prüft, ob Drift-Ratio Schwellwert überschreitet
    pub fn exceeds_threshold(&self, threshold: f64) -> bool {
        self.drift_ratio_5m() > threshold
    }

    /// Leert den Buffer
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Anzahl Events im Buffer
    pub fn buffer_size(&self) -> usize {
        self.buffer.len()
    }
}

impl Default for DriftAnalyzer {
    fn default() -> Self {
        Self::default()
    }
}

/// Drift-Statistiken für Reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftStats {
    /// Gesamtanzahl Events im Fenster
    pub total_events: usize,

    /// Anzahl Drift-Events
    pub drift_events: usize,

    /// Drift-Ratio (0.0 - 1.0)
    pub drift_ratio: f64,

    /// Fenstergröße
    pub window_duration: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drift_event_creation() {
        let pair = VerdictPair {
            shadow: Verdict::Ok,
            enforced: Verdict::Fail,
            enforced_applied: true,
        };

        let event =
            DriftEvent::from_verdict_pair(&pair, "test.v1".to_string(), "req-123".to_string());

        assert_eq!(event.policy_id, "test.v1");
        assert_eq!(event.request_id, "req-123");
        assert!(event.has_drift());
    }

    #[test]
    fn test_ring_buffer_push() {
        let mut buffer = DriftRingBuffer::new(3, Duration::from_secs(300));

        for i in 0..5 {
            let pair = VerdictPair {
                shadow: Verdict::Ok,
                enforced: Verdict::Ok,
                enforced_applied: true,
            };
            let event =
                DriftEvent::from_verdict_pair(&pair, format!("test_{}", i), format!("req_{}", i));
            buffer.push(event);
        }

        // Buffer sollte max 3 Events enthalten
        assert_eq!(buffer.len(), 3);
    }

    #[test]
    fn test_ring_buffer_rolling_window() {
        let mut buffer = DriftRingBuffer::new(100, Duration::from_secs(10));

        // Add events with different timestamps
        for i in 0..5 {
            let pair = VerdictPair {
                shadow: Verdict::Ok,
                enforced: if i % 2 == 0 {
                    Verdict::Fail
                } else {
                    Verdict::Ok
                },
                enforced_applied: true,
            };
            let event =
                DriftEvent::from_verdict_pair(&pair, "test.v1".to_string(), format!("req_{}", i));
            buffer.push(event);
        }

        // All events should be in 1-second window (just added)
        let events = buffer.get_events_in_window(Duration::from_secs(1));
        assert_eq!(events.len(), 5);
    }

    #[test]
    fn test_drift_analyzer_default() {
        let analyzer = DriftAnalyzer::default();

        assert_eq!(analyzer.drift_ratio_5m(), 0.0);
        assert_eq!(analyzer.buffer_size(), 0);
    }

    #[test]
    fn test_drift_analyzer_record() {
        let mut analyzer = DriftAnalyzer::default();

        // Record non-drift event
        let pair1 = VerdictPair {
            shadow: Verdict::Ok,
            enforced: Verdict::Ok,
            enforced_applied: true,
        };
        analyzer.record_verdict_pair(&pair1, "test.v1".to_string(), "req1".to_string());

        // Record drift event
        let pair2 = VerdictPair {
            shadow: Verdict::Ok,
            enforced: Verdict::Fail,
            enforced_applied: true,
        };
        analyzer.record_verdict_pair(&pair2, "test.v1".to_string(), "req2".to_string());

        assert_eq!(analyzer.buffer_size(), 2);
        assert_eq!(analyzer.drift_ratio_5m(), 0.5); // 1/2 = 50%
    }

    #[test]
    fn test_drift_analyzer_stats() {
        let mut analyzer = DriftAnalyzer::default();

        // Add 10 events, 3 with drift
        for i in 0..10 {
            let pair = VerdictPair {
                shadow: Verdict::Ok,
                enforced: if i < 3 { Verdict::Fail } else { Verdict::Ok },
                enforced_applied: true,
            };
            analyzer.record_verdict_pair(&pair, "test.v1".to_string(), format!("req{}", i));
        }

        let stats = analyzer.stats_5m();
        assert_eq!(stats.total_events, 10);
        assert_eq!(stats.drift_events, 3);
        assert_eq!(stats.drift_ratio, 0.3);
    }

    #[test]
    fn test_drift_analyzer_threshold() {
        let mut analyzer = DriftAnalyzer::default();

        // Add events below threshold
        for i in 0..100 {
            let pair = VerdictPair {
                shadow: Verdict::Ok,
                enforced: if i == 0 { Verdict::Fail } else { Verdict::Ok }, // 1% drift
                enforced_applied: true,
            };
            analyzer.record_verdict_pair(&pair, "test.v1".to_string(), format!("req{}", i));
        }

        assert!(!analyzer.exceeds_threshold(0.02)); // 2% threshold not exceeded
        assert!(analyzer.exceeds_threshold(0.005)); // 0.5% threshold exceeded
    }

    #[test]
    fn test_drift_events_filter() {
        let mut analyzer = DriftAnalyzer::default();

        // Add 5 events, 2 with drift
        for i in 0..5 {
            let pair = VerdictPair {
                shadow: Verdict::Ok,
                enforced: if i < 2 { Verdict::Fail } else { Verdict::Ok },
                enforced_applied: true,
            };
            analyzer.record_verdict_pair(&pair, "test.v1".to_string(), format!("req{}", i));
        }

        let drift_events = analyzer.drift_events_5m();
        assert_eq!(drift_events.len(), 2);

        let all_events = analyzer.events_5m();
        assert_eq!(all_events.len(), 5);
    }
}
