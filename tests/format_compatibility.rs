//! Verify the encoded layout matches the on-disk format produced by Firefox
//! and the original C reference tools (`mozLz40\0` + LE u32 size + raw LZ4
//! block). These tests intentionally rebuild the header byte-by-byte so a
//! regression in either component fails loudly.

use std::convert::TryInto;

#[test]
fn encoded_layout_is_magic_then_le_u32_size_then_body() {
    let payload = b"the quick brown fox jumps over the lazy dog".repeat(40);
    let encoded = jsonlz4::encode(&payload).unwrap();

    // Magic.
    assert_eq!(&encoded[..8], b"mozLz40\0");

    // Decompressed size, little-endian.
    let size_bytes: [u8; 4] = encoded[8..12].try_into().unwrap();
    assert_eq!(u32::from_le_bytes(size_bytes) as usize, payload.len());

    // Body must be a valid raw LZ4 block whose decompression equals the input.
    let body = &encoded[12..];
    let decoded = lz4_flex::block::decompress(body, payload.len()).unwrap();
    assert_eq!(decoded, payload);
}

#[test]
fn library_decode_matches_manual_reconstruction() {
    // Build a jsonlz4 file by hand from a freshly-compressed LZ4 block, then
    // ensure the library decodes it identically — proves the decoder doesn't
    // depend on a private quirk of our own encoder.
    let payload = b"manually-built header should also decode".to_vec();
    let body = lz4_flex::block::compress(&payload);

    let mut buf = Vec::new();
    buf.extend_from_slice(jsonlz4::MAGIC);
    buf.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    buf.extend_from_slice(&body);

    assert_eq!(jsonlz4::decode(&buf).unwrap(), payload);
}
