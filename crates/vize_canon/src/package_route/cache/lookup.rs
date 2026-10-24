use std::path::PathBuf;

use super::super::PackageRoute;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PackageRouteLookup {
    pub(in crate::package_route) route: Option<PackageRoute>,
    pub(in crate::package_route) invalidation_paths: Vec<PathBuf>,
    pub(in crate::package_route) watchable_negative: bool,
}

impl PackageRouteLookup {
    pub fn into_parts(self) -> (Option<PackageRoute>, Vec<PathBuf>) {
        (self.route, self.invalidation_paths)
    }

    pub fn is_watchable_negative(&self) -> bool {
        self.route.is_none() && self.watchable_negative
    }
}
