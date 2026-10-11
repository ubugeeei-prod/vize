# Direct SSR client guards

Related: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830),
[#8595](https://github.com/ubugeeei-prod/vize/pull/8595).

`ssr/no-browser-globals-in-ssr` has a framework-independent, default-enabled
SSR context. It does not receive the linter's Nuxt preset or a project realm
witness. Only a truthy, nonoptional static `import.meta.client` access, including
equivalent logical negations, proves that its arm does not execute in SSR.
Nuxt replaces this flag with false on the server; a host without the flag leaves
it undefined, which is also falsy. Nuxt documents these flags as
[statically injected values](https://nuxt.com/docs/4.x/api/advanced/import-meta).

Do not infer client execution from `import.meta.server`. A host without Nuxt's
injected flag evaluates `!import.meta.server` as true on the server. Both arms
remain checked until actual framework context establishes a stronger contract.
Vite's built-in server flag is
[`import.meta.env.SSR`](https://vite.dev/guide/env-and-mode#built-in-constants);
it does not establish an `import.meta.server` contract here.
This does not require a new framework field for the two reported npmX cases:
both use positive `import.meta.client` guards.

Keep condition reads, unsafe opposite arms, lexical lookalikes, existing `typeof`
guards, top-level reads, and indirect boolean/ref chains diagnostic. The paired
differential fixture preserves the whole unsafe packets and LF/CRLF offsets.
The indirect npmX `hasEmptyPayload` to `shallowRef.value` chain remains unproved.
