use super::{CorsaError, MAX_BIN_BYTES, MSG_CALL, MSG_RESPONSE, read_tuple, write_tuple};
use std::io::Cursor;

#[test]
fn roundtrips_small_tuple() {
    let mut buffer = Vec::new();
    write_tuple(&mut buffer, MSG_CALL, b"ping", br#"{"ok":true}"#).unwrap();
    let tuple = read_tuple(&mut Cursor::new(buffer)).unwrap();
    assert_eq!(tuple.kind, MSG_CALL);
    assert_eq!(tuple.method, b"ping");
    assert_eq!(tuple.payload, br#"{"ok":true}"#);
}

#[test]
fn reads_uint8_encoded_kind() {
    let bytes = [
        0x93_u8,
        0xcc,
        MSG_RESPONSE,
        0xc4,
        4,
        b'p',
        b'i',
        b'n',
        b'g',
        0xc4,
        0,
    ];
    let tuple = read_tuple(&mut Cursor::new(bytes)).unwrap();
    assert_eq!(tuple.kind, MSG_RESPONSE);
    assert_eq!(tuple.method, b"ping");
    assert!(tuple.payload.is_empty());
}

#[test]
fn rejects_non_tuple_marker() {
    let err = read_tuple(&mut Cursor::new([0x92_u8])).unwrap_err();
    assert!(
        matches!(err, CorsaError::Protocol(message) if message.contains("expected tuple marker"))
    );
}

#[test]
fn rejects_invalid_uint_marker() {
    let bytes = [0x93_u8, 0xcd, 0, 1, 0xc4, 0, 0xc4, 0];
    let err = read_tuple(&mut Cursor::new(bytes)).unwrap_err();
    assert!(
        matches!(err, CorsaError::Protocol(message) if message.contains("expected uint8 marker"))
    );
}

#[test]
fn rejects_invalid_bin_marker() {
    let bytes = [0x93_u8, MSG_CALL, 0xa1, b'x', 0xc4, 0];
    let err = read_tuple(&mut Cursor::new(bytes)).unwrap_err();
    assert!(
        matches!(err, CorsaError::Protocol(message) if message.contains("expected bin marker"))
    );
}

#[test]
fn rejects_oversized_bin_length_before_allocation() {
    let len = ((MAX_BIN_BYTES as u32) + 1).to_be_bytes();
    let bytes = [&[0x93_u8, MSG_CALL, 0xc4, 0, 0xc6][..], &len, &[][..]].concat();
    let err = read_tuple(&mut Cursor::new(bytes)).unwrap_err();
    assert!(matches!(err, CorsaError::Protocol(message) if message.contains("too large")));
}

#[test]
fn treats_truncated_tuple_as_closed_stdout() {
    let err = read_tuple(&mut Cursor::new([])).unwrap_err();
    assert!(matches!(err, CorsaError::Closed("msgpack stdout")));
}

#[test]
fn write_tuple_selects_bin_threshold_markers() {
    let mut bin8 = Vec::new();
    write_tuple(&mut bin8, MSG_CALL, &[b'a'; 255], b"").unwrap();
    assert_eq!(bin8[2], 0xc4);

    let mut bin16 = Vec::new();
    write_tuple(&mut bin16, MSG_CALL, &[b'a'; 256], b"").unwrap();
    assert_eq!(bin16[2], 0xc5);

    let mut bin32 = Vec::new();
    write_tuple(&mut bin32, MSG_CALL, b"m", &[b'b'; 65_536]).unwrap();
    let payload_marker = 5;
    assert_eq!(bin32[payload_marker], 0xc6);
}
