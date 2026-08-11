use oxc_span::SourceType;

use super::super::ImportRewriter;

#[test]
fn keeps_bare_vue_package_imports_for_native_resolution() {
    let rewriter = ImportRewriter::new();
    let source = r#"import Emoji from 'emoji-mart-vue-fast/src/components/Emoji.vue';"#;
    let result = rewriter.rewrite(source, SourceType::ts(), None);

    assert_eq!(result.code, source);
}

#[test]
fn explicit_resolution_mode_attributes_override_the_importer_default() {
    use crate::PackageResolutionMode::{Import, Require};

    let source = r#"
import type { CommonJs } from "static-package" with { "resolution-mode": "require" };
type Dynamic = import("dynamic-package", { with: { "resolution-mode": "require" } });
import type { EsModule } from "import-package" with { "resolution-mode": "import" };
"#;
    let occurrences = ImportRewriter::new()
        .collect_all_specifier_occurrences(source, SourceType::ts().with_module(true));

    assert_eq!(
        occurrences,
        vec![
            ("static-package".into(), Require),
            ("dynamic-package".into(), Require),
            ("import-package".into(), Import),
        ]
    );
}
