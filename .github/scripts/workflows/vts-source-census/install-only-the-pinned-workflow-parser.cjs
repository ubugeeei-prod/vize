require("node:fs").writeFileSync(
  "package.json",
  JSON.stringify({ private: true, dependencies: { yaml: "2.9.0" } }),
);
