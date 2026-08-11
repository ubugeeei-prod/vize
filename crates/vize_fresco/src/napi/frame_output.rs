//! JavaScript contract for measured terminal frame output.

use napi::bindgen_prelude::BigInt;
use napi_derive::napi;

/// Exact output cost of one NAPI-driven terminal frame.
#[napi(object)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameOutputTelemetryNapi {
    /// Cells differing from the previous frame, including wide continuations.
    #[napi(js_name = "changedCells")]
    pub changed_cells: BigInt,
    /// Bytes accepted by the configured terminal writer.
    #[napi(js_name = "bytesWritten")]
    pub bytes_written: BigInt,
}

impl From<crate::terminal::FrameOutputTelemetry> for FrameOutputTelemetryNapi {
    fn from(telemetry: crate::terminal::FrameOutputTelemetry) -> Self {
        Self {
            changed_cells: telemetry.changed_cells().into(),
            bytes_written: telemetry.bytes_written().into(),
        }
    }
}
