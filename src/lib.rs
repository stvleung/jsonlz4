//! Encode and decode Mozilla's `.jsonlz4` / `.mozlz4` container format.
//!
//! Firefox stores some on-disk artifacts (bookmark backups, session restore
//! data, search settings) in a non-standard layout that wraps a raw LZ4 block
//! with a fixed-size header. This crate reads and writes that container.
//!
//! # Format
//!
//! ```text
//! +--------------------+------------------------------+------------------------+
//! | 8 bytes: "mozLz40\0" | 4 bytes: u32 LE decomp size | N bytes: raw LZ4 block |
//! +--------------------+------------------------------+------------------------+
//! ```
//!
//! The trailing block is the LZ4 *block* format, not the LZ4 *frame* format
//! produced by the standard `lz4` CLI, so the two are not interchangeable.
//!
//! # Quick start
//!
//! ```
//! let original = b"{\"hello\": \"world\"}";
//! let encoded = jsonlz4::encode(original).unwrap();
//! let decoded = jsonlz4::decode(&encoded).unwrap();
//! assert_eq!(decoded, original);
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use thiserror::Error;

/// Magic bytes that prefix every jsonlz4 file: ASCII `mozLz40` followed by `\0`.
pub const MAGIC: &[u8; 8] = b"mozLz40\0";

/// Size in bytes of the fixed header (magic + uncompressed-size field).
pub const HEADER_SIZE: usize = MAGIC.len() + 4;

/// Errors returned when decoding a jsonlz4 stream.
#[derive(Debug, Error)]
pub enum DecodeError {
    /// Input is shorter than the 12-byte fixed header.
    #[error("input is too short to be a jsonlz4 file ({0} < {HEADER_SIZE} bytes)")]
    TooShort(usize),

    /// The 8-byte magic prefix did not match `mozLz40\0`.
    #[error("invalid magic bytes; not a jsonlz4 file")]
    BadMagic,

    /// The trailing LZ4 block could not be decompressed.
    #[error("lz4 decompression failed: {0}")]
    Lz4(#[from] lz4_flex::block::DecompressError),
}

/// Errors returned when encoding to a jsonlz4 stream.
#[derive(Debug, Error)]
pub enum EncodeError {
    /// Input exceeds the 4 GiB limit imposed by the 32-bit size field.
    #[error("input is too large for jsonlz4 ({size} bytes; max {})", u32::MAX)]
    TooLarge {
        /// Size of the input that was rejected.
        size: usize,
    },
}

/// Encode `input` as a jsonlz4 byte stream.
///
/// # Errors
///
/// Returns [`EncodeError::TooLarge`] if `input` is bigger than `u32::MAX`,
/// since the format records the uncompressed size in a 4-byte field.
pub fn encode(input: &[u8]) -> Result<Vec<u8>, EncodeError> {
    let uncompressed_len =
        u32::try_from(input.len()).map_err(|_| EncodeError::TooLarge { size: input.len() })?;

    let body = lz4_flex::block::compress(input);

    let mut out = Vec::with_capacity(HEADER_SIZE + body.len());
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&uncompressed_len.to_le_bytes());
    out.extend_from_slice(&body);
    Ok(out)
}

/// Decode a jsonlz4 byte stream back into its original payload.
///
/// The returned `Vec` is exactly the size the producer recorded in the
/// header. If the LZ4 block actually expands to fewer bytes, the result is
/// truncated to match what was decoded — same as the original C tool, which
/// merely warns on size mismatch.
///
/// # Errors
///
/// - [`DecodeError::TooShort`] if `input` is smaller than [`HEADER_SIZE`].
/// - [`DecodeError::BadMagic`] if the first 8 bytes are not [`MAGIC`].
/// - [`DecodeError::Lz4`] if the LZ4 block is malformed.
pub fn decode(input: &[u8]) -> Result<Vec<u8>, DecodeError> {
    let (declared_size, body) = parse_header(input)?;
    let mut out = lz4_flex::block::decompress(body, declared_size)?;
    // The C reference allocates `declared_size` bytes and decompresses into
    // it; if the LZ4 stream produces fewer bytes, the C tool prints a
    // warning. lz4_flex returns a `Vec` sized to actual output, which is
    // already the correct shape. Truncate just in case the implementation
    // ever over-allocates.
    out.truncate(declared_size);
    Ok(out)
}

