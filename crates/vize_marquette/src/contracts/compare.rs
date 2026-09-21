//! Additive/breaking classification between two contract surfaces.
//!
//! Compatibility is judged from the guest's side: a change is **additive**
//! when every guest built against the older version keeps working with a host
//! at the newer version, and **breaking** otherwise. Where the direction of a
//! value is not modelled (enum and variant cases, record fields), any change
//! to an existing type is breaking: the classifier may over-report, never
//! under-report.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use vize_s0::String;

use super::{ContractSurface, InterfaceSurface};
use crate::{CompatibilityChange, CompatibilityChangeKind};

/// Every change between two surfaces, in path order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContractSurfaceReport {
    pub changes: Vec<CompatibilityChange>,
}

impl ContractSurfaceReport {
    /// Whether any change breaks a guest built against the older surface.
    #[must_use]
    pub fn is_breaking(&self) -> bool {
        self.changes
            .iter()
            .any(|change| change.kind == CompatibilityChangeKind::Breaking)
    }
}

struct Changes(Vec<CompatibilityChange>);

/// `path` extended by one segment.
fn at<'a>(path: &[&'a str], key: &'a str) -> Vec<&'a str> {
    let mut extended = path.to_vec();
    extended.push(key);
    extended
}

impl Changes {
    fn push(&mut self, kind: CompatibilityChangeKind, path: &[&str], message: &str) {
        self.0.push(CompatibilityChange {
            kind,
            path: String::from(path.join(".")),
            message: String::from(message),
        });
    }

    fn breaking(&mut self, path: &[&str], message: &str) {
        self.push(CompatibilityChangeKind::Breaking, path, message);
    }

    /// Removed keys are breaking; added keys take `added`'s classification.
    fn keys<V>(
        &mut self,
        path: &[&str],
        old: &BTreeMap<String, V>,
        new: &BTreeMap<String, V>,
        added: CompatibilityChangeKind,
        added_message: &str,
    ) {
        for key in old.keys().filter(|key| !new.contains_key(*key)) {
            self.breaking(&at(path, key), "removed");
        }
        for key in new.keys().filter(|key| !old.contains_key(*key)) {
            self.push(added, &at(path, key), added_message);
        }
    }

    fn sets(
        &mut self,
        path: &[&str],
        old: &BTreeSet<String>,
        new: &BTreeSet<String>,
        (added, added_message): (CompatibilityChangeKind, &str),
        (removed, removed_message): (CompatibilityChangeKind, &str),
    ) {
        for key in old.difference(new) {
            self.push(removed, &at(path, key), removed_message);
        }
        for key in new.difference(old) {
            self.push(added, &at(path, key), added_message);
        }
    }
}

/// Classify every change from `previous` to `next`.
#[must_use]
pub fn compare_surfaces(
    previous: &ContractSurface,
    next: &ContractSurface,
) -> ContractSurfaceReport {
    use CompatibilityChangeKind::{Additive, Breaking};

    let mut changes = Changes(Vec::new());
    if previous.package != next.package {
        changes.breaking(&["package"], "the package identity changed");
    }
    if previous.protocol_version != next.protocol_version {
        changes.breaking(
            &["protocolVersion"],
            "the handshake protocol version changed",
        );
    }
    changes.keys(
        &["pages"],
        &previous.pages,
        &next.pages,
        Additive,
        "a page kind was added",
    );
    for (page, schema) in &previous.pages {
        if next.pages.get(page).is_some_and(|next| next != schema) {
            changes.breaking(&["pages", page], "the page schema version changed");
        }
    }

    let exported = |surface: &ContractSurface| -> BTreeSet<String> {
        surface
            .worlds
            .values()
            .flat_map(|world| world.exports.iter().cloned())
            .collect()
    };
    let guest_implemented = &exported(previous) | &exported(next);
    changes.keys(
        &["interfaces"],
        &previous.interfaces,
        &next.interfaces,
        Additive,
        "an interface was added",
    );
    for (name, old) in &previous.interfaces {
        if let Some(new) = next.interfaces.get(name) {
            interface(
                &mut changes,
                name,
                old,
                new,
                guest_implemented.contains(name),
            );
        }
    }

    changes.keys(
        &["worlds"],
        &previous.worlds,
        &next.worlds,
        Additive,
        "a world was added",
    );
    for (name, old) in &previous.worlds {
        let Some(new) = next.worlds.get(name) else {
            continue;
        };
        let path = ["worlds", name.as_str()];
        changes.sets(
            &at(&path, "exports"),
            &old.exports,
            &new.exports,
            (
                Breaking,
                "guests built against the older version do not export it",
            ),
            (Breaking, "the host no longer calls it"),
        );
        changes.sets(
            &at(&path, "imports"),
            &old.imports,
            &new.imports,
            (Additive, "the host provides more"),
            (Breaking, "guests importing it no longer instantiate"),
        );
        changes.sets(
            &at(&path, "requiredFeatures"),
            &old.required_features,
            &new.required_features,
            (
                Breaking,
                "guests built against the older version do not offer it",
            ),
            (Additive, "the host requires less"),
        );
    }

    let mut changes = changes.0;
    changes.sort_by(|left, right| {
        (&left.path, left.kind, &left.message).cmp(&(&right.path, right.kind, &right.message))
    });
    ContractSurfaceReport { changes }
}

fn interface(
    changes: &mut Changes,
    name: &str,
    old: &InterfaceSurface,
    new: &InterfaceSurface,
    guest_implemented: bool,
) {
    let types = ["interfaces", name, "types"];
    changes.keys(
        &types,
        &old.types,
        &new.types,
        CompatibilityChangeKind::Additive,
        "a type was added",
    );
    for (ty, shape) in &old.types {
        if new.types.get(ty).is_some_and(|next| next != shape) {
            changes.breaking(&at(&types, ty), "the type's shape changed");
        }
    }
    let functions = ["interfaces", name, "functions"];
    let (added, message) = if guest_implemented {
        (
            CompatibilityChangeKind::Breaking,
            "guests built against the older version do not implement it",
        )
    } else {
        (CompatibilityChangeKind::Additive, "the host provides more")
    };
    changes.keys(&functions, &old.functions, &new.functions, added, message);
    for (function, signature) in &old.functions {
        if new
            .functions
            .get(function)
            .is_some_and(|next| next != signature)
        {
            changes.breaking(&at(&functions, function), "the signature changed");
        }
    }
}
