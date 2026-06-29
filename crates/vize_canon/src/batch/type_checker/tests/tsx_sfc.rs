use super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};

#[test]
fn batch_type_checker_preserves_named_exports_from_tsx_sfc() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "tsx-sfc-named-export-contextual-typing",
        &[
            (
                "src/Widget.vue",
                r#"<script lang="tsx">
type WidgetOptions = {
  render?: () => unknown;
  slots: {
    root: (context: { size: "small" | "large" }) => string;
  };
};

export const WidgetConfig: WidgetOptions = {
  render: () => <span />,
  slots: {
    root: ({ size }) => size,
  },
};
</script>

<template>
  <div />
</template>
"#,
            ),
            (
                "src/register-widget.ts",
                r#"import { WidgetConfig } from "./Widget.vue";

export type RegisteredWidgetConfig = typeof WidgetConfig;
"#,
            ),
            (
                "src/theme.ts",
                r#"import type { RegisteredWidgetConfig } from "./register-widget";

export const widgetTheme: RegisteredWidgetConfig = {
  slots: {
    root: ({ size }) => {
      return size === "small" ? "text-sm" : "text-lg";
    },
  },
};
"#,
            ),
        ],
    );

    let Some(snapshot) = snapshot_project_diagnostics(&project_root) else {
        let _ = std::fs::remove_dir_all(&project_root);
        return;
    };

    assert!(
        snapshot.iter().all(|(file, code, message)| {
            !(file == "src/register-widget.ts" && *code == Some(2614))
                && !(file == "src/theme.ts" && *code == Some(7031) && message.contains("size"))
        }),
        "TSX SFC named exports should preserve downstream contextual typing, got: {snapshot:#?}"
    );

    let _ = std::fs::remove_dir_all(&project_root);
}
