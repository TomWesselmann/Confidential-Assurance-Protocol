/// CAPZ (CAP Proof Container v2) Format
///
/// Binary container format for proof packages with versioning and metadata.
///
/// # Header Layout (Little Endian, 78 bytes fixed)
/// ```
/// magic[4]      = b"CAPZ"
/// version[2]    = 0x0002 (u16 LE)
/// backend[1]    = 0=mock, 1=zkvm, 2=halo2
/// reserved[1]   = 0x00
/// vk_hash[32]   = verification key hash (optional, zeros if N/A)
/// params_hash[32] = params hash (optional, zeros if N/A)
/// payload_len[4]  = u32 LE
/// payload[payload_len] = proof data (JSON or binary)
/// ```
use anyhow::{anyhow, Result};
use std::io::{Cursor, Read, Write};

/// CAPZ Magic bytes
pub const CAPZ_MAGIC: &[u8; 4] = b"CAPZ";

/// Current CAPZ version
pub const CAPZ_VERSION: u16 = 0x0002;

/// Header size in bytes
pub const CAPZ_HEADER_SIZE: usize = 78;

/// Proof backend type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ProofBackend {
    Mock = 0,
    ZkVm = 1,
    Halo2 = 2,
}

impl ProofBackend {
    /// Parse backend from u8
    pub fn from_u8(value: u8) -> Result<Self> {
        match value {
            0 => Ok(ProofBackend::Mock),
            1 => Ok(ProofBackend::ZkVm),
            2 => Ok(ProofBackend::Halo2),
            _ => Err(anyhow!("Invalid backend value: {}", value)),
        }
    }

    /// Convert to string name
    pub fn as_str(&self) -> &str {
        match self {
            ProofBackend::Mock => "mock",
            ProofBackend::ZkVm => "zkvm",
            ProofBackend::Halo2 => "halo2",
        }
    }
}

/// CAPZ Header
#[derive(Debug, Clone)]
pub struct CapzHeader {
    pub version: u16,
    pub backend: ProofBackend,
    pub vk_hash: [u8; 32],
    pub params_hash: [u8; 32],
    pub payload_len: u32,
}

impl CapzHeader {
    /// Create new header
    pub fn new(backend: ProofBackend, payload_len: u32) -> Self {
        Self {
            version: CAPZ_VERSION,
            backend,
            vk_hash: [0u8; 32],
            params_hash: [0u8; 32],
            payload_len,
        }
    }

    /// Create header with hashes
    pub fn with_hashes(
        backend: ProofBackend,
        vk_hash: [u8; 32],
        params_hash: [u8; 32],
        payload_len: u32,
    ) -> Self {
        Self {
            version: CAPZ_VERSION,
            backend,
            vk_hash,
            params_hash,
            payload_len,
        }
    }

    /// Read header from bytes
    pub fn read<R: Read>(reader: &mut R) -> Result<Self> {
        let mut buf = [0u8; CAPZ_HEADER_SIZE];
        reader.read_exact(&mut buf)?;

        // Check magic
        if &buf[0..4] != CAPZ_MAGIC {
            return Err(anyhow!("Invalid CAPZ magic bytes"));
        }

        // Parse version (LE)
        let version = u16::from_le_bytes([buf[4], buf[5]]);
        if version != CAPZ_VERSION {
            return Err(anyhow!(
                "Unsupported CAPZ version: 0x{:04x} (expected 0x{:04x})",
                version,
                CAPZ_VERSION
            ));
        }

        // Parse backend
        let backend = ProofBackend::from_u8(buf[6])?;

        // Reserved byte (should be 0)
        if buf[7] != 0 {
            return Err(anyhow!("Reserved byte is non-zero: {}", buf[7]));
        }

        // Parse hashes
        let mut vk_hash = [0u8; 32];
        let mut params_hash = [0u8; 32];
        vk_hash.copy_from_slice(&buf[8..40]);
        params_hash.copy_from_slice(&buf[40..72]);

        // Parse payload length (LE)
        let payload_len = u32::from_le_bytes([buf[72], buf[73], buf[74], buf[75]]);

        // Validate payload length (max 100 MB)
        if payload_len > 100_000_000 {
            return Err(anyhow!("Payload length too large: {} bytes", payload_len));
        }

        Ok(Self {
            version,
            backend,
            vk_hash,
            params_hash,
            payload_len,
        })
    }

    /// Write header to bytes
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        let mut buf = [0u8; CAPZ_HEADER_SIZE];

        // Magic
        buf[0..4].copy_from_slice(CAPZ_MAGIC);

        // Version (LE)
        buf[4..6].copy_from_slice(&self.version.to_le_bytes());

        // Backend
        buf[6] = self.backend as u8;

        // Reserved
        buf[7] = 0;

        // Hashes
        buf[8..40].copy_from_slice(&self.vk_hash);
        buf[40..72].copy_from_slice(&self.params_hash);

        // Payload length (LE)
        buf[72..76].copy_from_slice(&self.payload_len.to_le_bytes());

        // Reserved padding (2 bytes)
        buf[76] = 0;
        buf[77] = 0;

        writer.write_all(&buf)?;
        Ok(())
    }
}

/// CAPZ Container
#[derive(Debug, Clone)]
pub struct CapzContainer {
    pub header: CapzHeader,
    pub payload: Vec<u8>,
}

