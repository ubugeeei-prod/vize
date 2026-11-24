#!/usr/bin/env node
if (process.argv[2] !== "lint") process.exit(2);
process.stdout.write(
  JSON.stringify([
    {
      file: "App.vue",
      messages: [
        {
          ruleId: "vue/require-v-for-key",
          severity: "error",
          line: 3,
          column: 6,
          endLine: 3,
          endColumn: 27,
          message:
            "[vize:vue/require-v-for-key] Elements in iteration expect to have 'v-bind:key' directives.",
        },
      ],
    },
  ]),
);
