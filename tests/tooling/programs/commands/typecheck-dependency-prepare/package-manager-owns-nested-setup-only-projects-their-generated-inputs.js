if (managerArgs[0] === "install") {
  fs.mkdirSync("node_modules", { recursive: true });
} else {
  fs.mkdirSync(".nuxt", { recursive: true });
  fs.writeFileSync(".nuxt/tsconfig.json", "{}\n");
}
