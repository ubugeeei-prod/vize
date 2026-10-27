# @vizejs/volar-plugin

Use Vize's native Vue projection with Volar's language context and the ordinary
TypeScript language service. Installation is explicit; this package does not
replace a workspace's editor server automatically.

Not yet published to npm: it is an experimental workspace package until it joins
the release.

```sh
npm install --save-dev @vizejs/volar-plugin typescript vue
```

```js
import ts from "typescript";
import { createVizeLanguageService } from "@vizejs/volar-plugin";

const { language, languageService } = createVizeLanguageService(ts, yourHost);
const diagnostics = languageService.getSemanticDiagnostics("/project/src/App.vue");
// Dispose when the owning project closes.
languageService.dispose();
```

`yourHost` is a normal TypeScript `LanguageServiceHost`. Its snapshots are the
source authority, including unsaved buffers. File existence, module resolution,
compiler options, versions and project versions belong to that host. Advance
versions when inputs change and remove deleted files from its script list.

The implementation uses the public Volar 2.4.30 `createLanguage`,
`decorateLanguageServiceHost` and `createProxyLanguageService` APIs. Native code
generation calls Canon's existing Content Mapper transform. It does not contain
a second Vue-to-TypeScript generator. Both sides of every native UTF-8 mapping,
diagnostic directive and semantic link are converted to UTF-16 before Volar sees
them. Incorrect byte boundaries fail instead of producing a shifted location.

Mappings retain the native feature masks. Diagnostic-only spans do not gain
navigation. Rename additionally requires an exact verbatim spelling. Native
IGNORE/EXPECT and generated-copy suppression directives are applied to a complete
syntactic/semantic collection; unused expectations and native parse diagnostics
are returned in authored coordinates. This service composition is the complete
diagnostic integration.

Diagnostics follow Content Mapper's range law, including synthetic punctuation
around a mapped expression. Volar's generic endpoint mapping alone can omit
those errors. Completely generated problems remain visible with the
`vize-generated` source label. This diagnostic fallback never enables navigation
or rename in generated gaps.

For an existing Volar host, `createVizeLanguagePlugin(ts, options)` exposes the
standard language plugin and virtual code. That lower-level host must also
integrate native parse diagnostics and `applyDiagnosticDirectives` from
`@vizejs/native/language-projection.js`; Volar's code-generation interface alone
cannot add framework diagnostics. Volar groups several operations under its
`semantic` and `navigation` flags. This package preserves the native mask in its
projection and enables the supported hover/completion/navigation groups
conservatively; it does not promise every optional Volar feature.

Options are explicit and fixed for a project lifetime:

```js
createVizeLanguageService(ts, yourHost, {
  configFileName: "/project/tsconfig.json", // optional; no config search by default
  checkJavaScript: true,
  optionsApi: true,
  preserveUnusedDiagnostics: true,
});
```

The selected tsconfig's Vue options use the same native configuration resolver
as Content Mapper. TypeScript paths, aliases and resolver options stay on the
host. Recreate the composition when native-generation options or that selected
Vue configuration change.

`.vue` is enabled by default. `includeArt: true` additionally admits `.art` and `.art.vue` files
using Canon's existing Vue and Musea variant projections. Script bindings in
variant templates use the same authored mappings. Gallery metadata editing,
rendering and variant selection commands are outside this package. Invalid
metadata and an Art block with no checkable variant report explicit diagnostics
through the native SFC/Musea parsers. No alternate provider is
substituted when the matching native projection API is unavailable.

The focused Actions workflow builds the reviewed native source and exercises
the real Volar context and TypeScript 6.0.3 service. It retains complete packets,
input/projection bytes, timings, provider versions and loaded native identity.
Those timings describe that fixture and host, not startup or project-scale
performance in other editors.

Upstream APIs: [Volar language definitions](https://volarjs.dev/reference/languages/),
[Volar.js source](https://github.com/volarjs/volar.js),
[Vue language tools](https://github.com/vuejs/language-tools).
