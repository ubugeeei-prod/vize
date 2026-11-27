## One disabled state for the whole field

`InputGroup` is a named group. Its input, textarea, and button read that
disabled state, so a prefix, the field, and the action stop together. shadcn's
Input Group is a styling composition. This one also owns the disabled contract.

```vue
<script setup lang="ts">
import { ref } from "vue";

import {
  InputGroup,
  InputGroupAddon,
  InputGroupButton,
  InputGroupInput,
} from "@vizejs/ui/input-group";

const site = ref("vize.dev");
</script>

<template>
  <InputGroup label="Website">
    <InputGroupAddon>https://</InputGroupAddon>
    <InputGroupInput v-model="site" type="url" />
    <InputGroupButton @press="copy">Copy</InputGroupButton>
  </InputGroup>
</template>
```

Parts stay headless. Affix them with CSS on `[data-vize-ui="input-group"]`.
