//! CLI Output Helpers
//!
//! Konsolidierte Output-Funktionen für einheitliche CLI-Ausgaben.
//! Extrahiert aus CLI-Modulen für bessere Wartbarkeit und Konsistenz.
//!
//! Einige Funktionen sind noch nicht in allen CLI-Modulen genutzt,
//! werden aber für zukünftige Refactorings bereitgestellt.

#![allow(dead_code)]

use std::error::Error;
use std::fs;
use std::path::Path;

// ============================================================================
// Status-Ausgaben
// ============================================================================

/// Gibt eine Erfolgsmeldung aus (grünes Häkchen)
#[inline]
pub fn success(msg: &str) {
    println!("✅ {}", msg);
}

/// Gibt eine Erfolgsmeldung mit Wert aus
#[inline]
pub fn success_with<T: std::fmt::Display>(msg: &str, value: T) {
    println!("✅ {}: {}", msg, value);
}

/// Gibt eine Fehlermeldung aus (rotes X)
#[inline]
pub fn error(msg: &str) {
    println!("❌ {}", msg);
}

/// Gibt eine Fehlermeldung mit Details aus
#[inline]
pub fn error_with<T: std::fmt::Display>(msg: &str, details: T) {
    println!("❌ {}: {}", msg, details);
}

/// Gibt eine Warnung aus (gelbes Dreieck)
#[inline]
pub fn warning(msg: &str) {
    println!("⚠️  {}", msg);
}

/// Gibt eine Info-Meldung aus (blaues i)
#[inline]
pub fn info(msg: &str) {
    println!("ℹ️  {}", msg);
}

// ============================================================================
// Aktions-Ausgaben (Emojis für spezifische Aktionen)
// ============================================================================

/// Schreib-Operation (Stift)
#[inline]
pub fn writing(msg: &str) {
    println!("📝 {}", msg);
}

/// Such-Operation (Lupe)
#[inline]
pub fn searching(msg: &str) {
    println!("🔍 {}", msg);
}

/// Zeit-Operation (Uhr)
#[inline]
pub fn timing(msg: &str) {
    println!("⏰ {}", msg);
}

/// Paket/Bundle-Operation (Paket)
#[inline]
pub fn packaging(msg: &str) {
    println!("📦 {}", msg);
}

/// Listen-Operation (Clipboard)
#[inline]
pub fn listing(msg: &str) {
    println!("📋 {}", msg);
}

/// Statistik-Ausgabe (Chart)
#[inline]
pub fn stats(msg: &str) {
    println!("📊 {}", msg);
}

/// Speicher-Operation (Diskette)
#[inline]
pub fn saving(msg: &str) {
    println!("💾 {}", msg);
}

/// Eingabe-Operation (Posteingang)
#[inline]
pub fn input(msg: &str) {
    println!("📥 {}", msg);
}

/// Dokument-Operation (Dokument)
#[inline]
pub fn document(msg: &str) {
    println!("📄 {}", msg);
}

/// Pin-Operation (Pinnnadel)
#[inline]
pub fn pinned(msg: &str) {
    println!("📌 {}", msg);
}

/// Lösch-Operation (Mülleimer)
#[inline]
pub fn deleting(msg: &str) {
    println!("🗑️  {}", msg);
}

/// Schlüssel-Operation (Schloss)
#[inline]
pub fn secure(msg: &str) {
    println!("🔐 {}", msg);
}

/// Netzwerk-Operation (Globus)
#[inline]
pub fn network(msg: &str) {
    println!("🌐 {}", msg);
}

/// Schlüssel-Operation (Schlüssel)
#[inline]
pub fn key(msg: &str) {
    println!("🔑 {}", msg);
}

// ============================================================================
// Detail-Ausgaben (eingerückt)
// ============================================================================

/// Gibt ein eingerücktes Detail aus
#[inline]
pub fn detail(label: &str, value: &str) {
    println!("   {:<14} {}", format!("{}:", label), value);
}

/// Gibt ein eingerücktes Detail mit beliebigem Wert aus
#[inline]
pub fn detail_fmt<T: std::fmt::Display>(label: &str, value: T) {
    println!("   {:<14} {}", format!("{}:", label), value);
}

/// Gibt einen einfachen eingerückten Text aus
#[inline]
pub fn indent(msg: &str) {
    println!("   {}", msg);
}

/// Gibt einen nummerierten Schritt aus (für Progress)
#[inline]
pub fn step(current: u32, total: u32, msg: &str) {
    println!("   {}/{} {}...", current, total, msg);
}

// ============================================================================
// JSON Output Helper
// ============================================================================

/// Schreibt JSON in Datei oder gibt es auf stdout aus
pub fn write_json<T: serde::Serialize>(
    data: &T,
    output: Option<&str>,
) -> Result<(), Box<dyn Error>> {
    let json = serde_json::to_string_pretty(data)?;

    if let Some(path) = output {
        fs::write(path, &json)?;
        saving(&format!("Geschrieben nach: {}", path));
    } else {
        println!("{}", json);
    }

    Ok(())
}

/// Schreibt JSON in Datei und gibt Erfolgsmeldung aus
pub fn write_json_file<T: serde::Serialize>(
    data: &T,
    path: &str,
    success_msg: &str,
) -> Result<(), Box<dyn Error>> {
    let json = serde_json::to_string_pretty(data)?;
    fs::write(path, &json)?;
    success(success_msg);
    Ok(())
}

// ============================================================================
// Datei-Output Helper
// ============================================================================

/// Schreibt Bytes in Datei mit Erfolgsmeldung
pub fn write_bytes(data: &[u8], path: &str, success_msg: &str) -> Result<(), Box<dyn Error>> {
    fs::write(path, data)?;
    success(success_msg);
    Ok(())
}

/// Prüft ob Ausgabedatei existiert (für --force Flag)
pub fn check_output_exists(path: &str, force: bool) -> Result<(), Box<dyn Error>> {
    if Path::new(path).exists() && !force {
        return Err(format!("Output existiert bereits: {} (nutze --force zum Überschreiben)", path).into());
    }
    Ok(())
}

// ============================================================================
// Tabellen-Helper
// ============================================================================

/// Gibt einen Tabellen-Header aus
pub fn table_header(columns: &[(&str, usize)]) {
    let header: String = columns
        .iter()
        .map(|(name, width)| format!("{:<width$}", name, width = width))
        .collect::<Vec<_>>()
        .join(" ");
    println!("{}", header);

    let separator_len: usize = columns.iter().map(|(_, w)| w + 1).sum();
    println!("{}", "-".repeat(separator_len));
}

/// Gibt eine Tabellen-Zeile aus
pub fn table_row(values: &[(&str, usize)]) {
    let row: String = values
        .iter()
        .map(|(val, width)| format!("{:<width$}", val, width = width))
        .collect::<Vec<_>>()
        .join(" ");
    println!("{}", row);
}

// ============================================================================
// Section Headers
// ============================================================================

/// Gibt einen Abschnitts-Header aus
pub fn section(title: &str) {
    println!("\n{}", title);
}

/// Gibt eine Trennlinie aus
pub fn separator() {
    println!("{}", "-".repeat(60));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_output_exists_force() {
        // Mit force=true sollte kein Fehler kommen, auch wenn Datei existiert
        // (hier testen wir nur, dass die Funktion korrekt kompiliert)
        let result = check_output_exists("/nonexistent/path", true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_output_exists_no_force_nonexistent() {
        // Nicht-existierende Datei ohne force sollte OK sein
        let result = check_output_exists("/nonexistent/path/that/does/not/exist", false);
        assert!(result.is_ok());
    }
}
