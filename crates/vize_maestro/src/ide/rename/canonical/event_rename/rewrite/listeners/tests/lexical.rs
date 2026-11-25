//! Exercise the same public rename path on files whose handlers are lexical closures.
use super::*;
use vize_l1_to_l2::native_file::lower_sfc_with_lexical_initializers;
use vize_l2::file::{DeclarationKind, InitializerKind};

#[test]
#[ignore = "complete edits and resident per-file measurements on original Vue callers"]
fn original_lexical_listener_edits() {
    let list = std::fs::read_to_string(std::env::var_os("VIZE_EXPRESSION_QUERY_MANIFEST").unwrap())
        .unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let (mut admitted, mut event_files, mut edits) = (0, 0, 0);
    #[cfg(feature = "micro-profiling")]
    davinci_harness::alloc::mark_installed();
    for name in list.split_terminator('\0') {
        let bytes = std::fs::read(root.join(name)).unwrap();
        let Ok(source) = std::str::from_utf8(&bytes) else {
            continue;
        };
        if !source.contains("=>") {
            continue;
        }
        document(source, |ctx| {
            let arena = Allocator::new();
            let owner = lower_sfc_with_lexical_initializers(
                &arena,
                &ctx.content,
                DescriptorOptions {
                    version: VueVersion::V3,
                    dialect: VueDialect::Vue,
                    template: Default::default(),
                },
            );
            let Ok(imports) = owner.setup_component_imports() else {
                return;
            };
            let file = owner
                .file()
                .map(|f| f.l2ir())
                .or_else(|| owner.rejected_file()?.file())
                .unwrap();
            if !file.bindings().filter_map(|b| b.declaration()).any(|d| {
                d.initializer == InitializerKind::Function
                    && matches!(
                        d.kind,
                        DeclarationKind::Const | DeclarationKind::Let | DeclarationKind::Var
                    )
            }) {
                return;
            }
            let result = compare(ctx, &imports);
            admitted += 1;
            event_files += usize::from(result.len() > 1);
            edits += result.len();
            #[cfg(feature = "micro-profiling")]
            profile::sample(name, ctx, &imports);
        });
    }
    if std::env::var_os("VIZE_REQUIRE_REPOSITORY_ARROWS").is_some() {
        assert!(admitted >= 23 && event_files > 0);
    }
    eprintln!("lexical_event_rewrites admitted={admitted} event_files={event_files} edits={edits}");
}
