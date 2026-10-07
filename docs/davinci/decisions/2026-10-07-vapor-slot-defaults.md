# Vapor slot destructuring defaults (#7886)

The original [report](https://github.com/ubugeeei-prod/vize/issues/7886) supplies
complete `App.vue` and `Child.vue` sources: a missing `label` must render
`fallback:1`. Preserve its issue body, shell reproduction, both SFCs and both
reported JavaScript snippets byte for byte in the compiler differential corpus.
The runtime pack also pins independently authored update controls.

The Vapor generator previously retained only destructured binding names, then
resolved every local to `_slotPropsN.local`. That discarded defaults and original
property paths. Default-bearing slot patterns now retain checked binding paths
and wrap each assignment read in Vue's lazy `getDefaultValue` helper. The existing
expression owner resolves fallback expressions and computed property keys in the
current slot/loop scope. The binding walk uses an explicit stack and the existing
expression nesting guard. Patterns without defaults retain their existing path;
no extra pipeline stage, level serialization or budget change is introduced.

The [reported Vue 3.6.0-rc.10 generator](https://github.com/vuejs/core/blob/v3.6.0-rc.10/packages/compiler-vapor/src/generators/for.ts)
uses the same lazy helper. Ordinary Actions run the original complete Vapor SFCs
and seven authored controls under the locked Vue 3.6.0-rc.9 runtime. The controls
compare complete official VDOM/Vapor observations, namespaces, fallback side effects,
reactive updates, diagnostics and unmount. Missing/undefined values use defaults;
null, empty strings, false and zero retain their supplied values. Named aliases,
comments, quoted keys, nested object/array paths and nested-parent defaults are
covered. Installed execution of the report's rc.10 remains unexecuted.

Default-bearing slots remain outside the current native L3 admission. This repair
changes the shared retained generator only; whole-product native SFC credit and
the compiler history gate remain unfinished. Source Actions, unchanged protected
instruction ceilings/full suites, actual merge and released installed verification
are pending. The release owner must verify the original two-SFC example after
publication. No n8n upstream state is changed.

Initial source `64349976189ab9fb073aadbe242f03c6b6907c41` genuinely executes the
original full Vapor example successfully in [Rust worker 3](https://github.com/ubugeeei-prod/vize/actions/runs/37589761191/job/112690171449).
Worker 1 fails the control oracle: Vize VDOM resolves script-setup default
initializers through `_ctx.record`/`_ctx.state`, while the returned setup object
is hidden from that public proxy. Preserve the complete raw failure/module in
`vize-vdom-source-64349976.failure.raw.txt` and the readable companion. TODO: repair the separate VDOM default
initializer binding ownership; this PR does not claim that behavior repaired.
Use the locked official Vue compiler for the control oracle while retaining
every original source, control, expected phase and diagnostics assertion.
The successor requires its own complete Actions; predecessor green lanes
and the single original-case pass grant no successor acceptance.

Source `0910fca0881094e7eb99cf97b5b98ef83a34d97d` passes the original again
and the first five official whole-trace controls. [Worker 1](https://github.com/ubugeeei-prod/vize/actions/runs/37591339702/job/112696103427)
fails only the sixth control’s literal expected vector after current/official
traces agree: static `:data-label` is correctly camelized to `dataLabel` by both
compilers, but the authored pattern requested `'data-label'`. Preserve the complete
prior controls and raw failure. Change only that pattern key to `'dataLabel'`;
the original reported App/Child, all independently authored expected phases and
every production blob remain unchanged. The final parent-default control and
whole successor still require actual execution. This correction does not relax
the official comparator or any assertion.

Source `903f341a992eff755db3b664cc5578c4eff3d5ce` actually passes the full
original in [worker 3](https://github.com/ubugeeei-prod/vize/actions/runs/37593078566/job/112702208693)
and all seven official/independent controls in [worker 1](https://github.com/ubugeeei-prod/vize/actions/runs/37593078566/job/112702208819).
All four Rust workers, their source report, compilation and Clippy pass.
The whole run still awaited canonical corpus and retained the old-base audit
failure. Refresh onto actual verified main `b27868776bbb872aba68873c29970f5c4ed77a2b`,
including the shared audit merge `706a5b7886c363f6c67a03964ac55f26c5a2a341` and
UI countdown test repair, preserving all production, original corpus, controls,
oracles and assertion bytes. The refreshed SHA requires its own complete source
Actions; these predecessor passes grant no merge, release or installed credit.

Existing PR #8170 is refreshed from its actual remote head
`c099620b79ab83b78fea4662e1d5300655753c80` by merging genuine fetched main
`8c7727613de6213e0a52cde96eeb295d01c9bef5`. Preserve all owned generator,
complete original source, control, reference and assertion bytes; retain both
independent runtime packs when joining the compiler registry. The common
decision record is coordinated across the existing PRs. Previous source-green
results remain historical until this exact refreshed source completes Actions
and protected validation; merge and installed-release acceptance remain pending.
