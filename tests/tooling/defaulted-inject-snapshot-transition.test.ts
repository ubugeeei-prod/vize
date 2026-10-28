import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";

test("the optional-inject snapshot changes only the explicitly retired fallback warning", () => {
  const original = fs.readFileSync(
    new URL(
      "../../tools/support/compat/defaulted-inject/historical-partial.snap.txt",
      import.meta.url,
    ),
    "utf8",
  );
  const actual = fs.readFileSync(
    new URL(
      "../../crates/vize_croquis_cf/src/analyzer/tests_snapshots/snapshots/vize_croquis_cf__analyzer__tests_snapshots__provide_inject__snapshot_partial_shared_child_branches.snap",
      import.meta.url,
    ),
    "utf8",
  );
  const retired = original
    .split("\n")
    .filter((line) => line.indexOf("Warning: **Conditionally Defaulted Inject**:") === 0);
  assert.equal(retired.length, 1);
  assert.deepEqual(actual, original.replace(retired[0] + "\n", ""));
});

test("the prior provider-context contract changes only the explicit fallback finding", () => {
  const historical = fs.readFileSync(
    new URL(
      "../../tools/support/compat/defaulted-inject/historical-provider-context.rs.txt",
      import.meta.url,
    ),
    "utf8",
  );
  const actual = fs.readFileSync(
    new URL(
      "../../crates/vize_croquis_cf/src/analyzer/tests_provide_inject/provider_context.rs",
      import.meta.url,
    ),
    "utf8",
  );
  const retired =
    '#[test]\nfn test_defaulted_inject_without_ancestor_provider_is_warning() {\n    use crate::diagnostics::CrossFileDiagnosticKind;\n    use crate::diagnostics::DiagnosticSeverity;\n\n    let mut analyzer =\n        CrossFileAnalyzer::new(CrossFileOptions::default().with_provide_inject(true));\n    analyzer.add_file_with_analysis(\n        Path::new("Child.vue"),\n        "",\n        script_analysis(\n            r#"import { inject } from \'vue\'\nconst theme = inject(\'theme\', \'light\')"#,\n            &[],\n        ),\n    );\n\n    let result = analyzer.analyze();\n    let diagnostic = result\n        .diagnostics\n        .iter()\n        .find(|diagnostic| {\n            matches!(\n                &diagnostic.kind,\n                CrossFileDiagnosticKind::UnmatchedInject { key } if key == "theme"\n            )\n        })\n        .expect("defaulted inject should still be reported");\n\n    assert_eq!(diagnostic.severity, DiagnosticSeverity::Warning);\n}\n';
  const replacement =
    '#[test]\nfn test_defaulted_inject_without_ancestor_provider_has_no_missing_finding() {\n    use crate::diagnostics::CrossFileDiagnosticKind;\n\n    let mut analyzer =\n        CrossFileAnalyzer::new(CrossFileOptions::default().with_provide_inject(true));\n    analyzer.add_file_with_analysis(\n        Path::new("Child.vue"),\n        "",\n        script_analysis(\n            r#"import { inject } from \'vue\'\nconst theme = inject(\'theme\', \'light\')"#,\n            &[],\n        ),\n    );\n\n    let result = analyzer.analyze();\n    let missing_count = result\n        .diagnostics\n        .iter()\n        .filter(|diagnostic| {\n            matches!(\n                &diagnostic.kind,\n                CrossFileDiagnosticKind::UnmatchedInject { key } if key == "theme"\n            )\n        })\n        .count();\n\n    assert_eq!(missing_count, 0);\n}\n';
  assert.equal(historical.split(retired).length, 2);
  assert.equal(actual, historical.replace(retired, replacement));
});
