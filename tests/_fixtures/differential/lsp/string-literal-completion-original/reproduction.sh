mkdir repro && cd repro
npm i -D vize@0.432.0 typescript@7.0.2 vue@3.5.43
printf '{\n  "compilerOptions": { "module": "ESNext", "moduleResolution": "Bundler", "strict": true, "noEmit": true, "allowImportingTsExtensions": true }\n}\n' > tsconfig.json
printf 'export type MessageKey = "form.name" | "form.help";\nexport const t = (key: MessageKey): string => key;\n' > messages.ts
cat > Page.vue <<'EOF'
<script setup lang="ts">
import { t } from "./messages.ts";

const title = t("form.name");
</script>

<template>
  <h1>{{ title }}</h1>
  <p>{{ t("form.help") }}</p>
</template>
EOF
# zero-based positions right after the opening quote: in the template, in <script setup>
node lsp-req.mjs . Page.vue textDocument/completion 8:11 3:17
