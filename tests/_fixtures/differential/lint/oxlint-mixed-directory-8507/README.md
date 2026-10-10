# Mixed Vue/HTML directory regression (#8507)

The root `package.json`, `.oxlintrc.json`, `app/index.html` and `app/App.vue`
are literal files extracted from [the third-party report](https://github.com/ubugeeei-prod/vize/issues/8507).
`issue.md` preserves the original body without an added newline; `issue.json`
preserves its captured GitHub response. The reproduction does not initialize Git.
The original package pins remain Oxlint 1.81.0 and oxlint-plugin-vize 0.441.0.

`controls.json` is an authored execution plan, not a runtime receipt. Materialize
only its four `baseFiles` into a fresh physical directory with no Git or
Jujutsu marker in any ancestor, then apply the listed overlays for each case. A `parentFiles` case places that project in a fresh `repro/`
child and writes the listed file in its parent. Never lint this whole fixture
directory: the separate control inputs are not part of the reporter's project.

The CLI laws require a successful clean mixed-directory run, correct CLI and
configuration HTML exclusion, and the same explicit Vue result. The duplicate
`id` companion proves that directory and ignored-HTML runs still lint Vue: its
one error has an independently authored original-file span and message. All four
Vue error companions explicitly enable `vize/vue/no-duplicate-attributes` as an
error; `settings.vize.preset` alone does not enable a JavaScript plugin rule.
The same `config-html-error.json` overlay serves the Vue and HTML error controls;
`config-ignore-html-error.json` preserves HTML exclusion while enabling the rule.
The separate HTML error companion must execute
the retained HTML once, preventing a silent skip from looking like a fix.

Retain the literal bare argv observations. Record the actual provider/child
environment; a bare invocation qualifies the default-format law only when
Oxlint genuinely selects default output. Separately execute all CLI cases in
each explicit `default`, `json`, `unix` and `stylish` format, retaining complete
stdout, stderr, status, file counts and diagnostic packets. Do not replace
whole reports with these authored diagnostic fields or transfer a result from
another format or provider version.

The native laws cover Git-free `.gitignore` matching inside the project and in
its parent, `--no-ignore` retaining Git-derived filtering, ignored cwd pruning,
and the explicit-file exception. Nested VCS metadata, nested configuration and an external custom
ignore source still refuse. Save whole selections, provenance, authorities and
refusal bytes. Existing 1.78/1.86 missing-Git refusals and unsupported-host
controls remain unchanged; this adds an exact 1.81 profile rather than changing
the old envelope.

Historical BEFORE observations are a separate campaign for each actual public
0.435.0, 0.439.0 and 0.441.0 plugin with actual Oxlint 1.81.0. Keep each package,
registry integrity, entrypoint/native identity, original input hashes, full
argv/environment, complete output and exit bound to its own packet. Preserve
all genuine failures, including failure before HTML selection. The report's
version table is reporter evidence, not a measured result. Source Actions,
protected merges and a future included public release each need their own
qualification. No historical or successor campaign has been run by this fixture.
