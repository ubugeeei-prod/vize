import fs from "node:fs";
fs.writeFileSync(
  "package.json",
  JSON.stringify({
    private: true,
    type: "module",
    dependencies: {
      vize: "0.602.0",
      "@vizejs/native": "0.602.0",
      vue: "3.5.42",
      typescript: "6.0.3",
      "typescript-native": "npm:typescript@7.1.0-beta",
    },
  }),
);
