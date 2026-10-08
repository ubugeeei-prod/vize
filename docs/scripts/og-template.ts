import { createHash } from "node:crypto";
import { readFile, readdir, writeFile } from "node:fs/promises";
import path from "node:path";
import { build } from "vite-plus";
import vize from "@vizejs/vite-plugin";

interface OgTemplateBuild {
  template: string;
  assetFingerprint: string;
}

/** Use the real Vite CSS pipeline; Ox Content 2.81's bare Rolldown Vue loader cannot. */
export async function buildOgTemplate(docsRoot: string): Promise<OgTemplateBuild> {
  const output = path.join(docsRoot, ".cache", "og-template");
  const componentDir = path.join(output, "ssr");
  const styleDir = path.join(output, "styles");
  await build({
    configFile: false,
    root: docsRoot,
    publicDir: false,
    plugins: [vize({ ssr: true, isProduction: true })],
    build: {
      ssr: path.join(docsRoot, "theme", "og.vue"),
      ssrEmitAssets: true,
      outDir: componentDir,
      emptyOutDir: true,
      minify: false,
      rolldownOptions: {
        external: ["vue", "vue/server-renderer"],
        output: { entryFileNames: "component.js", assetFileNames: "[name][extname]" },
      },
    },
  });
  // Native SSR intentionally omits client style imports. Emit the same Vue
  // authority's matching scoped stylesheet through its native client CSS build.
  await build({
    configFile: false,
    root: docsRoot,
    publicDir: false,
    plugins: [vize({ isProduction: true })],
    build: {
      lib: {
        entry: path.join(docsRoot, "theme", "og.vue"),
        formats: ["es"],
        fileName: "style-source",
      },
      outDir: styleDir,
      emptyOutDir: true,
      minify: false,
      cssCodeSplit: false,
      rolldownOptions: { external: ["vue"], output: { assetFileNames: "[name][extname]" } },
    },
  });
  const files = await readdir(styleDir);
  const css = (
    await Promise.all(
      files
        .filter((file) => file.endsWith(".css"))
        .map((file) => readFile(path.join(styleDir, file), "utf8")),
    )
  ).join("\n");
  if (!css.includes("--og-paper"))
    throw new Error("Native OG template build emitted no stylesheet");
  const component = await readFile(path.join(componentDir, "component.js"), "utf8");
  const logo = await readFile(path.join(docsRoot, "public", "logo.svg"));
  const assetFingerprint = createHash("sha256")
    .update(component)
    .update(css)
    .update(logo)
    .digest("hex");
  const template = path.join(output, "template.ts");
  // The native SSR bundle is generated JavaScript. Describe its public Vue
  // component contract so the generated TypeScript adapter remains checkable.
  await writeFile(
    path.join(componentDir, "component.d.ts"),
    [
      'import type { Component } from "vue";',
      "declare const component: Component;",
      "export default component;",
      "",
    ].join("\n"),
  );
  await writeFile(
    template,
    [
      'import { createSSRApp } from "vue";',
      'import { renderToString } from "vue/server-renderer";',
      'import type { OgImageTemplateProps } from "@ox-content/vite-plugin";',
      'import Component from "./ssr/component.js";',
      `const css = ${JSON.stringify(css)};`,
      "export default async function render(props: OgImageTemplateProps): Promise<string> {",
      "  const html = await renderToString(createSSRApp(Component, props));",
      '  return "<style>" + css + "</style>" + html;',
      "}",
      "",
    ].join("\n"),
  );
  return { template, assetFingerprint };
}
