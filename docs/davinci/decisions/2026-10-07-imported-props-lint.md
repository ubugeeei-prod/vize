# Imported props and undefined template references

Issue: [#7912](https://github.com/ubugeeei-prod/vize/issues/7912).

The single-file `vue/no-undefined-refs` pass cannot prove an identifier is
absent from `defineProps` when the prop shape depends on imported types. It
therefore defers undefined-reference findings for that template until a
provider supplies a complete prop shape. The existing provider completeness
metadata takes precedence over the local imported-type fallback.

The fallback follows imported root shapes, local aliases, interface heritage,
intersections and Vue-supported property utilities using the existing type
definitions. Imported types used only as prop values do not open the shape.
Closed local, inline, runtime and empty shapes retain unknown-name diagnostics.
No filesystem reads or additional script analysis are introduced.

The issue's original SFC and external interface are retained in the regression
corpus. Controls cover `withDefaults`, aliases, heritage and utility shapes,
plus exact spans for unknown names in closed shapes. This is a conservative
single-file policy: an unrelated typo can remain undiagnosed while the props
shape is unresolved. Cross-file lint precision remains a future provider
improvement; this legacy regression adds no Davinci native acceptance credit.
