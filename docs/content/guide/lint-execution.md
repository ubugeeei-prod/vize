# Native lint execution settings

Use this guide to include relationships between files in a lint run or make warnings fail CI.
Keep these settings in configuration when the CLI and Vite+ tasks should follow the same policy.
For a first run and the available command options, see [linting with the CLI](./cli.md#lint).
To understand the cross-file reports, see [component trees and complexity](./cross-file-complexity.md).

Keep native lint settings in `vue.linter` in `vite.config.*`; Vite+ task overrides can use `lint.vue`:

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vue: {
      preset: "opinionated",
      crossFile: true,
      strictReactivity: true,
      crossFileTree: true,
      crossFileComplexity: true,
      maxWarnings: 0,
      reportUnusedDisableDirectives: true,
    },
  },
});
```

For settings shared with CLI and editor, put these keys in `vue.linter`. `crossFileTree` and
`crossFileComplexity` also enable cross-file analysis. Cross-file checks operate
on the files selected for a native CLI run. The strict-reactivity switch enables
`type/no-reactivity-loss` for both CLI and editor diagnostics; explicit rule
settings, including scoped `off` entries, take precedence over this switch.

CLI opt-in flags remain available for direct `vize lint` invocation, and
`--max-warnings` overrides the configured limit. `--no-config` ignores all
configured lint settings. An absent warning limit permits warnings; `0` fails
when any warning remains. Negative limits are invalid.

Vite+ lint tasks accept paths and `--fix`. Put native-only flags in `lint.vue`
instead of passing them through to Oxlint. For example, `vp run lint -- src`
runs both linters with their own configuration.

`reportUnusedDisableDirectives: true` reports unused template `eslint-disable`,
`eslint-disable-line`, and `eslint-disable-next-line` comments, including their
`oxlint-` equivalents. It is off by default; `false` disables it for configured
entries. The explicit `--report-unused-disable-directives` flag enables reporting
even when configuration is `false`. Only rules executed by Vize count, so comments
for external linters are left alone. This uses the existing native template pragma
check; script/style and `vize-disable` comments do not produce unused reports.

## Static class defaults

`html/no-duplicate-class` warns when the same token appears more than once in a
literal `class` attribute, such as `class="button button"`. Dynamic bindings
remain unchecked, and the rule does not apply an automatic fix.

| Preset | Previous default | Current default |
| --- | --- | --- |
| `happy-path`, `ecosystem` | Off | Warning |
| `opinionated`, `nuxt` | Warning | Warning |
| `essential`, `incremental` | Off | Off |

An explicit `rules` entry can disable the warning or change its severity.
`incremental` continues to start empty; enable this rule explicitly when adopting it.
