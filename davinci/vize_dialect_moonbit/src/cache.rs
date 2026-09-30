//! Bounded checker reuse. A toolchain change invalidates the same projection;
//! package and file names belong to the key because they affect diagnostics.

use crate::host::{CheckUnit, HostError, MooncHost, RawCheck};
use vize_l0::{String, ToCompactString};

const ENTRY_LIMIT: usize = 32;
const BYTE_LIMIT: usize = 4 * 1024 * 1024;

/// The complete inputs to one checker result, without a lossy digest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckKey {
    pub toolchain: String,
    pub package: String,
    pub file_name: String,
    pub source: String,
    pub environment: Option<String>,
    pub dependencies: String,
}

impl CheckKey {
    #[must_use]
    pub fn of(toolchain: &str, unit: &CheckUnit<'_>) -> Self {
        Self {
            toolchain: toolchain.to_compact_string(),
            package: unit.package.to_compact_string(),
            file_name: unit.file_name.to_compact_string(),
            source: unit.source.to_compact_string(),
            environment: unit.environment.map(ToCompactString::to_compact_string),
            dependencies: String::default(),
        }
    }

    fn bytes(&self) -> usize {
        self.toolchain.len()
            + self.package.len()
            + self.file_name.len()
            + self.source.len()
            + self
                .environment
                .as_ref()
                .map_or(0, |environment| environment.len())
            + self.dependencies.len()
    }
}

/// A session cache for checker facts and diagnostics. It keeps at most 32
/// results and 4 MiB of input/output text; failures are always retried.
#[derive(Debug)]
pub struct CachedMoonc<H> {
    host: H,
    entries: Vec<(CheckKey, RawCheck)>,
    bytes: usize,
}

impl<H> CachedMoonc<H> {
    #[must_use]
    pub const fn new(host: H) -> Self {
        Self {
            host,
            entries: Vec::new(),
            bytes: 0,
        }
    }

    /// The checker may be refreshed when the session's toolchain changes.
    pub fn inner_mut(&mut self) -> &mut H {
        &mut self.host
    }
}

impl<H: MooncHost> MooncHost for CachedMoonc<H> {
    fn toolchain(&self) -> &str {
        self.host.toolchain()
    }

    fn dependency_key(&self) -> Result<String, HostError> {
        self.host.dependency_key()
    }

    fn check(&mut self, unit: &CheckUnit<'_>) -> Result<RawCheck, HostError> {
        let mut key = CheckKey::of(self.toolchain(), unit);
        key.dependencies = self.host.dependency_key()?;
        if let Some((_, result)) = self.entries.iter().find(|(stored, _)| *stored == key) {
            return Ok(result.clone());
        }
        let result = self.host.check(unit)?;
        if result.toolchain != key.toolchain {
            return Err(HostError::Failed(
                "checker answer changed its toolchain during the request".into(),
            ));
        }
        if self.host.dependency_key()? != key.dependencies {
            return Err(HostError::Failed(
                "checker dependencies changed during the request".into(),
            ));
        }
        let result_bytes =
            result.lines.iter().map(|line| line.len()).sum::<usize>() + result.toolchain.len();
        let bytes = key.bytes() + result_bytes;
        if bytes <= BYTE_LIMIT {
            while !self.entries.is_empty()
                && (self.entries.len() >= ENTRY_LIMIT || self.bytes + bytes > BYTE_LIMIT)
            {
                let (old_key, old_result) = self.entries.remove(0);
                self.bytes -= old_key.bytes()
                    + old_result.toolchain.len()
                    + old_result
                        .lines
                        .iter()
                        .map(|line| line.len())
                        .sum::<usize>();
            }
            self.entries.push((key, result.clone()));
            self.bytes += bytes;
        }
        Ok(result)
    }
}
