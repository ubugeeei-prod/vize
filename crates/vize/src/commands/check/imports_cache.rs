//! Coherent package-resolution snapshots scoped to one import collection.

use std::ffi::OsString;
use std::path::PathBuf;

use vize_canon::{
    PackageResolutionContext, PackageResolutionMode, PackageRouteLookup, PackageSourceOptions,
};
use vize_s0::{FxHashMap, String};

type ResolutionContextKey = (PathBuf, Option<OsString>, PackageResolutionMode);
pub(super) type ResolutionContextCache =
    FxHashMap<ResolutionContextKey, (PackageResolutionContext, Vec<PathBuf>)>;

type PackageLookupKey = (
    PathBuf,
    String,
    PackageSourceOptions,
    PackageResolutionContext,
);
pub(super) type PackageLookupCache = FxHashMap<PackageLookupKey, PackageRouteLookup>;
