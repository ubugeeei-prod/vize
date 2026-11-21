#![expect(
    clippy::unwrap_used,
    reason = "test hosts and packet serialization are asserted"
)]
use super::*;
use crate::source_project::SourceSnapshotCache;
use tower_lsp::lsp_types::Url;
use vize_canon::{SfcTypeCheckOptions, type_check_sfc};
use vize_l0::Allocator;
use vize_l1::embed::authored_program::AuthoredProgram;

#[test]
fn public_sfc_diagnostics_agree_for_both_parser_profiles_and_recovery() {
    let state = ServerState::new();
    let uri = Url::parse("file:///checker/Original.vue").unwrap();
    let mut errors = 0;
    for source in [
        "<script>const value: number = 1</script>",
        "<script setup>const invalid: = 2</script>",
        "<!-- 雪🌱 -->\r\n<script lang='tsx'>export const tree = <div/>;</script>\r\n<script setup lang='ts'>const value: = 2;</script>",
        "<script lang='ts'>import './leaf'; const incomplete = (</script>",
        "<script setup lang='jsx'>const x = <span/></script>",
        "<script lang=' tsx '>export default <span/></script>",
        "<script lang='coffee'>const value = 2</script>",
        "<script setup>export { value } from './leaf';</script>",
        "<template><div/></template>",
        "<!-- Ω🌱 -->\r\n<script\r\n lang='js'>export default {}</script>",
        "<template>🌱</template><script>export default {}</script>\r<script setup>import 'x'</script>",
        "<script lang='ts'/>\n<script setup lang='tsx'>const bad: = ;</script>",
    ] {
        state
            .documents
            .open(uri.clone(), source.into(), 2, "vue".into());
        let snapshot = SourceSnapshotCache::default()
            .capture(&state.documents, &uri)
            .unwrap();
        let (editor, actual) = snapshot
            .script_query_products_with_diagnostics(&state)
            .unwrap()
            .unwrap();
        assert_eq!(
            Some(editor),
            snapshot.script_query_products(&state).unwrap()
        );
        let original: Vec<_> = type_check_sfc(source, &SfcTypeCheckOptions::default())
            .diagnostics
            .into_iter()
            .filter(|error| error.code.as_deref() == Some("script-parse-error"))
            .collect();
        assert_eq!(
            serde_json::to_vec(&actual).unwrap(),
            serde_json::to_vec(&original).unwrap(),
            "{source}"
        );
        errors += actual.len();
    }
    assert!(errors >= 3);
}

#[test]
fn nested_and_static_module_candidates_keep_the_original_order_and_ranges() {
    let state = ServerState::new();
    let uri = Url::parse("file:///checker/Modules.vue").unwrap();
    for (lang, body, names) in [
        (
            "js",
            "import a from 'a'; export { x } from 'b'; export * from 'c';",
            vec!["a", "b", "c"],
        ),
        (
            "js",
            "const x = import /*comment*/ ('async'); export * from 'first';",
            vec!["first", "async"],
        ),
        ("js", r"const x = requ\u0069re('escaped');", vec!["escaped"]),
        (
            "js",
            "function read() { return (require)('wrapped') }",
            vec![],
        ),
        ("js", "require /*comment*/ ('wrapped')", vec!["wrapped"]),
        (
            "ts",
            "type Imported = import('types').Node; export {};",
            vec!["types"],
        ),
    ] {
        let source = vize_l0::cstr!("<script lang='{lang}'>{body}</script>");
        state
            .documents
            .open(uri.clone(), source.as_str().into(), 1, "vue".into());
        let snapshot = SourceSnapshotCache::default()
            .capture(&state.documents, &uri)
            .unwrap();
        let actual = snapshot
            .script_query_products_with_diagnostics(&state)
            .unwrap();
        same(actual.as_ref(), separate(&snapshot, &state).as_ref(), body);
        let (editor, _) = actual.unwrap();
        assert_eq!(
            editor
                .module_specifiers
                .iter()
                .map(|item| item.specifier.as_str())
                .collect::<Vec<_>>(),
            names
        );
    }
}

type Products = (ScriptQueryProducts, Vec<SfcTypeDiagnostic>);

