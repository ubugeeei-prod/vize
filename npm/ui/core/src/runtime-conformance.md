# Runtime conformance contract

Every source-owned `@vizejs/ui` component must pass the same cross-runtime
gates. Compiler coverage discovers SFCs recursively. Runtime fixtures describe
the component-specific accessible markup, and a coverage assertion rejects any
new SFC that does not add its SSR and hydration evidence.

| Lane         | Executable evidence                                                | Required invariant                                                                        |
| ------------ | ------------------------------------------------------------------ | ----------------------------------------------------------------------------------------- |
| DOM          | Native Vize SFC compilation                                        | No errors, warnings, empty modules, or Vapor markers                                      |
| SSR          | Native Vize SFC compilation plus concurrent `renderToString` tests | Stable request-local markup, accessible server semantics, and no Vapor fallback           |
| Hydration    | SSR markup hydrated by a fresh client application                  | No warning, error, or root-node replacement                                               |
| Vapor        | Native Vize SFC compilation                                        | A real Vapor component marker with no diagnostic fallback                                 |
| Tree shaking | Production consumer bundles through public package exports         | Root and subpath parity, unused family elimination, exact CSS retention, and gzip budgets |

The hydration suite covers every currently shipped SFC, including the
renderless deterministic-ID provider. The dedicated ID suite additionally
exercises repeated and concurrent requests, nested providers, islands with
distinct seeds, and byte-stable hydration IDs. Interaction suites remain the
authority for keyboard, pointer, focus, form, controlled/uncontrolled, and
accessible-name behavior.

## Deliberate non-claim

Vize currently falls back to standard SSR when Vapor and SSR are requested
together. This contract does not disguise that fallback as support. Native
Vapor SSR, streaming, async boundaries, Teleport, independently hydrated roots,
and islands remain a release-blocking item in issue #3134. Once the compiler
ships that lane, it must be added here without weakening the DOM SSR gate.