/// Parse the fixed header and return `(declared_uncompressed_size, lz4_body)`.
///
/// Useful when streaming the LZ4 block elsewhere instead of buffering the
/// entire decoded payload.
///
/// # Errors
///
/// See [`decode`].
pub fn parse_header(input: &[u8]) -> Result<(usize, &[u8]), DecodeError> {
    if input.len() < HEADER_SIZE {
        return Err(DecodeError::TooShort(input.len()));
    }
    if &input[..MAGIC.len()] != MAGIC {
        return Err(DecodeError::BadMagic);
    }
    let size_bytes: [u8; 4] = input[MAGIC.len()..HEADER_SIZE].try_into().unwrap();
    let declared = u32::from_le_bytes(size_bytes) as usize;
    Ok((declared, &input[HEADER_SIZE..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_empty() {
        let encoded = encode(b"").unwrap();
        assert_eq!(&encoded[..MAGIC.len()], MAGIC);
        assert_eq!(&encoded[MAGIC.len()..HEADER_SIZE], &[0, 0, 0, 0]);
        assert_eq!(decode(&encoded).unwrap(), b"");
    }

    #[test]
    fn round_trip_small_text() {
        let payload = b"{\"hello\": \"world\"}";
        let encoded = encode(payload).unwrap();
        assert_eq!(decode(&encoded).unwrap(), payload);
    }

    #[test]
    fn round_trip_repetitive_payload_compresses() {
        // Highly-redundant input should fit in fewer bytes than it started.
        let payload = vec![b'a'; 64 * 1024];
        let encoded = encode(&payload).unwrap();
        assert!(
            encoded.len() < payload.len(),
            "repetitive input should compress; got {} >= {}",
            encoded.len(),
            payload.len()
        );
        assert_eq!(decode(&encoded).unwrap(), payload);
    }

    #[test]
    fn round_trip_random_like_payload() {
        // Pseudo-random data won't compress, but must still round-trip.
        let payload: Vec<u8> = (0..32_768)
            .map(|i| (i as u32).wrapping_mul(2_654_435_761) as u8)
            .collect();
        let encoded = encode(&payload).unwrap();
        assert_eq!(decode(&encoded).unwrap(), payload);
    }

    #[test]
    fn header_layout_is_little_endian() {
        let payload = vec![0u8; 0x0102_0304];
        let encoded = encode(&payload).unwrap();
        assert_eq!(
            &encoded[MAGIC.len()..HEADER_SIZE],
            &[0x04, 0x03, 0x02, 0x01]
        );
    }

    #[test]
    fn decode_rejects_too_short_input() {
        let err = decode(b"mozLz40").unwrap_err();
        assert!(matches!(err, DecodeError::TooShort(7)));
    }

    #[test]
    fn decode_rejects_bad_magic() {
        let mut buf = vec![0u8; HEADER_SIZE + 4];
        buf[..MAGIC.len()].copy_from_slice(b"NOPELZ4\0");
        let err = decode(&buf).unwrap_err();
        assert!(matches!(err, DecodeError::BadMagic));
    }

    #[test]
    fn decode_propagates_corrupt_block() {
        // Valid header, garbage body.
        let mut buf = Vec::from(*MAGIC);
        buf.extend_from_slice(&100u32.to_le_bytes());
        buf.extend_from_slice(&[0xFFu8; 16]);
        let err = decode(&buf).unwrap_err();
        assert!(matches!(err, DecodeError::Lz4(_)));
    }

    #[test]
    fn parse_header_returns_size_and_body() {
        let payload = b"hi there";
        let encoded = encode(payload).unwrap();
        let (size, body) = parse_header(&encoded).unwrap();
        assert_eq!(size, payload.len());
        assert_eq!(body, &encoded[HEADER_SIZE..]);
    }
}
