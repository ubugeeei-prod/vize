use crate::{CorsaError, Result};
use corsa_core::fast::compact_format;
use std::io::{ErrorKind, Read, Write};

pub(crate) const MSG_REQUEST: u8 = 1;
pub(crate) const MSG_CALL_RESPONSE: u8 = 2;
pub(crate) const MSG_CALL_ERROR: u8 = 3;
pub(crate) const MSG_RESPONSE: u8 = 4;
pub(crate) const MSG_ERROR: u8 = 5;
pub(crate) const MSG_CALL: u8 = 6;
const MAX_BIN_BYTES: usize = 512 * 1024 * 1024;

#[derive(Debug)]
pub(crate) struct MsgpackTuple {
    pub kind: u8,
    pub method: Vec<u8>,
    pub payload: Vec<u8>,
}

pub(crate) fn read_tuple<R: Read>(reader: &mut R) -> Result<MsgpackTuple> {
    let mut tag = [0_u8; 1];
    read_exact(reader, &mut tag)?;
    if tag[0] != 0x93 {
        return Err(CorsaError::Protocol(compact_format(format_args!(
            "expected tuple marker, got {:x}",
            tag[0]
        ))));
    }
    let kind = read_int(reader)?;
    let method = read_bin(reader)?;
    let payload = read_bin(reader)?;
    Ok(MsgpackTuple {
        kind,
        method,
        payload,
    })
}

pub(crate) fn write_tuple<W: Write>(
    writer: &mut W,
    kind: u8,
    method: &[u8],
    payload: &[u8],
) -> Result<()> {
    write_all(writer, &[0x93, kind])?;
    write_bin(writer, method)?;
    write_bin(writer, payload)?;
    flush(writer)?;
    Ok(())
}

fn read_int<R: Read>(reader: &mut R) -> Result<u8> {
    let mut buf = [0_u8; 1];
    read_exact(reader, &mut buf)?;
    if buf[0] <= 0x7f {
        return Ok(buf[0]);
    }
    if buf[0] != 0xcc {
        return Err(CorsaError::Protocol(compact_format(format_args!(
            "expected uint8 marker, got {:x}",
            buf[0]
        ))));
    }
    read_exact(reader, &mut buf)?;
    Ok(buf[0])
}

fn read_bin<R: Read>(reader: &mut R) -> Result<Vec<u8>> {
    let mut tag = [0_u8; 1];
    read_exact(reader, &mut tag)?;
    let len = match tag[0] {
        0xc4 => read_len::<1, _>(reader)?,
        0xc5 => read_len::<2, _>(reader)?,
        0xc6 => read_len::<4, _>(reader)?,
        other => {
            return Err(CorsaError::Protocol(compact_format(format_args!(
                "expected bin marker, got {:x}",
                other
            ))));
        }
    };
    if len > MAX_BIN_BYTES {
        return Err(CorsaError::Protocol(compact_format(format_args!(
            "msgpack bin is too large: {len} bytes exceeds {MAX_BIN_BYTES} byte safety limit"
        ))));
    }
    let mut buf = Vec::new();
    buf.try_reserve_exact(len).map_err(|err| {
        CorsaError::Protocol(compact_format(format_args!(
            "failed to reserve msgpack bin buffer for {len} bytes: {err}"
        )))
    })?;
    buf.resize(len, 0);
    read_exact(reader, &mut buf)?;
    Ok(buf)
}

fn read_len<const N: usize, R: Read>(reader: &mut R) -> Result<usize> {
    match N {
        1 => {
            let mut buf = [0_u8; 1];
            read_exact(reader, &mut buf)?;
            Ok(buf[0] as usize)
        }
        2 => {
            let mut buf = [0_u8; 2];
            read_exact(reader, &mut buf)?;
            Ok(u16::from_be_bytes(buf) as usize)
        }
        4 => {
            let mut buf = [0_u8; 4];
            read_exact(reader, &mut buf)?;
            Ok(u32::from_be_bytes(buf) as usize)
        }
        _ => Err(CorsaError::Protocol(compact_format(format_args!(
            "unsupported msgpack length width: {N}"
        )))),
    }
}

fn write_bin<W: Write>(writer: &mut W, bytes: &[u8]) -> Result<()> {
    if bytes.len() > MAX_BIN_BYTES {
        return Err(CorsaError::Protocol(compact_format(format_args!(
            "msgpack bin is too large: {} bytes exceeds {MAX_BIN_BYTES} byte safety limit",
            bytes.len()
        ))));
    }
    match bytes.len() {
        0..=255 => write_all(writer, &[0xc4, bytes.len() as u8])?,
        256..=65535 => {
            write_all(writer, &[0xc5])?;
            write_all(writer, &(bytes.len() as u16).to_be_bytes())?;
        }
        _ => {
            write_all(writer, &[0xc6])?;
            write_all(writer, &(bytes.len() as u32).to_be_bytes())?;
        }
    }
    write_all(writer, bytes)?;
    Ok(())
}

fn read_exact<R: Read>(reader: &mut R, buf: &mut [u8]) -> Result<()> {
    reader.read_exact(buf).map_err(|error| match error.kind() {
        ErrorKind::UnexpectedEof => CorsaError::Closed("msgpack stdout"),
        _ => CorsaError::Io(error),
    })
}

fn write_all<W: Write>(writer: &mut W, buf: &[u8]) -> Result<()> {
    writer.write_all(buf).map_err(map_write_error)
}

fn flush<W: Write>(writer: &mut W) -> Result<()> {
    writer.flush().map_err(map_write_error)
}

fn map_write_error(error: std::io::Error) -> CorsaError {
    match error.kind() {
        ErrorKind::BrokenPipe | ErrorKind::ConnectionReset => CorsaError::Closed("msgpack stdin"),
        _ => CorsaError::Io(error),
    }
}

#[cfg(test)]
#[path = "msgpack_codec_tests.rs"]
mod tests;
