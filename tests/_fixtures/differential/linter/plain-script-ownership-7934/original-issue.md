### Area

Linter, `script/no-export-in-script-setup` and `script/require-typed-ref` on plain `.ts` modules

### Version

`vize` 0.432.0

### Minimal reproduction

`src/config.ts`

```ts
import { ref } from "vue";
import type { Ref } from "vue";

async function loadSettings(): Promise<{ theme: string }> {
  return { theme: "light" };
}

const settings = await loadSettings();

export const theme = settings.theme;

export function getTheme(): string {
  return theme;
}

export const current: Ref<string | null> = ref(null);
```

`vize.config.json`

```json
{ "linter": { "rules": { "script/no-export-in-script-setup": "warn", "script/require-typed-ref": "warn" } } }
```

```sh
vize lint src
```

### Actual

```
config.ts:10:1  script/no-export-in-script-setup Unexpected `export` in `<script setup>`: it is compiled into setup() and the export is meaningless
config.ts:12:1  script/no-export-in-script-setup Unexpected `export` in `<script setup>`: ...
config.ts:16:1  script/no-export-in-script-setup Unexpected `export` in `<script setup>`: ...
config.ts:16:44 script/require-typed-ref         This ref() should have an explicit type argument.
```

1. **`script/no-export-in-script-setup`** reports every export of a plain `.ts` ES module that uses top-level `await`. `is_script_setup_block` treats "has a top-level `await`" as proof that the program is a `<script setup>` block, but top-level `await` is valid in any ES module (for example a module that loads locale messages or config before it is imported). The rule's default severity is `error`.
2. **`script/require-typed-ref`** asks for a type argument on `ref(null)` although the binding is annotated as `Ref<string | null>`, so the ref is already explicitly typed (adding `ref<string | null>(null)` would just repeat the annotation).

### Expected

1. The rule only runs on SFC `<script setup>` blocks (the linter knows when it is linting a `.ts` / `.js` file), or at least does not use top-level `await` as the marker outside SFCs.
2. A declared type annotation on the binding satisfies `script/require-typed-ref`.

### Environment

- OS: macOS 26.4.1
- Architecture: arm64
- Node.js: 26.8.1
- Package manager: npm 11.19.0
