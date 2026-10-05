# Explicit config-schema lifecycle

P0 [#7986](https://github.com/ubugeeei-prod/vize/issues/7986) reports that
`vize lint --no-config -f plain MyText.vue` creates
`node_modules/.vize/vize.config.schema.json` even in an otherwise empty project.
The complete original 34-byte SFC and issue body are retained under
`tests/_fixtures/cli/lint-schema-side-effects/`, with immutable input hashes.
The authored file remains `MyText.vue.txt` at rest and is copied byte-for-byte
to its original runtime filename; existing compiler corpora are unchanged.

Both lint and direct check unconditionally called `write_schema(None)` before
loading configuration. The correction consults the successfully selected JSON
configuration after loading: materialization is requested only by the exact
`$schema` value `node_modules/.vize/vize.config.schema.json` or the same value
prefixed with `./`. The target belongs to that configuration's directory and
requires an already present physical `node_modules` directory. Dependency,
cache-directory and schema-file symlinks are declined. Unconfigured commands,
`--no-config`, ordinary JSON configs, remote/other schema references, and
projects without installed dependencies retain their complete filesystem.

Explicit schema authoring remains available: `write_schema(Some(directory))`
can initialize an authoring directory and refresh stale bundled bytes; the
published `vize/schemas/vize.config.schema.json` export and existing
`gen:schema` Pkl generation command remain unchanged. There is no existing
native `vize schema`, `setup`, or `init` command; this change adds none.

The normal source-built tooling test requires the exact current CLI build
receipt and binary hash. Its 22 scenarios retain 44 complete process outputs
and statuses, whole UTF-8 byte arrays, every file's complete bytes, empty
directories, links and untouched external sentinels before asserting. They
cover original plain/JSON output, default/explicit/disabled configuration,
uninstalled/installed authoring, stale/current schema, relocated config,
no-config refusal, and all three link destinations plus a non-directory
dependency path. Check's disabled-tool controls exercise the same lifecycle
without launching the backend. Current-schema modification times must remain
unchanged. The existing explicit writer laws and genuine Pkl-generation test
remain mandatory.

The current upstream-lint observation previously expected this unintended
schema addition without a Vize config. Its inventory expectation now requires
the entire original work tree to remain unchanged. All original plants,
inputs, judges, parser, provider, full diagnostics, and thread settings remain
unchanged; this grants no benchmark ranking, speed or native migration credit.

Runtime remains pending until fresh exact-head Actions executes the default
source CLI and all 44 observations. Protected full suites, unchanged 104
instruction ceilings, actual signed merge, reporter attribution and a later
verified release are separate delivery requirements. These ordinary metadata
checks do not claim atomic protection against concurrent filesystem retargeting
or independently executed Windows junction behavior.

Paired issue decision: [preparation record](https://github.com/ubugeeei-prod/vize/issues/7986#issuecomment-5993667036).
