This corpus exercises the published Vue Router 5.4.0 experimental resolver and
parameter parsers with its actual generated route declarations. `provider.json`
pins the npm archive integrity, four installed provider files, and every project
input. The lockfile fixes Vue, the router, TypeScript, and the reference checker.
The shared provider helper verifies these inputs before executing a fresh install
and the router's public route scanner; its complete declaration output must match
`project/typed-router.d.ts.txt` byte for byte.

The opt-in is the existing project-local
`vueCompilerOptions.plugins: ["vue-router/volar/sfc-typed-router"]`. The fixture
uses `TypesConfig.Router`, a numeric path parser, a default numeric query parser,
an optional array query parser, an optional custom Date parser, and an optional
hash parameter. Authored-error controls preserve the generated declarations so
regeneration cannot erase the error under test.

The published package imports `useRoute` from `vue-router` and `definePage` from
`vue-router/experimental`. Its shipped page plugin has no component-props
augmentation and the package has no `_RoutePropsForFilePath` export. Newer
upstream source has a different contract; it requires a separately pinned
provider and qualification rather than adding undeclared helpers to this one.

The dedicated checker, editor, and mapper tests retain raw process/protocol
observations before comparing complete diagnostics and navigation packets.
Checker tests require the receipted source executable. Mapper tests additionally
require the official content-mapper-capable compiler; an ordinary native compiler
that rejects `--runExternalCode` is not a mapper oracle.
