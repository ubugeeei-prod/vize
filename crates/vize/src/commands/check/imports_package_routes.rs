use std::cmp::Ordering;

use vize_canon::{PackageResolutionContext, PackageRouteBinding};

pub(super) fn sort_package_route_bindings(bindings: &mut Vec<PackageRouteBinding>) {
    bindings.sort_by(compare_binding_keys);

    let mut deduped = Vec::with_capacity(bindings.len());
    let mut merged = false;
    for mut binding in bindings.drain(..) {
        if let Some(previous) = deduped.last_mut()
            && same_binding_key(previous, &binding)
        {
            previous
                .invalidation_paths
                .append(&mut binding.invalidation_paths);
            merged = true;
            if binding.route.is_some() {
                previous.route = binding.route;
            }
            continue;
        }
        deduped.push(binding);
    }
    if merged {
        for binding in &mut deduped {
            binding.invalidation_paths.sort();
            binding.invalidation_paths.dedup();
        }
    }
    *bindings = deduped;
}

fn compare_binding_keys(left: &PackageRouteBinding, right: &PackageRouteBinding) -> Ordering {
    (&left.importer_path, &left.specifier, left.occurrence_mode)
        .cmp(&(
            &right.importer_path,
            &right.specifier,
            right.occurrence_mode,
        ))
        .then_with(|| compare_contexts(&left.context, &right.context))
}

fn compare_contexts(left: &PackageResolutionContext, right: &PackageResolutionContext) -> Ordering {
    (
        &left.module_resolution,
        left.mode,
        &left.active_conditions,
        &left.scope_manifest_path,
    )
        .cmp(&(
            &right.module_resolution,
            right.mode,
            &right.active_conditions,
            &right.scope_manifest_path,
        ))
}

fn same_binding_key(left: &PackageRouteBinding, right: &PackageRouteBinding) -> bool {
    left.importer_path == right.importer_path
        && left.specifier == right.specifier
        && left.occurrence_mode == right.occurrence_mode
        && left.context == right.context
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use vize_canon::{PackageResolutionContext, PackageResolutionMode};

    use super::*;

    fn binding(
        mode: PackageResolutionMode,
        invalidation_paths: impl IntoIterator<Item = &'static str>,
    ) -> PackageRouteBinding {
        PackageRouteBinding {
            importer_path: PathBuf::from("/workspace/src/app.ts"),
            specifier: vize_s0::String::from("pkg"),
            occurrence_mode: mode,
            context: PackageResolutionContext::new(Some("bundler"), mode, ["import"]),
            route: None,
            invalidation_paths: invalidation_paths.into_iter().map(PathBuf::from).collect(),
        }
    }

    #[test]
    fn package_route_bindings_merge_by_importer_specifier_mode_and_context() {
        let mut bindings = vec![
            binding(
                PackageResolutionMode::Import,
                ["/workspace/pkg/package.json"],
            ),
            binding(PackageResolutionMode::Import, ["/workspace/pkg/index.d.ts"]),
            binding(
                PackageResolutionMode::Require,
                ["/workspace/pkg-cjs/package.json"],
            ),
        ];

        sort_package_route_bindings(&mut bindings);

        assert_eq!(bindings.len(), 2);
        assert_eq!(bindings[0].occurrence_mode, PackageResolutionMode::Import);
        assert_eq!(
            bindings[0].invalidation_paths,
            vec![
                PathBuf::from("/workspace/pkg/index.d.ts"),
                PathBuf::from("/workspace/pkg/package.json"),
            ]
        );
        assert_eq!(bindings[1].occurrence_mode, PackageResolutionMode::Require);
    }
}