impl CapzContainer {
    /// Create new container
    pub fn new(backend: ProofBackend, payload: Vec<u8>) -> Self {
        let payload_len = payload.len() as u32;
        let header = CapzHeader::new(backend, payload_len);
        Self { header, payload }
    }

    /// Create container with hashes
    pub fn with_hashes(
        backend: ProofBackend,
        vk_hash: [u8; 32],
        params_hash: [u8; 32],
        payload: Vec<u8>,
    ) -> Self {
        let payload_len = payload.len() as u32;
        let header = CapzHeader::with_hashes(backend, vk_hash, params_hash, payload_len);
        Self { header, payload }
    }

    /// Read container from bytes
    pub fn read<R: Read>(reader: &mut R) -> Result<Self> {
        let header = CapzHeader::read(reader)?;

        let mut payload = vec![0u8; header.payload_len as usize];
        reader.read_exact(&mut payload)?;

        Ok(Self { header, payload })
    }

    /// Write container to bytes
    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.header.write(writer)?;
        writer.write_all(&self.payload)?;
        Ok(())
    }

    /// Read from file
    pub fn read_from_file(path: &str) -> Result<Self> {
        let bytes = std::fs::read(path)?;
        let mut cursor = Cursor::new(bytes);
        Self::read(&mut cursor)
    }

    /// Write to file
    pub fn write_to_file(&self, path: &str) -> Result<()> {
        let mut file = std::fs::File::create(path)?;
        self.write(&mut file)
    }

    /// Get payload as string (assumes UTF-8 JSON)
    pub fn payload_as_string(&self) -> Result<String> {
        String::from_utf8(self.payload.clone())
            .map_err(|e| anyhow!("Payload is not valid UTF-8: {}", e))
    }

    /// Get total size (header + payload)
    pub fn total_size(&self) -> usize {
        CAPZ_HEADER_SIZE + self.payload.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_roundtrip() {
        let header = CapzHeader::new(ProofBackend::Mock, 1024);

        let mut buf = Vec::new();
        header.write(&mut buf).unwrap();

        assert_eq!(buf.len(), CAPZ_HEADER_SIZE);

        let mut cursor = Cursor::new(&buf);
        let parsed = CapzHeader::read(&mut cursor).unwrap();

        assert_eq!(parsed.version, CAPZ_VERSION);
        assert_eq!(parsed.backend, ProofBackend::Mock);
        assert_eq!(parsed.payload_len, 1024);
    }

    #[test]
    fn test_container_roundtrip() {
        let payload = b"test payload data".to_vec();
        let container = CapzContainer::new(ProofBackend::ZkVm, payload.clone());

        let mut buf = Vec::new();
        container.write(&mut buf).unwrap();

        let mut cursor = Cursor::new(&buf);
        let parsed = CapzContainer::read(&mut cursor).unwrap();

        assert_eq!(parsed.header.backend, ProofBackend::ZkVm);
        assert_eq!(parsed.payload, payload);
    }

    #[test]
    fn test_invalid_magic() {
        let mut buf = vec![0u8; CAPZ_HEADER_SIZE];
        buf[0..4].copy_from_slice(b"XXXX"); // Wrong magic

        let mut cursor = Cursor::new(&buf);
        let result = CapzHeader::read(&mut cursor);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid CAPZ magic"));
    }

    #[test]
    fn test_invalid_version() {
        let mut buf = vec![0u8; CAPZ_HEADER_SIZE];
        buf[0..4].copy_from_slice(CAPZ_MAGIC);
        buf[4..6].copy_from_slice(&0x9999u16.to_le_bytes()); // Wrong version

        let mut cursor = Cursor::new(&buf);
        let result = CapzHeader::read(&mut cursor);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unsupported CAPZ version"));
    }

    #[test]
    fn test_invalid_backend() {
        let mut buf = vec![0u8; CAPZ_HEADER_SIZE];
        buf[0..4].copy_from_slice(CAPZ_MAGIC);
        buf[4..6].copy_from_slice(&CAPZ_VERSION.to_le_bytes());
        buf[6] = 99; // Invalid backend

        let mut cursor = Cursor::new(&buf);
        let result = CapzHeader::read(&mut cursor);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid backend value"));
    }

    #[test]
    fn test_payload_too_large() {
        let mut buf = vec![0u8; CAPZ_HEADER_SIZE];
        buf[0..4].copy_from_slice(CAPZ_MAGIC);
        buf[4..6].copy_from_slice(&CAPZ_VERSION.to_le_bytes());
        buf[6] = ProofBackend::Mock as u8;
        buf[72..76].copy_from_slice(&200_000_000u32.to_le_bytes()); // Too large

        let mut cursor = Cursor::new(&buf);
        let result = CapzHeader::read(&mut cursor);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Payload length too large"));
    }

    #[test]
    fn test_backend_from_str() {
        assert_eq!(ProofBackend::Mock.as_str(), "mock");
        assert_eq!(ProofBackend::ZkVm.as_str(), "zkvm");
        assert_eq!(ProofBackend::Halo2.as_str(), "halo2");
    }

    #[test]
    fn test_container_with_hashes() {
        let vk_hash = [1u8; 32];
        let params_hash = [2u8; 32];
        let payload = b"test".to_vec();

        let container =
            CapzContainer::with_hashes(ProofBackend::Halo2, vk_hash, params_hash, payload.clone());

        assert_eq!(container.header.vk_hash, vk_hash);
        assert_eq!(container.header.params_hash, params_hash);
        assert_eq!(container.payload, payload);
    }
}
