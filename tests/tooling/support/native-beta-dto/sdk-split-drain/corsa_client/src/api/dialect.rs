//! Wire dialects spoken by Corsa runtimes.
//!
//! Upstream does not version its stdio API, and it reshapes it between
//! TypeScript releases. This module names the generations `corsa-bind` knows
//! how to talk to and detects which one a runtime speaks, so everything above
//! the client keeps one request/response vocabulary regardless of the runtime
//! it was pointed at.

use serde::Deserialize;
use std::sync::atomic::{AtomicU8, Ordering};

use super::responses::InitializeResponse;
use crate::{CorsaError, Result};

/// Wire dialect spoken by a connected Corsa runtime.
///
/// The dialect is detected from the `initialize` response, so it is only known
/// once [`ApiClient::initialize`](crate::ApiClient::initialize) has completed.
/// Every dialect is adapted to the same public request and response types;
/// this value exists for diagnostics and for callers that reach the raw
/// upstream-shaped endpoints and therefore need to know which shape to send.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ApiDialect {
    /// TypeScript 7.0.
    ///
    /// One implicit snapshot lineage per connection: `updateSnapshot` takes the
    /// changes directly and the server remembers which projects are open.
    /// Symbols are identified by a bare numeric id.
    SessionSnapshots,
    /// TypeScript 7.1 beta and development builds.
    ///
    /// Snapshots are independent values: `createSnapshot` starts one and
    /// `updateSnapshot` derives a new one from an explicit base. Symbols are
    /// identified by a reference that names the source file or snapshot that
    /// owns them, and filesystem callbacks answer with tagged results.
    DerivedSnapshots,
}

impl ApiDialect {
    /// Returns a stable lowercase label suitable for logs and reports.
    ///
    /// # Examples
    ///
    /// ```
    /// use corsa_client::ApiDialect;
    ///
    /// assert_eq!(ApiDialect::SessionSnapshots.as_str(), "session-snapshots");
    /// assert_eq!(ApiDialect::DerivedSnapshots.as_str(), "derived-snapshots");
    /// ```
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SessionSnapshots => "session-snapshots",
            Self::DerivedSnapshots => "derived-snapshots",
        }
    }
}

const SESSION_SNAPSHOTS: u8 = 1;
const DERIVED_SNAPSHOTS: u8 = 2;

/// Dialect of one connection, shared by every layer that has to encode for it.
///
/// Filesystem callbacks are wired up before the handshake runs, so they hold
/// this cell rather than a resolved [`ApiDialect`].
#[derive(Debug, Default)]
pub(crate) struct DialectCell(AtomicU8);

impl DialectCell {
    pub(crate) fn get(&self) -> Option<ApiDialect> {
        match self.0.load(Ordering::Acquire) {
            SESSION_SNAPSHOTS => Some(ApiDialect::SessionSnapshots),
            DERIVED_SNAPSHOTS => Some(ApiDialect::DerivedSnapshots),
            _ => None,
        }
    }

    pub(crate) fn set(&self, dialect: ApiDialect) {
        let value = match dialect {
            ApiDialect::SessionSnapshots => SESSION_SNAPSHOTS,
            ApiDialect::DerivedSnapshots => DERIVED_SNAPSHOTS,
        };
        self.0.store(value, Ordering::Release);
    }

    pub(crate) fn is_derived_snapshots(&self) -> bool {
        self.0.load(Ordering::Acquire) == DERIVED_SNAPSHOTS
    }
}

/// `initialize` response as either dialect puts it on the wire.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct InitializeWire {
    #[serde(default)]
    use_case_sensitive_file_names: Option<bool>,
    /// `0` case-insensitive, `1` case-sensitive.
    #[serde(default)]
    case_sensitivity: Option<u8>,
    current_directory: String,
}

impl InitializeWire {
    /// Splits the wire response into the public response and its dialect.
    ///
    /// The two dialects are told apart by which case-sensitivity field the
    /// runtime sent, which is the first thing either of them says.
    pub(crate) fn into_response(self) -> Result<(InitializeResponse, ApiDialect)> {
        let (use_case_sensitive_file_names, dialect) = match (
            self.case_sensitivity,
            self.use_case_sensitive_file_names,
        ) {
            (Some(case_sensitivity), _) => (case_sensitivity != 0, ApiDialect::DerivedSnapshots),
            (None, Some(sensitive)) => (sensitive, ApiDialect::SessionSnapshots),
            (None, None) => {
                return Err(CorsaError::Protocol(
                        "initialize response reported neither `caseSensitivity` nor `useCaseSensitiveFileNames`"
                            .into(),
                    ));
            }
        };
        Ok((
            InitializeResponse {
                use_case_sensitive_file_names,
                current_directory: self.current_directory,
            },
            dialect,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{ApiDialect, DialectCell, InitializeWire};
    use serde_json::json;

    fn decode(value: serde_json::Value) -> crate::Result<(bool, ApiDialect)> {
        let wire: InitializeWire = serde_json::from_value(value).unwrap();
        wire.into_response()
            .map(|(response, dialect)| (response.use_case_sensitive_file_names, dialect))
    }

    #[test]
    fn typescript_7_0_initialize_selects_session_snapshots() {
        let decoded = decode(json!({
            "useCaseSensitiveFileNames": true,
            "currentDirectory": "/workspace",
        }))
        .unwrap();

        assert_eq!(decoded, (true, ApiDialect::SessionSnapshots));
    }

    #[test]
    fn typescript_7_1_initialize_selects_derived_snapshots() {
        for (case_sensitivity, expected) in [(0, false), (1, true)] {
            let decoded = decode(json!({
                "caseSensitivity": case_sensitivity,
                "currentDirectory": "/workspace",
            }))
            .unwrap();

            assert_eq!(decoded, (expected, ApiDialect::DerivedSnapshots));
        }
    }

    #[test]
    fn initialize_without_case_sensitivity_is_a_protocol_error() {
        let error = decode(json!({ "currentDirectory": "/workspace" })).unwrap_err();

        assert!(error.to_string().contains("caseSensitivity"), "{error}");
    }

    #[test]
    fn dialect_cell_starts_unknown_and_remembers_the_handshake() {
        let cell = DialectCell::default();
        assert_eq!(cell.get(), None);
        assert!(!cell.is_derived_snapshots());

        cell.set(ApiDialect::DerivedSnapshots);
        assert_eq!(cell.get(), Some(ApiDialect::DerivedSnapshots));
        assert!(cell.is_derived_snapshots());
    }
}
