#!/usr/bin/env node
if (process.argv[2] !== "lint") process.exit(2);
process.stdout.write(
  JSON.stringify([
    {
      file: "App.vue",
      messages: [
        {
          ruleId: "vue/no-unused-components",
          severity: "warning",
          line: 1,
          column: 11,
          endLine: 2,
          endColumn: 12,
          message:
            "[vize:vue/no-unused-components] Component 'UploaderFile' is registered but never used in template",
        },
        {
          ruleId: "vue/no-unused-components",
          severity: "warning",
          line: 1,
          column: 11,
          endLine: 2,
          endColumn: 12,
          message:
            "[vize:vue/no-unused-components] Component 'UploaderFiles' is registered but never used in template",
        },
      ],
    },
  ]),
);
