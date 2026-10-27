Issue #10579: the source-built `vue-router` at `593d7ad` and the published
`5.4.0` archive report the same version, but only the source build declares
`_RoutePropsForFilePath` and a component-aware `definePage`. The batch run
installs the unchanged source-build members bound by
`tests/fixtures/typechecker/experimental-router-project-capability/artifacts.json`,
so the project admits the plugin's `props: true` projection.

Both pages declare a hoisted `Props` alias. `route-params-as-props` reads route
params through `props` in setup and template and a bare hash param. In
`required-component-prop` the component requires a prop the route cannot pass,
so `props: true` reports the error vue-tsc reports with the real plugin.

`auto-routes.d.mts.txt` is the source build's `vue-router-auto-routes.d.mts`;
its digest is recorded in
`tests/fixtures/typechecker/experimental-router-preview-593d7ad/frozen/provider-package.json`.
