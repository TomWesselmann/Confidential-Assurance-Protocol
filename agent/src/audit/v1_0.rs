use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

/// Audit-Log-Eintrag mit Hash-Chain
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuditEntry {
    pub seq: u64,
    pub ts: String,
    pub event: String,
    pub details: serde_json::Value,
    pub prev_digest: String,
    pub digest: String,
}

/// Audit-Log-Manager für kryptografische Event-Logs
pub struct AuditLog {
    path: String,
    last_digest: String,
    seq: u64,
}

impl AuditLog {
    /// Erstellt einen neuen AuditLog oder lädt einen bestehenden
    ///
    /// # Argumente
    /// * `path` - Pfad zur JSONL-Audit-Datei
    ///
    /// # Rückgabe
    /// Neuer AuditLog-Manager
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let path_str = path.as_ref().to_str().unwrap().to_string();

        // Lese letzten Eintrag falls Datei existiert
        let (last_digest, seq) = if path.as_ref().exists() {
            Self::read_last_entry(&path_str)?
        } else {
            ("0x0000000000000000000000000000000000000000000000000000000000000000".to_string(), 0)
        };

        Ok(AuditLog {
            path: path_str,
            last_digest,
            seq,
        })
    }

    /// Liest den letzten Eintrag aus der Audit-Datei
    ///
    /// # Argumente
    /// * `path` - Pfad zur Datei
    ///
    /// # Rückgabe
    /// Tuple (letzter Digest, letzte Sequenznummer)
    fn read_last_entry(path: &str) -> Result<(String, u64), Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut last_entry: Option<AuditEntry> = None;

        for line in reader.lines() {
            let line = line?;
            if !line.trim().is_empty() {
                let entry: AuditEntry = serde_json::from_str(&line)?;
                last_entry = Some(entry);
            }
        }

        match last_entry {
            Some(entry) => Ok((entry.digest, entry.seq)),
            None => Ok((
                "0x0000000000000000000000000000000000000000000000000000000000000000".to_string(),
                0,
            )),
        }
    }

    /// Berechnet SHA3-256 Digest für einen Audit-Eintrag
    ///
    /// # Argumente
    /// * `entry` - Der Audit-Eintrag (ohne digest Feld)
    /// * `prev_digest` - Digest des vorherigen Eintrags
    ///
    /// # Rückgabe
    /// Hex-String des SHA3-256 Hashes
    fn compute_digest(
        seq: u64,
        ts: &str,
        event: &str,
        details: &serde_json::Value,
        prev_digest: &str,
    ) -> String {
        let mut hasher = Sha3_256::new();
        hasher.update(seq.to_string().as_bytes());
        hasher.update(ts.as_bytes());
        hasher.update(event.as_bytes());
        hasher.update(details.to_string().as_bytes());
        hasher.update(prev_digest.as_bytes());

        let result = hasher.finalize();
        format!("0x{}", hex::encode(result))
    }

    /// Fügt einen neuen Event zum Audit-Log hinzu
    ///
    /// # Argumente
    /// * `event` - Event-Typ (z.B. "merkle_root_computed")
    /// * `details` - JSON-Details zum Event
    ///
    /// # Rückgabe
    /// Result mit () bei Erfolg
    pub fn log_event(&mut self, event: &str, details: serde_json::Value) -> Result<(), Box<dyn Error>> {
        self.seq += 1;
        let ts = Utc::now().to_rfc3339();
        let prev_digest = self.last_digest.clone();

        let digest = Self::compute_digest(self.seq, &ts, event, &details, &prev_digest);

        let entry = AuditEntry {
            seq: self.seq,
            ts,
            event: event.to_string(),
            details,
            prev_digest,
            digest: digest.clone(),
        };

        // Schreibe als JSONL (eine Zeile pro Entry)
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;

        let json = serde_json::to_string(&entry)?;
        writeln!(file, "{}", json)?;

        self.last_digest = digest;

        Ok(())
    }

    /// Gibt die aktuelle Sequenznummer zurück
    #[allow(dead_code)]
    pub fn current_seq(&self) -> u64 {
        self.seq
    }

    /// Gibt den aktuellen Audit-Tip (letzter Digest) zurück
    ///
    /// # Rückgabe
    /// Hex-String des letzten Digest
    pub fn get_tip(&self) -> String {
        self.last_digest.clone()
    }

    /// Schreibt den Audit-Tip (H_n) in eine Datei
    ///
    /// # Argumente
    /// * `out_path` - Zielpfad für die Tip-Datei
    ///
    /// # Rückgabe
    /// Result mit () bei Erfolg
    pub fn write_tip<P: AsRef<Path>>(&self, out_path: P) -> Result<(), Box<dyn Error>> {
        let tip = self.get_tip();
        // Entferne "0x" Präfix für saubere Hex-Ausgabe
        let hex_only = tip.trim_start_matches("0x");
        std::fs::write(out_path, hex_only)?;
        Ok(())
    }

    /// Liest den Audit-Tip aus einer Datei
    ///
    /// # Argumente
    /// * `tip_path` - Pfad zur Tip-Datei
    ///
    /// # Rückgabe
    /// Result mit Hex-String des Tip
    pub fn read_tip<P: AsRef<Path>>(tip_path: P) -> Result<String, Box<dyn Error>> {
        let hex = std::fs::read_to_string(tip_path)?;
        let hex = hex.trim();
        // Füge "0x" Präfix hinzu falls nicht vorhanden
        if hex.starts_with("0x") {
            Ok(hex.to_string())
        } else {
            Ok(format!("0x{}", hex))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;

    #[test]
    fn test_audit_log_creation() {
        let temp_path = "/tmp/test_audit.jsonl";
        let _ = fs::remove_file(temp_path); // Cleanup falls existiert

        let mut audit = AuditLog::new(temp_path).unwrap();
        assert_eq!(audit.current_seq(), 0);

        audit.log_event("test_event", json!({"foo": "bar"})).unwrap();
        assert_eq!(audit.current_seq(), 1);

        let _ = fs::remove_file(temp_path); // Cleanup
    }

    #[test]
    fn test_digest_deterministic() {
        let digest1 = AuditLog::compute_digest(
            1,
            "2025-01-01T00:00:00Z",
            "test",
            &json!({"foo": "bar"}),
            "0x0000",
        );
        let digest2 = AuditLog::compute_digest(
            1,
            "2025-01-01T00:00:00Z",
            "test",
            &json!({"foo": "bar"}),
            "0x0000",
        );
        assert_eq!(digest1, digest2);
    }

    #[test]
    fn test_digest_chain() {
        let digest1 = AuditLog::compute_digest(
            1,
            "2025-01-01T00:00:00Z",
            "event1",
            &json!({}),
            "0x0000",
        );
        let digest2 = AuditLog::compute_digest(
            2,
            "2025-01-01T00:00:01Z",
            "event2",
            &json!({}),
            &digest1,
        );

        // Digests sollten unterschiedlich sein
        assert_ne!(digest1, digest2);
    }

    #[test]
    fn tip_write_and_read_ok() {
        let temp_audit = "/tmp/test_tip_audit.jsonl";
        let temp_tip = "/tmp/test_audit.head";
        let _ = fs::remove_file(temp_audit); // Cleanup
        let _ = fs::remove_file(temp_tip); // Cleanup

        // Erstelle Audit-Log mit einigen Events
        let mut audit = AuditLog::new(temp_audit).unwrap();
        audit.log_event("test_event", json!({"foo": "bar"})).unwrap();
        audit.log_event("another_event", json!({"baz": "qux"})).unwrap();

        // Schreibe Tip
        audit.write_tip(temp_tip).unwrap();

        // Lese Tip zurück
        let tip = AuditLog::read_tip(temp_tip).unwrap();

        // Tip sollte dem letzten Digest entsprechen
        assert_eq!(tip, audit.get_tip());
        assert!(tip.starts_with("0x"));

        let _ = fs::remove_file(temp_audit); // Cleanup
        let _ = fs::remove_file(temp_tip); // Cleanup
    }
}
