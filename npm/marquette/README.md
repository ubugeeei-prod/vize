# `@vizejs/marquette`

Typed application marquettes for every Vize target.

> Experimental: serialized contracts are versioned, but the TypeScript API may
> evolve while target adapters are being validated.

Marquette is the small, language-neutral description shared by web, native,
desktop, terminal, backend, transport, testing, and gallery tooling. This
package provides literal-preserving TypeScript authoring. Runtime validation,
canonicalization, and compatibility analysis are delivered as separate,
focused layers.

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

## Guarantees

- Exact identifiers remain available as `EnvironmentId`, `BackendId`,
  `ProtocolId`, and `RouteId` unions.
- `defineApplicationMarquette` returns its input without allocation or
  normalization.
- The published application-contract schema is available from
  `@vizejs/marquette/schema`.
- Every optional contract field documents its default in JSDoc.
- Package builds enforce a 1 KiB gzip budget for the runtime entry.

Marquette describes components and environments; it does not author UI
components. Public Vize UI components use real `.vue` SFC files as their
canonical source, never authored `h()` calls or handwritten render functions.
