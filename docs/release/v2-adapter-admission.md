# V2 Adapter Admission

The bounded adapter claims below implement the admission rules in
[#13019](https://github.com/ubugeeei-prod/vize/issues/13019). Each claim has a
semantic owner, an executable compatibility matrix, and a public maturity label.
Admission requires the owning workflow and repository-required checks to pass
for the exact candidate source. The [checked stability table](../content/stability.md#package-support-tiers)
and package metadata continue to own release maturity.

| Adapter                | Semantic owner                                             | Public maturity                                                       | Executable host matrix                                                                                                                                                            |
| ---------------------- | ---------------------------------------------------------- | --------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `prettier-plugin-vize` | [formatter/vize_glyph](../../crates/vize_glyph)            | Experimental; independent formatter v0 line                           | [Prettier consumer](../../.github/workflows/09-prettier-plugin.yml): Prettier 3.9.10 on Node 22.0.0 and 24.14.0                                                                   |
| `eslint-plugin-vize`   | [linter/vize_patina](../../crates/vize_patina)             | Experimental                                                          | [ESLint consumer](../../.github/workflows/09-eslint-plugin.yml): ESLint 10.13.0 on Node 22.13.0 and 24.14.0, each with TypeScript 4.8.4, 5.9.3 and 6.0.3                          |
| `@vizejs/babel-plugin` | [compiler/vize_atelier_jsx](../../crates/vize_atelier_jsx) | Compatibility preview; Vapor modes retain their experimental contract | [Babel host contracts](../../.github/workflows/08-babel.yml): Babel 7.29.7 and 8.0.9, ESM and CommonJS consumers on Node 24.14.0; the application-module row uses Vue 3.5.44 VDOM |

## Fresh consumers and application sources

The Prettier matrix installs the public tarball in an isolated consumer and
formats every SFC in the [Vite Musea](../../examples/vite-musea) and
[JSX/TSX](../../examples/jsx-tsx) example applications. Complete direct-native,
installed API, repeated API, and CLI outputs must agree. A rejected host option
must leave the copied application intact. Both Node versions execute the same
tarball, authenticated native image, and emitted test helper; the complete
application packet must agree across runtimes.

The ESLint matrix installs both public packages and runs every SFC from those
same applications through the standalone adapter and its original shared bridge.
The rule objects must be identical. Every native call, source, option, diagnostic,
range, suggestion, and fix retains its original order and must agree. Real host
parsing must succeed, autofix must reach a fixed point, and the original application
sources must remain intact. All six runtime/provider combinations execute this
application matrix with the same source-native artifact.

The Babel matrix compiles the original
[`AccessibleMedia.jsx`](../../examples/jsx-tsx/src/AccessibleMedia.jsx)
application module at its authored filename through all four packed host/entry
combinations. It verifies complete returned AST eligibility, native pass metadata,
the full authored source in the final source map, and exact HTML from the actual
Vue server renderer. Source and application-manifest digests identify the real
project input. This is a standalone VDOM application-module claim; complete
application builds require their environment's adapter qualification.

These are executable application checks in addition to the existing positive,
negative, boundary, source-authority, and unsupported-input cases. Their owning
workflows retain complete reports, including failed observations. Fixture presence
alone supplies no admission credit.

## Semantic authority and existing contracts

The adapters delegate formatting, lint rules, and compilation to their existing
shared native engines. The application gates add no independent formatter, rule,
typechecker, source-offset mapper, or compiler semantics. The original v1
diagnostic, source-map, edit, output, ordering, and exit-behavior oracles remain
exact. Repository-required checks continue to enforce the existing instruction,
size, compatibility, and correctness budgets.

An added host, dialect, environment, or framework claim must name its owner,
publish its maturity, and extend an executable fresh-consumer and real-project
matrix before admission. Successful qualification does not change a package's
maturity automatically. Experimental work retains its experimental label until
its complete promotion gate passes. Formatter and Musea retain their independent
v0 line; UI and Fresco retain their early-development contract.
