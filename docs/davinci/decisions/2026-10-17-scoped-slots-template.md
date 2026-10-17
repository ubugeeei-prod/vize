# Removed `$scopedSlots` in component template expressions

The existing n8n authored `script-scopedslots--template-instance` control uses
`export default {}` and a template interpolation of `$scopedSlots`. Its source,
independent full-51 provider packet and complete CLI expectation remain unchanged
in `tests/_fixtures/differential/lint/n8n-cli-config/script-rule-research.json`.
At main `8df6e1fe`, the native rule had only a script visitor, so it could not
consume the template reference. No fresh before execution is claimed here.

The pinned primary [ESLint Vue 10.9.2 rule](https://github.com/vuejs/eslint-plugin-vue/blob/v10.9.2/lib/rules/no-deprecated-dollar-scopedslots-api.js)
reports unbound `$scopedSlots` template references. Its template visitor skips
references with local variables. This fix enables the existing native template
capability and uses the already validated canonical reference packet from #9251.
Only component-context references with authenticated component-binding inspection
are reported. Template-local aliases and positively witnessed setup values are
preserved; refused inspection does not establish a component reference.

No parser, lexical inference, rule default, script warning, script range or
diagnostic descriptor changes. The native control packets retain full ranges,
messages, help, fixes and labels. Additional controls cover scriptless CRLF and
Unicode/entity ranges, loop and slot shadows beside an actual component read,
setup values, comments/strings, suppression, replacement `$slots` and explicit
disable. Eleven CLI invocations include the unchanged original full-51 projection
and preserve input/config bytes. Complete raw process packets are written under
the existing nextest artifact directory before assertions.

Runtime qualification is pending Actions. This does not turn the retained failed
22-source campaign green, qualify all 51 rules, resolve registration-range or
self-recursion disagreements, or establish public-package acceptance. The user
closed #8142; it is not reopened or reinterpreted as complete adoption.

Template capabilities report through the original template context before unused
directives are finalized. This preserves authenticated comment ownership and the
same suppression usage records without another parse or comment scan. Only
capabilities that actually execute contribute to unused-directive accounting.
The active CLI carrier is `vite.config.mjs` with `vue:`; receipts retain the
original JSON policy and argv beside the current module bytes and argv.
