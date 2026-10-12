# Preserve complete authored definition identity

Owner: [#3952](https://github.com/ubugeeei-prod/vize/issues/3952).

Release Matrix run 38147644073, shard 1, source
`1d988f6bcac8afc6a0fcad9b76b0d5f2da27be71` returned two identical
definitions for tailwind-config-viewer's `sectionComponent` method. Both
mapped to the pinned `Canvas.vue` URI and complete range 102:4–102:20.
The existing whole-vector oracle correctly refused this response.

Deduplicate after virtual locations map back to authored sources, using exact
URI and full range identity. Keep the first occurrence and preserve the order
of every distinct target. Preserve null, scalar and array response shapes.
Native declarations and worker ownership stay intact; no provider fallback,
new pipeline stage or request serialization is introduced.

A minimized Options API corpus fixture exercises the actual native definition
service repeatedly. Separate conversion laws cover nonadjacent duplicates,
different URIs and both range endpoints. The unchanged real-project authored
oracle and full Linux lifecycle remain required qualification. Local source
checks or incomplete lifecycle fields are not diagnostic acceptance evidence.
