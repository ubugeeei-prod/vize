# Native lint execution settings

Use this guide to include relationships between files in a lint run or make warnings fail CI.
Keep these settings in configuration when the CLI and Vite+ tasks should follow the same policy.
For a first run and the available command options, see [linting with the CLI](./cli.md#lint).
To understand the cross-file reports, see [component trees and complexity](./cross-file-complexity.md).

Compose native lint settings through `.withVue({ linter: ... })`. Keep the root `lint` section for
the host's Oxlint flat config:

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    preset: "opinionated",
    crossFile: true,
    strictReactivity: true,
    crossFileTree: true,
    crossFileComplexity: true,
    maxWarnings: 0,
    reportUnusedDisableDirectives: true,
  },
});
```

| Setting                         | Default      | Effect                                                                          |
| ------------------------------- | ------------ | ------------------------------------------------------------------------------- |
| `preset`                        | `happy-path` | Select the group of enabled rules.                                              |
| `crossFile`                     | `false`      | Check relationships between the selected Vue files.                             |
| `crossFileTree`                 | `false`      | Print the provide/inject tree and enable cross-file analysis.                   |
| `crossFileComplexity`           | `false`      | Print complexity scores and hotspots and enable cross-file analysis.            |
| `strictReactivity`              | `false`      | Enable `type/no-reactivity-loss`, unless an explicit rule setting overrides it. |
| `maxWarnings`                   | Unset        | Fail when the warning count exceeds this limit; `0` permits none.               |
| `reportUnusedDisableDirectives` | `false`      | Report supported disable comments that suppress no Vize finding.                |

For settings shared with CLI and editor, put these keys in the `linter` object passed to
`withVue`. `crossFileTree` and
`crossFileComplexity` also enable cross-file analysis. Cross-file checks operate
on the files selected for a native CLI run. The strict-reactivity switch enables
`type/no-reactivity-loss` for both CLI and editor diagnostics; explicit rule
settings, including scoped `off` entries, take precedence over this switch.

CLI opt-in flags remain available for direct `vize lint` invocation, and
`--max-warnings` overrides the configured limit. `--no-config` ignores all
configured lint settings. An absent warning limit permits warnings; `0` fails
when any warning remains. Negative limits are invalid.

In standalone JavaScript/TypeScript files and SFC script blocks, use
`// vize-disable-next-line script/rule-name`, an end-of-line
`// vize-disable-line script/rule-name`, or a `/* vize-disable script/rule-name */`
region followed by `/* vize-enable script/rule-name */`. Omit the rule list to
control all rules, or separate rule names with commas. These aliases let Vize
script rules coexist with Oxlint's unused-disable checks. Marker text inside
strings, template literals, and regular expressions does not suppress findings.
Template comments retain the `eslint-` / `oxlint-` grammar described below;
existing CSS `vize-disable` comments keep their style-block behavior.

Vite+ lint tasks accept paths and `--fix`. Put native-only settings in the `linter` object passed
to `withVue` instead of passing them through to Oxlint. For example, `vp run lint -- src`
runs both linters with their own configuration.

<span id="unused-disable-directives"></span>

## Unused disable comments

`reportUnusedDisableDirectives: true` reports unused suppression comments as
`vue/comment-directive` errors at the comment. This is an opt-in execution setting,
not a preset rule: adding the diagnostic ID to `rules` does not enable the check.
It is off by default; `false` disables it for configured entries. The explicit
`--report-unused-disable-directives` flag enables it even when configuration is `false`.

In templates, this checks `eslint-disable`, `eslint-disable-line`, and
`eslint-disable-next-line`, including their `oxlint-` equivalents. A comment is used
when it suppresses a finding. Only enabled rules executed by Vize count. Disabled
rules are not counted; names owned by other linters are left to those linters.

When cross-file analysis is enabled, supported named project-rule directives in
the selected Vue files also participate, including supported script-block comments.
This does not imply general coverage for arbitrary script/style directives or all
`vize-disable` alias forms. Text inside strings, attributes, or custom blocks cannot
create a suppression comment.

See [comment annotations](./comment-annotations.md) for suppression scope and
[cross-file rules](../rules/cross-file.md) for the project checks. This native
reporting option does not establish full `eslint-plugin-vue` comment-directive parity.

## Parser and project diagnostics

Lint reports also include failures from parsing source or loading a type-aware
project. These diagnostic IDs are outside the preset rule catalogue. Follow the
message and reference page to repair the input, then run lint again.

| Diagnostic                   | Reference                                                                            |
| ---------------------------- | ------------------------------------------------------------------------------------ |
| `parser/template`            | [Template syntax and parser recovery](../rules/reference/parser-template.md)         |
| `parser/sfc`                 | [Single-file component structure](../rules/reference/parser-sfc.md)                  |
| `parser/jsx`                 | [JSX/TSX parsing and lowering](../rules/reference/parser-jsx.md)                     |
| `type/project-configuration` | [TypeScript project configuration](../rules/reference/type-project-configuration.md) |

## Static class defaults

`html/no-duplicate-class` warns when the same token appears more than once in a
literal `class` attribute, such as `class="button button"`. Dynamic bindings
remain unchecked, and the rule does not apply an automatic fix.

| Preset                     | Previous default | Current default |
| -------------------------- | ---------------- | --------------- |
| `happy-path`, `ecosystem`  | Off              | Warning         |
| `opinionated`, `nuxt`      | Warning          | Warning         |
| `essential`, `incremental` | Off              | Off             |

An explicit `rules` entry can disable the warning or change its severity.
`incremental` continues to start empty; enable this rule explicitly when adopting it.
