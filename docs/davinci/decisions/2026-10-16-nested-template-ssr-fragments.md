# Nested template branches retain their SSR fragment

The [App inspector audit](https://github.com/ubugeeei-prod/vize/pull/9230) found a real discrepancy
in Elk's complete `NavUser.vue` output. Executing the retained original/current/official SSR
render functions with real Vue helpers showed that current Vize added a parent attribute to
a native button that both the old compiler and official Vue left inside a Fragment.
The audit records the incorrect initial classification and complete runtime observations.

The minimal template is:

```vue
<template v-if="outer">
  <button v-if="inner">a</button>
  <button v-else>b</button>
</template>
```

With `outer=true`, `inner=false`, and parent `data-owner="parent"`, the original current
compiler rendered `<button data-owner="parent">b</button>`. Official Vue renders
`<!--[--><button>b</button><!--]-->`. The missing anchors affect hydration, and passing
attributes across that Fragment changes component attribute ownership.

The public v0.457.0 macOS executable reproduces the discrepancy. Its archive SHA-256 is
`f6016dfc0ecce3dd631b1621613527071a2f461ad069f4b24040ed23b3249f75`, independently checked
against the release checksum. The complete minimized comparison is retained in
`fixtures/nested-template-ssr-fragments-20261016/published-0.457.0.json.gz`.

The AST emitter treated a single nested `If` node like a single physical root. The string
plan made the same decision from a child count of one. Both now retain a Fragment around
that conditional child list. The existing Fragment boundary prevents `_attrs` inheritance
by its descendants. A single physical element and a loop retain their existing ownership:
the element inherits attributes, while the loop owns its own anchors.

The regression compiles five templates with both regular and Vapor SSR options on both
the selected string-plan route and explicitly retained AST route. It requires actual native
plan admission and compares complete code, preamble, and source map. The existing compiler
fixture battery also includes the two nested-root shapes across its full option sets.

The existing scoped SSR action captures all ten source-built module pairs, executes both
routes against actual pinned Vue 3.6.0-beta.10 SSR, and compares complete HTML against the
official compiler for all four `outer`/`inner` assignments. Negative checks require fragment
anchors and no inherited button attribute; positive controls require a single physical root
to keep that attribute and a root loop to keep exactly its own fragment anchors. Complete
modules and all 40 runtime observations are uploaded by the existing action.

No inspector budget or expected compiler error count changes. This source correction still
requires exact-head Actions, actual merge, and the coordinated full App qualification before
any separately reviewed inspector baseline decision. The runtime comparison exercises SSR
options for both client modes; it does not claim a fresh browser hydration run by itself.

Refs #8849, #3957.
