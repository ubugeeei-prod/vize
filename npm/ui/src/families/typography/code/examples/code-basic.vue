<!-- Inline code within prose next to a preformatted block of shell commands. -->
<script setup lang="ts">
import { Code } from "../code.ts";
</script>

<template>
  <div>
    <p>
      Install the package, then import <Code>Button</Code> from
      <Code tone="accent">@vizejs/ui</Code>.
    </p>
    <pre><Code variant="block" size="sm">npm install @vizejs/ui
npx vize lint src</Code></pre>
  </div>
</template>
