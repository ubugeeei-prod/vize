const [f, side, mode] = process.argv.slice(2);
const rows = Object.fromEntries(
  require("fs")
    .readFileSync(f, "utf8")
    .split("\n")
    .filter((l) => /^[0-9]/.test(l))
    .map((l) => {
      const c = l.split(",");
      return [c[2], Number(c[0])];
    }),
);
console.log(
  `| ${side} | ${mode} | ${(rows.instructions / 1e9).toFixed(3)}G | ${(rows.cycles / 1e9).toFixed(3)}G | ${(rows["task-clock"] / 1e6).toFixed(1)} ms |`,
);
