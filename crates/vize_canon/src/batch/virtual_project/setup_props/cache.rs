#![expect(
    clippy::disallowed_types,
    reason = "walk-owned immutable source snapshots share parsed imports across roots"
)]

use std::path::Path;
use std::sync::{Arc, Mutex};

use vize_atelier_sfc::script::{SfcTypeProject, TypeSourceSnapshot};
use vize_carton::{CompactString, FxHashMap, FxHashSet};
use vize_croquis::macros::PropDefinition;

use super::super::passthrough::PassthroughProbes;
use super::syntax::runtime_prop_shape_member_type;

#[derive(Default)]
pub(crate) struct RuntimePropResolveCache {
    // One immutable dependency snapshot per register_paths batch or reachable
    // walk. Root text comes from the current descriptor; later scans start fresh.
    pub(super) type_sources: Arc<TypeSourceSnapshot>,
    // The same batch's directory listings, shared across its roots.
    passthrough_probes: Arc<PassthroughProbes>,
    props: Mutex<FxHashMap<CompactString, Vec<PropDefinition>>>,
    default_names: Mutex<FxHashMap<CompactString, FxHashSet<CompactString>>>,
}

impl RuntimePropResolveCache {
    /// A fresh walk-owned snapshot that resolves bare specifiers (`paths`,
    /// `baseUrl`, package conditions) through the program's own tsconfig.
    ///
    /// Without a bound project the snapshot discovers the nearest
    /// `tsconfig.json` per importer. A solution-style config (`files: []`
    /// plus `references`, as reka-ui ships) declares no `paths`, so imported
    /// prop types behind an alias such as `@/Primitive` resolved to nothing
    /// while TypeScript, driven by the selected tsconfig, resolved them.
    pub(in crate::batch::virtual_project) fn for_tsconfig(tsconfig: Option<&Path>) -> Self {
        let Some(project) = tsconfig.and_then(|path| SfcTypeProject::from_tsconfig(path).ok())
        else {
            return Self::default();
        };
        Self::with_type_sources(Arc::new(TypeSourceSnapshot::with_project(
            std::iter::empty(),
            project,
        )))
    }

    /// A per-root cache over a dependency snapshot owned elsewhere.
    pub(in crate::batch::virtual_project) fn with_type_sources(
        type_sources: Arc<TypeSourceSnapshot>,
    ) -> Self {
        Self {
            type_sources,
            passthrough_probes: Arc::default(),
            props: Mutex::default(),
            default_names: Mutex::default(),
        }
    }

    /// Keep source reads consistent within a walk, but resolve each root's
    /// runtime exports independently: cycle cuts depend on that root.
    pub(in crate::batch::virtual_project) fn with_shared_type_sources(&self) -> Self {
        Self {
            type_sources: self.type_sources.clone(),
            passthrough_probes: self.passthrough_probes.clone(),
            props: Mutex::default(),
            default_names: Mutex::default(),
        }
    }

    pub(in crate::batch::virtual_project) fn passthrough_probes(&self) -> &PassthroughProbes {
        &self.passthrough_probes
    }

    pub(super) fn props(
        &self,
        key: &CompactString,
        root_runtime_binding: &str,
    ) -> Option<Vec<PropDefinition>> {
        let props = self.props.lock().ok()?.get(key).cloned()?;
        Some(attach_runtime_prop_types(props, root_runtime_binding))
    }

    pub(super) fn insert_props(&self, key: CompactString, props: Vec<PropDefinition>) {
        if let Ok(mut cache) = self.props.lock() {
            cache.insert(key, detach_runtime_prop_types(props));
        }
    }

    pub(super) fn default_names(&self, key: &CompactString) -> Option<FxHashSet<CompactString>> {
        self.default_names.lock().ok()?.get(key).cloned()
    }

    pub(super) fn insert_default_names(&self, key: CompactString, names: FxHashSet<CompactString>) {
        if let Ok(mut cache) = self.default_names.lock() {
            cache.insert(key, names);
        }
    }
}

fn detach_runtime_prop_types(props: Vec<PropDefinition>) -> Vec<PropDefinition> {
    props
        .into_iter()
        .map(|mut prop| {
            prop.prop_type = None;
            prop
        })
        .collect()
}

fn attach_runtime_prop_types(
    props: Vec<PropDefinition>,
    root_runtime_binding: &str,
) -> Vec<PropDefinition> {
    props
        .into_iter()
        .map(|mut prop| {
            prop.prop_type =
                runtime_prop_shape_member_type(root_runtime_binding, prop.name.as_str());
            prop
        })
        .collect()
}
