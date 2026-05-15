# Oxlint + Vize Example

This is the smallest runnable example of Vize Patina executed through `oxlint-plugin-vize`.
The local scripts use the temporary `oxlint-vize` wrapper workflow so scriptless `.vue` files can still be linted during the alpha period.

## Prerequisites

This example uses the locally built Vize Oxlint plugin bundle and `@vizejs/native`. It assumes Node 24+ and the repository's `.node-version`, and the example scripts rebuild dependencies automatically. If you want to build them up front from the repository root:

```bash
vp install
vp run --filter './npm/vize-native' build
vp run --filter './npm/oxlint-plugin-vize' build
```

## Run

```bash
vp run --filter './examples/oxlint-vize' lint
```

This command intentionally exits non-zero because it includes `src/HasPatinaErrors.vue`.
It mixes:

- Oxlint core diagnostics from `no-console`
- Patina diagnostics from `oxlint-plugin-vize`
- The `stylish` formatter so the default code frame does not dominate the output
- `settings.vize.helpLevel: "none"` so the long Patina remediation block stays hidden by default

If you want the currently recommended balance of terminal readability plus concise remediation hints:

```bash
vp run --filter './examples/oxlint-vize' lint:short-help
```

If you want to check `no-unused-vars` specifically:

```bash
vp run --filter './examples/oxlint-vize' lint:unused-vars-probe
```

Current observed behavior in this repository: that command reports `0` findings on `.vue` even though `src/UnusedVarProbe.vue` contains an unused binding. I verified separately that the same `no-unused-vars` rule does fire on a plain `.ts` file, so this is an Oxlint/Vue behavior to be aware of rather than a Patina bridge conflict.

If you only want the success path:

```bash
vp run --filter './examples/oxlint-vize' lint:clean
```

If you want JSON output from the same command:

```bash
vp run --filter './examples/oxlint-vize' lint:json
```

Because of [oxc-project/oxc#20465](https://github.com/oxc-project/oxc/issues/20465), treat that JSON output as best-effort debugging output for now, not as a source of truth for original template/style locations.

If you want the long Patina `Help:` block as well:

```bash
vp run --filter './examples/oxlint-vize' lint:with-help
```

If you want the raw direct CLI path, use the wrapper entry once the plugin has been built:

```bash
vp exec node ../../npm/oxlint-plugin-vize/dist/cli.mjs -c .oxlintrc.json -f stylish src
```

## Notes

- Oxlint's built-in `vue` plugin is enabled through `"plugins": ["vue"]`.
- Oxlint's built-in `no-console` rule is enabled so the example shows native Oxlint output mixed with Patina output in one run.
- The default example commands use `-f stylish` because Oxlint's default formatter prints a large code frame for every finding, while `stylish` keeps the Patina message body intact and remains the most usable formatter today.
- The wrapper appends a temporary `<script setup>` block only for scriptless `.vue` files, then rewrites the reported path back to the original file. This is an alpha-period workaround and is intended to be removed once upstream JS plugin coverage improves.
- `settings.vize.helpLevel` controls remediation density. `lint` keeps it at `"none"`, `lint:short-help` is the recommended terminal compromise, and `lint:with-help` shows the full verbose help block.
- `helpLevel: "full"` only changes how much remediation text Patina prints. It does not fix Oxlint's current original-SFC reporting limitation.
- A dedicated `lint:unused-vars-probe` command is included because `no-unused-vars` currently does not emit on the example `.vue` SFC, even without the Patina plugin.
- The Vize Oxlint plugin is loaded from `../../npm/oxlint-plugin-vize/dist/index.mjs`.
- The plugin starts with a single-rule native Patina run and only upgrades to a shared full-file pass when multiple Patina rules are enabled for the same file.
- Patina help text is normalized to plain text so terminal output stays readable even without Markdown rendering.
- The wrapper removes the need for `<script setup>` in pure template/style files, but the workaround is temporary and should disappear once upstream support is fixed.
- Because of [oxc-project/oxc#20465](https://github.com/oxc-project/oxc/issues/20465), formatter anchors still come from the extracted script program. The human-readable summary carries the original SFC `line:column`, which is why `stylish` is the recommended workflow for now.
- `src/HasPatinaErrors.vue` is the failing mixed-output sample, `src/Clean.vue` is the clean sample, and `src/UnusedVarProbe.vue` is the `no-unused-vars` probe.
