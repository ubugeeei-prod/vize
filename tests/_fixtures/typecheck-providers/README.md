The matrix normally uses the published `vue-tsc` package pinned by the repository.

Vuetify uses a clean build of the official source at
[`b77bba35e9fa78b4bdab6baf8588a4e4c9469ce7`](https://github.com/vuejs/language-tools/commit/b77bba35e9fa78b4bdab6baf8588a4e4c9469ce7).
The latest published 3.3.14 release corrupts `https://` in the inference copy of
bound template props and stops before semantic checking. The official
[fix #6371](https://github.com/vuejs/language-tools/pull/6371) preserves those
expressions and has positive and negative regression controls. No published
release containing that fix was available when this provider was pinned.

The descriptor seals the complete source commit and relevant Git blobs. CI uses
the upstream frozen lockfile and build command, then calls the upstream
`run(tscPath)` entry with the matrix's existing TypeScript 6.0.3 peer. This keeps
the runtime compiler peer the same as the published oracle even though the
upstream build dogfoods a different compiler through its workspace override.

Preparation records the descriptor, lockfile, compiler peer and emitted module
hashes. The comparator rechecks the source and outputs immediately before the
baseline and mutation controls. Reports name the source provider explicitly.
These results qualify that pinned upstream source build; they do not qualify the
unmodified published 3.3.14 package. The compared application sources, their
tsconfigs and all diagnostics stay intact.
