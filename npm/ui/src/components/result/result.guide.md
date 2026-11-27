## Reserve alerts for failures

`Result` is a status page. `error` is `role="alert"` so assistive tech interrupts.
`success`, `info`, `warning`, `not-found`, and `empty` are `role="status"` so a
missing page does not shout. The title is the accessible name.

```vue
<script setup lang="ts">
import { Result } from "@vizejs/ui/result";
</script>

<template>
  <Result status="not-found" title="Page not found" description="That URL is not on this site." />
  <Result v-slot="{ status }" status="error" title="Payment failed">
    <button type="button">Retry {{ status }}</button>
  </Result>
</template>
```

Style `[data-vize-ui="result"]` and `[data-status]`. The component ships no CSS.
