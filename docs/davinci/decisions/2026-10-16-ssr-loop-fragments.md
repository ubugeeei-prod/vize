# SSR loop callbacks retain their nested ranges

The nested-template audit in [PR #9244](https://github.com/ubugeeei-prod/vize/pull/9244)
exposed an adjacent loop boundary. This template needs separate ranges for the loop,
each item, and the nested conditional branch:

```vue
<template v-for="n in 2">
  <template v-if="outer">
    <button v-if="inner">a</button>
    <button v-else>b</button>
  </template>
</template>
```

With both conditions true, public Vize 0.457.0 renders
`<!--[--><button>a</button><button>a</button><!--]-->`. Pinned Vue 3.6.0-beta.10
also surrounds each item and each conditional child list with Fragment anchors.
The missing ranges affect the node structure expected during hydration even though
the visible buttons look the same. All four condition assignments reproduce the
discrepancy with real Vue SSR helpers. Complete public and official generated modules,
HTML observations, and the independently verified release archive checksum are retained
in `fixtures/ssr-loop-fragments-20261016/published-0.457.0.json.gz`.

Vue's SSR `ssrProcessFor` makes two separate decisions. The loop and its item wrappers
honor the incoming `disableNestedFragments` setting. Each item needs a Fragment unless
its logical content is exactly one element, including a component or slot outlet.
The callback body then uses `processChildrenAsStatement`, which starts a fresh fragment
context. A loop does not suppress all descendant conditional and list ranges.

Both Vize emission routes now follow those rules. The AST route looks through retained
structural-template carriers when classifying the logical item content. The native
string-plan route classifies the direct child segments. The old keyed-template/count
heuristic is unnecessary: keys do not change whether a single element owns an item.

The fresh callback context matters inside TransitionGroup. Regular TransitionGroup still
suppresses its direct loop/item wrappers, while the callback's nested conditions retain
their own ranges. Vize's existing Vapor TransitionGroup policy retains nested ranges;
the Vapor boundary control therefore uses official SSR for an equivalent physical
`section` wrapper as its explicit oracle. The regular boundary control uses the original
TransitionGroup template unchanged.

The existing native SSR action captures both emission routes, requires native admission,
and compares complete code, preambles, and source maps for regular and Vapor options.
The real Vue runtime harness compares all four condition assignments for the new loop
cases, including nested loops, single physical roots, keyed bodies, text, empty bodies,
and TransitionGroup. It also executes the retained public module as a negative control,
so an implementation that reproduces the old HTML cannot pass by agreeing with itself.

These checks cover emitted SSR behavior. They do not substitute for browser hydration
or full App qualification. No inspector budget, baseline, or expected error count changes.
