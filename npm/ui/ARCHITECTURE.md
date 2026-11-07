# UI source architecture

`@vizejs/ui` is a source-owned Vue component system. Public components are real
single-file components, while behavior, styles, documentation, and verification
remain independently consumable.

## Source roots

- `src/components/` contains public component units. This is the only home for
  new public components.
- `src/families/` contains the legacy catalogue while it is migrated one unit at
  a time. Moving a component must preserve its public package subpath.
- `src/catalog/` composes colocated component metadata. It must not duplicate a
  component's file inventory.
- `src/authoring/`, `src/conformance/`, `src/resolver/`,
  `src/source-registry/`, and `src/story-testbed/` are package tooling rather
  than component implementations.

## Component unit

Each directory under `src/components/` owns one public package subpath:

```text
button/
├── button.vue              # public SFC and accessible behavior
├── index.ts                # public exports
├── types.ts                # public types
├── keyboard.ts             # isolated behavior logic, when needed
├── styles.css              # optional native-CSS visual layer
├── metadata.ts             # catalogue and build metadata
├── button.behavior.md      # normative behavior table
├── examples/
│   └── basic.vue
├── stories/
│   └── button.art.vue
└── tests/
    ├── behavior.test.ts
    └── button.types.test-d.ts
```

The names describe responsibility rather than build tooling. A reader can find
the public SFC, its types, its visual layer, and its evidence without searching
the rest of the repository.

## Styling boundary

Component behavior is headless. Visuals are optional native CSS assets built
from `styles.css`; Tailwind, CSS-in-JS, and runtime styling libraries are not
dependencies. Stable `data-vize-ui`, `data-state`, and typed visual attributes
form the selector contract. Theme tokens and component custom properties form
the customization contract.

CSS is authored with semantic selectors and native nesting. Component states
are nested beneath one `data-vize-ui` hook; BEM and long flat selectors are
forbidden. Lightning CSS lowers that source to the declared browser floor, so
consumers do not inherit a CSS toolchain requirement.

Presets live in cascade layers. They may change geometry, typography, depth,
and motion, but must preserve focus, forced-colors, reduced-motion, and pointer
target policies.

## Acceptance boundary

A component unit is incomplete until its catalogue entry points only inside the
unit and the declared behavior, type, renderer, SSR, hydration, tree-shaking,
and bundle gates pass. `component-unit-layout.test.ts` prevents the directory
contract from drifting.
