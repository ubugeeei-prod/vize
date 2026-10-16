# Practical virtual-list catalogue

Issue: #8374. This completes one missing public-entry example, not the whole
component/composable documentation audit.

The existing 123 composable examples did not include `virtual-list`. Its API
requires the source list, container bindings, wrapper bindings and measured row
size to work together; an import/signature alone does not show their relationship.

Add a complete SFC that searches 1,000 local guides. Native list scrolling,
first/last controls, rendered positions, empty results and search recovery show
the effect without a network service or browser capability mock. Fixed 40px rows
include their padding and borders. The example describes two-row overscan,
zero-based `scrollTo`, deterministic server output and component-owned observation.

The scroll region retains keyboard focus while rows are replaced. Search retains
input focus and returns to the first match. Rows expose their position and total
count, but the prose explicitly explains that browser Find and assistive technology
can only read rendered rows. Recommend search or pagination/full rendering where
all entries or focusable row controls must remain available.

Reuse the source-owned preview pipeline: the displayed SFC and compiled preview
are the same file, checked by the existing SHA/source alignment assertions.
Extend the existing browser capture with real Home/End/PageDown scrolling,
Enter/Space jump activation, actual 40px geometry, bounded DOM rows, ARIA counts,
filter/empty/recovery and retained focus. The normal Docs workflow captures initial
and interacted states at 720px and 360px, verifies hydration identity and checks
the rendered reference at desktop and mobile widths. Public acceptance must run
against the actual deployed source after merge; source CI alone is insufficient.

The real SFC exposed an existing public typing defect: `VirtualWrapperStyle`
was not assignable to Vue's `StyleValue`, so the advertised `v-bind="wrapperProps"`
failed Vue template checking. Both style interfaces now extend Vue's
`CSSProperties`, preserving their explicit overflow/display/geometry constraints.
Existing compile-only assertions check Vue binding compatibility and those narrow
types; the complete example also passes through the normal Vue SFC typecheck.
No runtime behavior, navigation, site renderer or upstream package is changed.
