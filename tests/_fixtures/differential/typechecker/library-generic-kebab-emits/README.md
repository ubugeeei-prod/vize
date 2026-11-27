# Kebab emits of packaged generic components

Hal ([Hal-Spidernight](https://github.com/Hal-Spidernight)) reported
[`@row-clicked` payloads of `bootstrap-vue-next`'s `BTable` typed as `any`](https://github.com/ubugeeei-prod/vize/issues/13788).

A generic `<script setup>` component shipped as `.d.ts` keeps Vue's
`EmitsToProps` spelling in its `__ctx.props`: an authored kebab emit is
`onRow-clicked`, not `onRowClicked`. The fixture checks that an authored kebab
listener reads that key, while a camel emit, a non-generic packaged component
and a local generic SFC with the same emit keep their payloads.