// Both readers get the same warm descriptor, root and publication guard, even
// in the baseline. The editor already shares its parse for symbols and rename.
fn separate(snapshot: &SourceSnapshot, state: &ServerState) -> Option<Products> {
    let mut diagnostics = Vec::new();
    let products = snapshot
        .with_script_blocks(state, |out, root, name, block| {
            out.read_script(root, name, block, None, |_| Ok(()))?;
            let profile = ScriptDiagnosticProfile::for_language(block.lang.as_deref());
            let text = root.source().get(block.loc.start..block.loc.end).unwrap();
            let frame = root.block(text, block.loc.start as u32).unwrap();
            let arena = Allocator::default();
            let syntax = AuthoredProgram::parse(&arena, frame, profile.source_type());
            diagnostics.extend(profile.diagnostics(&syntax).unwrap());
            Ok(())
        })
        .unwrap()?;
    Some((products, diagnostics))
}

fn same(actual: Option<&Products>, original: Option<&Products>, name: &str) {
    assert_eq!(actual.map(|v| &v.0), original.map(|v| &v.0), "{name}");
    assert_eq!(
        serde_json::to_vec(&actual.map(|v| &v.1)).unwrap(),
        serde_json::to_vec(&original.map(|v| &v.1)).unwrap(),
        "{name}"
    );
}

fn for_originals(mut read: impl FnMut(usize, &str, &str)) -> usize {
    let manifest = std::fs::read(std::env::var_os("VIZE_SCRIPT_QUERY_INPUTS").unwrap()).unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut invalid = 0;
    for (i, entry) in manifest
        .split(|&byte| byte == 0)
        .filter(|p| !p.is_empty())
        .enumerate()
    {
        let name = std::str::from_utf8(entry).unwrap();
        let bytes = std::fs::read(root.join(name)).unwrap();
        match std::str::from_utf8(&bytes) {
            Ok(source) => read(i, name, source),
            Err(_) => invalid += 1,
        }
    }
    invalid
}

#[test]
#[ignore = "requires original Vue source manifest"]
fn complete_editor_and_checker_packets_match_separate_profile_reads() {
    let state = ServerState::new();
    let uri = Url::parse("file:///checker/Corpus.vue").unwrap();
    let (mut files, mut accepted, mut errors, mut shared, mut distinct) = (0, 0, 0, 0, 0);
    let invalid = for_originals(|_, name, source| {
        state
            .documents
            .open(uri.clone(), source.into(), 1, "vue".into());
        let snapshot = SourceSnapshotCache::default()
            .capture(&state.documents, &uri)
            .unwrap();
        let actual = snapshot
            .script_query_products_with_diagnostics(&state)
            .unwrap();
        same(actual.as_ref(), separate(&snapshot, &state).as_ref(), name);
        files += 1;
        if let Some((_, diagnostics)) = actual {
            accepted += 1;
            errors += diagnostics.len();
            let descriptor = state.sfc_descriptor(&uri, source).unwrap();
            for block in [descriptor.script.as_ref(), descriptor.script_setup.as_ref()]
                .into_iter()
                .flatten()
            {
                let lang = block.lang.as_deref();
                if ScriptQueryProducts::script_type(lang)
                    == ScriptDiagnosticProfile::for_language(lang).source_type()
                {
                    shared += 1;
                } else {
                    distinct += 1;
                }
            }
        }
    });
    assert!(accepted > 0 && shared > 0 && distinct > 0);
    eprintln!(
        "editor_checker files={files} accepted={accepted} shared={shared} distinct={distinct} diagnostics={errors} invalid={invalid}"
    );
}

#[test]
fn publication_after_the_checker_read_rechecks_the_root() {
    let state = ServerState::new();
    let uri = Url::parse("file:///checker/Old.vue").unwrap();
    let source = "<script setup lang='ts'>const old = 1</script>";
    state
        .documents
        .open(uri.clone(), source.into(), 1, "vue".into());
    let snapshot = SourceSnapshotCache::default()
        .capture(&state.documents, &uri)
        .unwrap();
    let result = snapshot.with_script_blocks(&state, |products, root, name, block| {
        let profile = ScriptDiagnosticProfile::for_language(block.lang.as_deref());
        products.read_script(root, name, block, Some(profile.source_type()), |syntax| {
            let _ = profile.diagnostics(syntax).unwrap();
            state
                .documents
                .open(uri.clone(), "<template/>".into(), 2, "vue".into());
            Ok(())
        })
    });
    assert_eq!(
        result,
        Err(ScriptQueryRefusal::Host(
            crate::source_project::SnapshotRefusal::Superseded
        ))
    );
}

#[cfg(feature = "micro-profiling")]
mod profile;
