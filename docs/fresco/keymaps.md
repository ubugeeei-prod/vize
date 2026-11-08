# Configurable keymaps

`useKeymap` maps Fresco's normalized key stream to typed application commands. A binding may be a
single chord, an Emacs-style multi-chord, or a Vim-style key sequence. Modifier matching is exact,
so a plain `g` binding does not consume `Shift+G`.

```ts
import { defineKeymap, useKeymap } from "@vizejs/fresco";

const keymap = defineKeymap([
  {
    command: "save",
    sequence: [
      { key: "x", ctrl: true },
      { key: "s", ctrl: true },
    ],
  },
  { command: "top", sequence: [{ key: "g" }, { key: "g" }] },
] as const);

useKeymap({
  keymap,
  sequenceTimeout: 1_000,
  onCommand(command) {
    if (command === "save") save();
    if (command === "top") scrollTo(0);
  },
});
```

Keymaps are ordinary readonly values. Applications can provide them through Vue injection, switch
them reactively for modes such as Vim insert/normal state, or layer component-specific commands on
top of a shared map. Incomplete sequences are cleared after `sequenceTimeout`; `reset()` is also
available when a mode or focus scope changes.
