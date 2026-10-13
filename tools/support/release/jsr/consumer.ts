/**
 * Exercises the `@vizejs/vize` JSR entry points from a consumer's runtime.
 *
 * Usage: `consumer.ts <root> [--vite]`, where `<root>` is how the consumer
 * names the package:
 *
 * - `@vizejs/vize` after `npx jsr add` or `bunx jsr add`;
 * - `jsr:@vizejs/vize@<version>` under Deno;
 * - a `file:` directory URL ending in `/` for a staged, unpublished package.
 *
 * The file imports nothing else from the Vize checkout. Copy it into the
 * consumer project so that bare specifiers resolve from that project.
 */
const [root = "@vizejs/vize", ...flags] = process.argv.slice(2);
const staged = root.endsWith("/");

function entry(subpath: "" | "config" | "native" | "vite"): string {
  if (staged) return `${root}${subpath === "" ? "mod" : subpath}.ts`;
  return subpath === "" ? root : `${root}/${subpath}`;
}

function check(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message);
}

const { defineConfig } = await import(entry(""));
const config = defineConfig({ compiler: { vapor: false } });
check(config.compiler.vapor === false, "defineConfig returns the authored configuration");

const { loadConfig } = await import(entry("config"));
check(typeof loadConfig === "function", "the config entry point exports loadConfig");

const { compileSfc } = await import(entry("native"));
const compiled = compileSfc(
  '<script setup lang="ts">const title: string = "JSR consumer"</script>' +
    "<template><h1>{{ title }}</h1></template>",
  { filename: "App.vue" },
);
check(compiled.errors.length === 0, `the native compiler reports no errors: ${compiled.errors}`);
check(compiled.code.includes("JSR consumer"), "the native compiler emits the component");

if (flags.includes("--vite")) {
  const { default: vize } = await import(entry("vite"));
  const plugins = [vize()].flat();
  check(plugins.length > 0, "the Vite entry point's default export creates the plugin");
}

console.log(`Verified ${root} (${flags.join(" ") || "no flags"})`);
