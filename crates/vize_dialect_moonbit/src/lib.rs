//! The MoonBit expression dialect (Davinci phase 6, lane D).
//!
//! **Experimental — the P6-4a hosting spike.** Charter #4 puts foreign
//! expression languages inside Vue templates in scope; charter #28 makes
//! MoonBit the first, and the second implementation of the
//! `ExprRef` capability contract. This crate proves the hosting model end
//! to end on the contracts that exist today:
//!
//! 1. [`sfc::split`] — S0: a file is a MoonBit file when its script block
//!    says `lang="moonbit"`; the dialect resolves once per file.
//! 2. [`projection::project`] — S1 → S2 lowering as for any Vue template,
//!    every expression position re-read as `ExprRef::Foreign` and emitted
//!    by [`dialect::MoonBitDialect`] (the first
//!    `vize_s2::expr::capability::ExprDialect` implementor) into one
//!    virtual `.mbt` file with span links — charter #14's virtual
//!    host-language projection.
//! 3. [`host::MooncHost`] — the toolchain boundary. The `moonc` feature
//!    adds `native::NativeMoonc`, the P6-4a hosting choice: the pinned
//!    native `moonc`, the virtual file passed on its command line.
//! 4. [`diagnostic`] and [`render`] — `moonc`'s diagnostics mapped back
//!    through the links to authored, file-absolute spans.
//!
//! What P6-4b replaces: the S2 lowering constructing `Foreign` directly
//! (instead of the re-read), a generated `.mbti` environment from S2
//! scope facts (instead of the script block verbatim), the P6-1b
//! expression world as the transport, and the toolchain version in the
//! fact cache key.

pub mod diagnostic;
pub mod dialect;
pub mod host;
pub mod lines;
#[cfg(feature = "moonc")]
pub mod native;
pub mod projection;
pub mod render;
pub mod sfc;

use core::fmt;

use vize_s0::{Allocator, String};

use crate::diagnostic::{Mapped, ParseError, map_all};
use crate::host::{CheckUnit, HostError, MooncHost};
use crate::projection::{PACKAGE, Projection, project};
use crate::sfc::SfcError;

/// One SFC, projected and checked.
#[derive(Debug)]
pub struct Checked<'a> {
    /// The virtual MoonBit file and its links.
    pub projection: Projection<'a>,
    /// The toolchain version that answered.
    pub toolchain: String,
    /// `moonc`'s diagnostics, mapped, in emission order.
    pub diagnostics: Vec<Mapped>,
}

/// Why an SFC could not be checked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The file is not a MoonBit SFC.
    Sfc(SfcError),
    /// The checker produced no answer.
    Host(HostError),
    /// The checker answered something that is not a diagnostic.
    Parse(ParseError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sfc(error) => error.fmt(f),
            Self::Host(error) => error.fmt(f),
            Self::Parse(error) => error.fmt(f),
        }
    }
}

/// Project `source` (an SFC named `file_name`) and check it with `host`.
///
/// The host is a generic parameter: the dialect binds one checker per
/// session, statically, like the expression capability contract itself.
///
/// # Errors
///
/// [`Error`] when the file is not a MoonBit SFC or the host gives no
/// readable answer; type errors are diagnostics, not errors.
pub fn check<'a, H: MooncHost>(
    allocator: &'a Allocator,
    source: &'a str,
    file_name: &str,
    host: &mut H,
) -> Result<Checked<'a>, Error> {
    let sfc = sfc::split(source).map_err(Error::Sfc)?;
    let projection = project(allocator, &sfc, file_name);
    let raw = host
        .check(&CheckUnit {
            package: PACKAGE,
            file_name: &projection.file_name,
            source: &projection.text,
        })
        .map_err(Error::Host)?;
    let diagnostics = map_all(&projection, &raw.lines).map_err(Error::Parse)?;
    Ok(Checked {
        projection,
        toolchain: raw.toolchain,
        diagnostics,
    })
}
