# Musea copied component tags

Issue: [#8468](https://github.com/ubugeeei-prod/vize/issues/8468).

The Props editor used a gallery display title as a Vue tag. An Art importing
`AliasButton` with title `Props / Probe` rendered its actual native preview
correctly, but copied `<Props / Probe ...>`. The same real Vize compiler rejected
the full copied SFC with `UnexpectedSolidusInTag`; browser module import failed.

## Decision

Reuse the existing `parseScriptSetupForArt` and `resolveArtComponent` result in
the palette producer. Its optional `componentTagName` keeps the actual binding
separate from the display title. The Props editor uses that field when present;
old/unresolved palette responses retain their existing fallback. Display titles,
palette JSON/TypeScript exports and component resolution semantics stay intact.

Static detail generation already captures the same palette handler, so it
preserves the field without a second naming rule. No title sanitization or
independent import-name inference is introduced in the editor.

Ordinary isolated usage keeps its existing template-only output. Its consumer
provides the demonstrated component in script scope. Complete copied SFCs,
including the default-as import alias case, are executed byte-for-byte. This
change does not broaden the existing script/import ownership contract.

## Reproduction and qualification

The source contract processes actual Art files through the native parser and
checks complete palette responses for five forms: an imported alias with a
display title, an ordinary imported component, a source argument, a component
attribute, and inline Self. All original palette tests remain intact.

The new native browser contract runs the real Vize and Musea plugins with
unmocked routes. It edits ordinary and literal `constructor`/`hasOwnProperty`
props, adds a custom dotted name and own `__proto__` value, checks the full
generated preview/current values, then compiles and mounts the copied usage
with the real native compiler. Vue's existing direct reserved-key behavior
remains explicit. Original native Self and browser/clipboard controls stay
mandatory; the new entry is appended to the native Actions command and its
observations use the existing always-uploaded native artifact.

The contract acquires its server and browser inside the guarded test body.
Receipt failures still run browser/server/temp cleanup through nested finalizers.
A genuine missing-browser launch also retained its failure and removed its
temporary project; this local cleanup check does not substitute for Actions.

Local diagnostic evidence uses the published **0.440.0** macOS native package
with current-source UI. The package SHA-512 integrity was verified, with
tarball SHA-256
`1b5dd9963321fe3657e7cfb2aff79e9a4985d7524dcf73bd2700d8aa267a39b5`
and binary SHA-256
`e159f541bc865ac0295f6207b277be240bb4a551028e3f03729e352a42f9900b`.
The supported native path override loads the official Vize loader once and is
restored before unrelated Vite/Rolldown bindings load. This is published-native
diagnostic evidence, not current Rust source qualification or release acceptance.

The [whole original failure packet](../../../tests/_fixtures/differential/musea/usage-component-tag-before.tar.gz)
retains native source/API/clipboard/compilation/DOM bytes and custody, the actual
failure after reverting only the Props editor expression, and the successful
restored-browser observation. Packet SHA-256:
`dca396d230814c9c7717a0bbdf4851fb3855aa76c3db2d6ac82db7e59b71c45b`.
Browser console observations remain complete; Chromium's existing sandbox
advisories are retained, while page errors and Vue warnings must be empty.
Fresh exact-head source/native/browser/security Actions, protected delivery and
successful publication remain required. Preparing this successor grants none
of that delivery credit and does not mutate the qualified predecessor heads.

## Remaining work

The existing component-prop fallback inferred only the first field from the
compact one-line `defineProps<{ label?: string; constructor?: string;
hasOwnProperty?: string }>()` vector. The complete compact input and failed
palette law are retained in the packet. The same declared multiline fields are
used in this tag contract. That prop-inference omission remains unfinished;
this change does not alter `analyzeSfcFallback` or claim complete arbitrary prop
extraction. Already documented Vue reserved-name boundaries also remain intact.
