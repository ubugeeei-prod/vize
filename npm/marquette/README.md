# `@vizejs/marquette`

Typed application marquettes for every Vize target.

> Experimental: serialized contracts are versioned, but the TypeScript API may
> evolve while target adapters are being validated.

Marquette is the small, language-neutral description shared by web, native,
desktop, terminal, backend, transport, testing, and gallery tooling. This
package provides literal-preserving TypeScript authoring. Runtime validation is
available through a separate entry, while canonicalization and compatibility
analysis remain focused layers.

## Author a marquette

```ts
import { defineApplicationMarquette } from "@vizejs/marquette";

export default defineApplicationMarquette({
  application: "shop",
  targets: ["web", "native"],
  environments: [
    {
      id: "web",
      target: "web",
      consumer: "client",
      runtime: "browser",
    },
    {
      id: "native",
      target: "native",
      consumer: "client",
      runtime: "native",
    },
    {
      id: "server",
      target: "web",
      consumer: "server",
      runtime: "rust",
    },
  ],
  backends: [
    {
      id: "api",
      family: "rust",
      environment: "server",
    },
  ],
  protocols: [
    {
      id: "api.query",
      family: "schema-query",
      backend: "api",
    },
  ],
  routes: [
    {
      id: "home",
      path: "/",
      environment: "web",
      rendering: "hybrid",
      backend: "api",
      protocol: "api.query",
    },
  ],
});
```

Environment dependencies, backend owners, protocol owners, and route
references are checked against identifiers declared in the same object. A
misspelled reference is therefore an authoring error rather than a runtime
surprise.

## Runtime validation

Use the dedicated validation entry when contracts cross file, process, or
language boundaries:

```ts
import { validateApplicationMarquette } from "@vizejs/marquette/validate";

const diagnostics = validateApplicationMarquette(shop);
for (const diagnostic of diagnostics) {
  console.error(diagnostic.code, diagnostic.path, diagnostic.message);
}
```

Validation is deterministic, does not mutate the authored object, and reports
stable codes shared with the native contract implementation. Keeping it in a
separate entry means authoring-only applications do not include validation
code.

## Guarantees

- Exact identifiers remain available as `EnvironmentId`, `BackendId`,
  `ProtocolId`, and `RouteId` unions.
- `defineApplicationMarquette` returns its input without allocation or
  normalization.
- The published application-contract schema is available from
  `@vizejs/marquette/schema`.
- Every optional contract field documents its default in JSDoc.
- Package builds enforce a 1 KiB gzip budget for authoring and a 3 KiB budget
  for runtime validation.

Marquette describes components and environments; it does not author UI
components. Public Vize UI components use real `.vue` SFC files as their
canonical source, never authored `h()` calls or handwritten render functions.
