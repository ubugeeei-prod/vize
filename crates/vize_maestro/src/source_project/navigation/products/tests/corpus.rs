use super::*;
use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};
use vize_l1::{container::vue::DescriptorOptions, markup::SurfaceParseOptions};
use vize_l1_to_l2::native_file::{NativeReferenceRef, NativeSfc, lower_sfc_native};

pub(super) fn originals(mut read: impl FnMut(usize, &str, &str)) -> usize {
    let list = std::fs::read(std::env::var_os("VIZE_NATIVE_NAVIGATION_INPUTS").unwrap()).unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut invalid = 0;
    for (i, path) in list
        .split(|&b| b == 0)
        .filter(|s| !s.is_empty())
        .enumerate()
    {
        let name = std::str::from_utf8(path).unwrap();
        let bytes = std::fs::read(root.join(name)).unwrap();
        match std::str::from_utf8(&bytes) {
            Ok(text) => read(i, name, text),
            Err(_) => invalid += 1,
        }
    }
    invalid
}

pub(super) fn options() -> DescriptorOptions {
    DescriptorOptions {
        version: VueVersion::V3,
        dialect: VueDialect::Vue,
        template: SurfaceParseOptions::default(),
    }
}

pub(super) fn sites(native: &NativeSfc<'_, '_>) -> Vec<u32> {
    let file = native.file().l2ir();
    let mut scalar = vec![0, file.artifact().source().len() as u32];
    let mut add = |span: vize_l0::Span| scalar.extend([span.start, span.end]);
    for binding in file.bindings() {
        if let Some(row) = binding.declaration() {
            add(row.span);
        }
        let _ = native.for_each_reference_to(binding, |reference| add(reference.span()));
    }
    for row in file.references() {
        add(row.span);
    }
    scalar.sort_unstable();
    scalar.dedup();
    scalar
}

pub(super) fn position(source: &str, offset: u32) -> Position {
    let lines = vize_l0::line_index::LineBreaks::Lsp
        .line_starts(source)
        .collect::<Vec<_>>();
    crate::source_project::navigation::coordinates::range(
        source,
        &lines,
        vize_l0::Span::new(offset, offset),
    )
    .unwrap()
    .start
}

fn compare_native(native: &NativeSfc<'_, '_>, offset: u32, name: &str) {
    let info = |r: NativeReferenceRef<'_, '_>| {
        (
            r.binding().id(),
            r.scope(),
            r.span(),
            r.usage(),
            r.node(),
            r.script().map(|r| std::ptr::from_ref(r) as usize),
            r.template().map(|r| std::ptr::from_ref(r) as usize),
        )
    };
    let mut original = Vec::new();
    let expected = native.binding_at_offset(offset).and_then(|binding| {
        if let Some(binding) = binding {
            native.for_each_reference_to(binding, |r| original.push(info(r)))?;
        }
        Ok(binding.map(|b| b.id()))
    });
    let mut observed = Vec::new();
    let actual = native
        .for_each_reference_at_binding(offset, |r| observed.push(info(r)))
        .map(|b| b.map(|b| b.id()));
    assert_eq!(actual, expected, "{name} at {offset}");
    if actual.is_ok() {
        assert_eq!(observed, original, "{name} at {offset}");
    }
}

#[test]
#[ignore = "all genuine native Vue files and their original positions"]
fn original_file_membership_and_public_navigation_packets_match() {
    let state = Arc::new(ServerState::new());
    let uri = Url::parse("file:///navigation/Source.vue").unwrap();
    let (mut files, mut admitted, mut positions, mut resolved, mut bytes) = (0, 0, 0, 0, 0);
    let invalid = originals(|_, name, source| {
        files += 1;
        let arena = Allocator::default();
        let observation = lower_sfc_native(&arena, source, options());
        let Some(native) = observation.admitted() else {
            return;
        };
        admitted += 1;
        state
            .documents
            .open(uri.clone(), source.into(), 1, "vue".into());
        let project =
            NativeNavigationProject::new(SourceQueryProject::new_server(Arc::clone(&state)));
        for offset in sites(&native) {
            compare_native(&native, offset, name);
            let pos = position(source, offset);
            for include in [false, true] {
                let expected = separate(&project, &uri, pos, include);
                let actual = block_on(project.symbol_products(&uri, pos, include));
                assert_eq!(expected, actual, "{name} {pos:?}");
                positions += 1;
                if let Ok(products) = actual {
                    resolved += usize::from(products.definition.is_some());
                    bytes += serde_json::to_vec(&(
                        products.definition,
                        products.references,
                        products.highlights,
                    ))
                    .unwrap()
                    .len();
                }
            }
        }
        state.documents.close(&uri);
        project.notify_host_change(&uri);
    });
    assert!(files > 0 && admitted > 0 && positions > 0);
    eprintln!(
        "native_navigation files={files} admitted={admitted} requests={positions} resolved={resolved} bytes={bytes} invalid={invalid}"
    );
}
