//! Retain the original host Program and its region lowering for multiple consumers.
use super::{TsrxArtifacts, TsrxIssue, lower};
use alloc::vec::Vec;
use vize_l0::Allocator;
use vize_l1::markup::tsrx::{self, TsrxModule};

/// State of the original region producer at this module's revision.
pub enum RegionLowering<'a> {
    /// Read the syntax diagnostics on the original host module.
    Syntax,
    /// The module parsed and its regions refused lowering.
    Refused(Vec<TsrxIssue>),
    /// All admitted regions in authored order, including their host expressions.
    Ready(TsrxArtifacts<'a>),
}

/// The arena, host Program and canonical regions belong to one parse.
pub struct PreparedTsrx<'a> {
    arena: &'a Allocator,
    module: &'a TsrxModule<'a>,
    regions: RegionLowering<'a>,
}

impl<'a> PreparedTsrx<'a> {
    #[must_use]
    pub fn arena(&self) -> &'a Allocator {
        self.arena
    }

    #[must_use]
    pub fn module(&self) -> &'a TsrxModule<'a> {
        self.module
    }

    #[must_use]
    pub fn regions(&self) -> &RegionLowering<'a> {
        &self.regions
    }

    /// Transfer the sealed regions to a consuming transformation after any readers finish.
    #[must_use]
    pub fn into_regions(self) -> RegionLowering<'a> {
        self.regions
    }
}

/// Parse with the TSRX compiler's host options and lower a clean module once.
/// Readers may borrow the original Program and artifacts before one target
/// takes ownership. The returned product cannot retain a stack-owned module.
pub fn with_prepared<T>(source: &str, request: impl for<'a> FnOnce(PreparedTsrx<'a>) -> T) -> T {
    let arena = Allocator::new();
    let module = tsrx::parse(&arena, source);
    let regions = if module.is_clean() {
        match lower(&arena, &module) {
            Ok(artifacts) => RegionLowering::Ready(artifacts),
            Err(issues) => RegionLowering::Refused(issues),
        }
    } else {
        RegionLowering::Syntax
    };
    request(PreparedTsrx {
        arena: &arena,
        module: &module,
        regions,
    })
}
