/* Translate operations, preserving Vite+ tasks instead of inventing scripts. */
(() => {
  const managers = ["vp", "npm", "pnpm", "yarn", "bun", "aube", "jsr"];
  const local = {
    vp: "vp exec",
    npm: "npm exec --",
    pnpm: "pnpm exec",
    yarn: "yarn exec",
    bun: "bun x --no-install",
    aube: "aube exec",
  };
  const once = {
    vp: "vpx",
    npm: "npx",
    pnpm: "pnpm dlx",
    yarn: "yarn dlx",
    bun: "bun x",
    aube: "aube dlx",
  };
  const add = {
    vp: "vp install",
    npm: "npm install",
    pnpm: "pnpm add",
    yarn: "yarn add",
    bun: "bun add",
    aube: "aube add",
  };
  const managerLine = /^(?:vp|vpx|npm|npx|pnpm|yarn|bun|bunx|aube|aubr|aubx)\b/;

  function operation(command) {
    let match = /^(vp|npm|pnpm|yarn|bun|aube) (install|add)(?:\s+(.*))?$/.exec(command);
    if (match) {
      const args = match[3] ?? "";
      const tokens = args.split(/\s+/).filter(Boolean);
      if (
        tokens.some(
          (token) =>
            token.startsWith("-") && !["-D", "--save-dev", "-E", "--save-exact"].includes(token),
        )
      )
        return null;
      return { kind: tokens.length ? "add" : "install", args };
    }
    match =
      /^(?:vp exec|npm exec --|pnpm exec|yarn exec|bun x --no-install|aube exec)\s+(.+)$/.exec(
        command,
      );
    if (match) return { kind: "local", args: match[1] };
    match = /^(?:vpx|npx|pnpm dlx|yarn dlx|bunx|bun x|aube dlx|aubx)\s+(.+)$/.exec(command);
    if (match) return { kind: "once", args: match[1] };
    match = /^vp (run\s+.+|dev|build|test|check|lint|fmt)(\s.*)?$/.exec(command);
    if (match) return { kind: "vite", args: command };
    match = /^(?:npm|pnpm|yarn|bun|aube) run\s+(.+)$/.exec(command);
    if (match) return { kind: "script", args: match[1] };
    match = /^aubr\s+(.+)$/.exec(command);
    if (match) return { kind: "script", args: match[1] };
    return null;
  }

  function translate(source, manager) {
    let changed = false;
    const output = [];
    for (const line of source.split("\n")) {
      const match = /^(\s*)(.*?)(\s+#.*)?$/.exec(line);
      const [, indent, command, comment = ""] = match;
      if (!managerLine.test(command)) {
        output.push(line);
        continue;
      }
      // Do not rewrite shell programs or package-manager-specific flags.
      if (/[;&|<>`\\]|\$\(/.test(command)) return null;
      const value = operation(command);
      if (!value) return null;
      changed = true;
      let converted;
      switch (value.kind) {
        case "install":
          converted = `${manager} install`;
          break;
        case "add":
          converted = `${add[manager]} ${value.args}`;
          break;
        case "local":
          converted = `${local[manager]} ${value.args}`;
          break;
        case "once":
          converted = `${once[manager]} ${value.args}`;
          break;
        case "vite":
          converted = manager === "vp" ? value.args : `${local[manager]} ${value.args}`;
          break;
        case "script":
          converted = `${manager} run ${value.args}`;
          break;
      }
      output.push(`${indent}${converted}${comment}`);
    }
    return changed ? output.join("\n") : null;
  }

  function choices(source) {
    if (!source.split("\n").some((line) => managerLine.test(line.trim()))) return null;
    return managers.map((manager) => ({
      manager,
      command: manager === "jsr" ? null : translate(source, manager),
      gap: manager === "jsr" ? "registry" : "workspace",
    }));
  }

  globalThis.__vizeDocsCommands = { managers, choices };
})();
