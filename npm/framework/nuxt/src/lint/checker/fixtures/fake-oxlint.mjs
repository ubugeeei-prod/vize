const argv = process.argv.slice(2);
const message = argv.join(" ");

process.stdout.write(
  `${JSON.stringify({
    diagnostics: [
      {
        code: "vize(nuxt/error)",
        filename: argv.at(-1),
        labels: [{ span: { column: 2, line: 1 } }],
        message,
        severity: "error",
      },
      {
        code: "vize(nuxt/warning)",
        filename: argv.at(-1),
        labels: [{ span: { column: 4, line: 3 } }],
        message: "warning from fake oxlint",
        severity: "warning",
      },
    ],
    number_of_files: 1,
  })}\n`,
);
process.exitCode = 1;
