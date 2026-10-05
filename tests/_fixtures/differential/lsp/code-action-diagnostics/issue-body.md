## Version

- `vize` 0.432.0 (npm, `node_modules/.bin/vize lsp --stdio`), serverInfo `vize-maestro 0.432.0`
- Node 26.6.0, macOS (arm64)

## Minimal reproduction

`vize.config.json`:

```json
{
  "linter": { "preset": "opinionated" },
  "languageServer": { "lint": true, "codeActions": true }
}
```

`src/App.vue`:

```vue
<template>
  <img src="/logo.png">
</template>
```

`vize lint src/App.vue` reports two warnings on the same span (`2:3`): `a11y/alt-text` and `vue/html-self-closing`.

## Steps

Drive `vize lsp --stdio` (`initialize` with the project as `rootUri`, `initialized`, `textDocument/didOpen` for `src/App.vue`). The server publishes:

```json
{"code":"a11y/alt-text","range":{"start":{"line":1,"character":2},"end":{"line":1,"character":23}},"message":"<img> elements must have an alt attribute ..."}
{"code":"vue/html-self-closing","range":{"start":{"line":1,"character":2},"end":{"line":1,"character":23}},"message":"Void element should be self-closing ..."}
```

Then, for each of them, `textDocument/codeAction` with that diagnostic's `range` and `context.diagnostics: [<that diagnostic>]`.

## Actual

Both requests return the action for `a11y/alt-text`:

```
codeAction for [a11y/alt-text] ->
   {"title":"Suppress with @vize:forget (a11y/alt-text)","kind":"quickfix","edit":{"changes":{"file:///tmp/repro/src/App.vue":[{"newText":"  <!-- @vize:forget a11y/alt-text -->\n","range":{"start":{"line":1,"character":0},"end":{"line":1,"character":0}}}]}}}

codeAction for [vue/html-self-closing] ->
   {"title":"Suppress with @vize:forget (a11y/alt-text)","kind":"quickfix","edit":{"changes":{"file:///tmp/repro/src/App.vue":[{"newText":"  <!-- @vize:forget a11y/alt-text -->\n","range":{"start":{"line":1,"character":0},"end":{"line":1,"character":0}}}]}}}
```

With both diagnostics in `context.diagnostics` (what VS Code sends when the cursor is on the tag), only one action comes back, again for `a11y/alt-text`:

```
codeAction with both diagnostics in context ->
   {"title":"Suppress with @vize:forget (a11y/alt-text)","kind":"quickfix"}
```

In an editor, choosing "suppress" on the `vue/html-self-closing` squiggle inserts a comment for a different rule, so the warning stays, and there is no way to suppress `vue/html-self-closing` from the light bulb at all. The returned actions also carry no `diagnostics` field, so the client cannot tell which diagnostic an action resolves.

## Expected

The action is derived from the diagnostics in `context.diagnostics` (here: `Suppress with @vize:forget (vue/html-self-closing)`), and each action lists the diagnostic it resolves in `CodeAction.diagnostics`. A request whose range covers both diagnostics should return one action per rule.

## Why

LSP 3.17 `CodeActionContext.diagnostics` is "an array of diagnostics known on the client side overlapping the current provided range" and is the input that identifies what to fix; ESLint's VS Code server and the TypeScript server build quick fixes per diagnostic code from that array. Two rules flagging the same element is common in templates (a11y + style rules on one tag), so matching by range alone picks the wrong rule.
