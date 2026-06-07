# Ecosystem Roadmap

> [!WARNING]
> This page is an **aspirational vision**, not a commitment or a status report.
> **None of the packages below are implemented yet.** They are recorded here so the
> intended shape of the Vize ecosystem is written down and reviewable. Anything on
> this page may change, be renamed, be merged with another item, or be dropped.
> Nothing here is production-ready, alpha-supported, or even started unless and until
> it appears in the [stability tiers](../content/stability.md) and the
> [production-readiness checklist](./production-readiness.md).

The current shipping surface is the Vue toolchain (compiler, lint, type-check,
formatter, LSP, Vite plugin) described in the [Vue Parity Matrix](./vue-parity-matrix.md).
This roadmap describes a longer-term, multi-year application-framework ecosystem on
top of that toolchain.

## Guiding principles

Every ecosystem package is expected to follow the same principles as the toolchain:

- **Rust-native engine** — hot paths in Rust, exposed through napi / wasm.
- **High performance** — performance budgets, no avoidable allocation or overhead.
- **Vendor-free** — no lock-in to a single host or paid backend.
- **Fully typesafe, strong type inference** — types flow end-to-end with minimal annotation.
- **Maximize DX** — first-class editor, DevTools, and error messages.
- **Opt-in** — ships behind experimental / incubating tiers until it earns promotion.

## Proposed packages

### Routing & navigation

| Package               | Inspiration                 | One-line intent                                                  |
| --------------------- | --------------------------- | ---------------------------------------------------------------- |
| **Vize Router**       | Vue Router + Navigation API | Transition-first SPA router built on the browser Navigation API. |
| **FileSystem Router** | Nuxt / file-based routing   | Filesystem-driven routes on top of Vue Router / Vize Router.     |

### State, data, and effects

| Package        | Inspiration           | One-line intent                                               |
| -------------- | --------------------- | ------------------------------------------------------------- |
| **Vize Store** | Recoil / Jotai        | Atomic, composable state management.                          |
| **Vize Query** | TanStack Query        | Async data loader with caching, invalidation, and suspense.   |
| **Vize Saga**  | Redux Saga (next-gen) | Declarative, typesafe side-effect / async-flow orchestration. |

### Full-stack framework

| Package / Surface                | Inspiration   | One-line intent                                                                                                                                                                   |
| -------------------------------- | ------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Vize (framework)**             | Nuxt / Next   | App framework on the Vite Environment API + UnJS, Rust engine, end-to-end typesafe.                                                                                               |
| **Server Components**            | RSC           | Server-rendered components with a clear server/client boundary.                                                                                                                   |
| **Server Actions**               | RSC actions   | Typesafe server mutations callable from the client.                                                                                                                               |
| **SEO framework**                | —             | Metadata, structured data, sitemap, and crawlability primitives.                                                                                                                  |
| **Zero-JS prerenderer (Island)** | Astro islands | Island-architecture prerendering that emits zero client JS for static islands. _(also tracked as a production-ready target in [production-readiness](./production-readiness.md))_ |

### API & transport

| Package             | Inspiration  | One-line intent                                     |
| ------------------- | ------------ | --------------------------------------------------- |
| **Vize RPC**        | tRPC         | End-to-end typesafe RPC with no schema duplication. |
| **Vize GraphQL**    | —            | Typed GraphQL client/server integration.            |
| **void sdk bridge** | VoidZero SDK | Bridge to the VoidZero / Vite+ SDK surface.         |

### Native & cross-platform

| Package         | Inspiration  | One-line intent                                            |
| --------------- | ------------ | ---------------------------------------------------------- |
| **Vize Native** | React Native | Build native apps from Vue components with a Rust runtime. |

### UI

| Package                    | Inspiration         | One-line intent                                         |
| -------------------------- | ------------------- | ------------------------------------------------------- |
| **Headless Accessible UI** | Radix / Headless UI | Unstyled, fully accessible (a11y) component primitives. |

### Tooling, testing & observability

| Package / Surface             | Inspiration     | One-line intent                                                              |
| ----------------------------- | --------------- | ---------------------------------------------------------------------------- |
| **Vize DevTools**             | Vue DevTools    | Inspector for components, state, router, query, and performance.             |
| **Observability**             | OpenTelemetry   | Tracing / metrics / logging framework for Vize apps.                         |
| **Musea VRT**                 | Storybook + VRT | Visual regression testing with local or remote storage backends.             |
| **Vue Markdown (Ox Content)** | —               | Markdown authoring/components for Vue, powered by Oxc-based content tooling. |

## Near-term, actionable work

Unlike the packages above, these are extensions of the existing toolchain and can be
picked up immediately:

- **More SSR testing** — broaden SSR compiler/lint fixture and snapshot coverage.
- **More Options API testing** — continue expanding Options API parity fixtures beyond
  the initial set.

## How items graduate

An ecosystem package moves off this page and into a real tier only when it has: a
published package name, documented install/usage, CI for build/install/runtime, an
owner, and the experimental/incubating contract recorded in
[stability.md](../content/stability.md). Production-ready promotion additionally
requires clearing every gate in the [production-readiness checklist](./production-readiness.md).
