export interface NuxtUiSnapshotOptions {
  cwd: string;
}

export function normalizeNuxtUiSnapshotHtml(html: string, options: NuxtUiSnapshotOptions): string {
  const normalizedWorktreePath = encodeURIComponent(options.cwd);
  return html
    .replaceAll(normalizedWorktreePath, "__NUXT_UI_WORKTREE__")
    .replaceAll(options.cwd, "__NUXT_UI_WORKTREE__")
    .replace(
      /<script type="application\/json" data-nuxt-logs="nuxt-app">[\s\S]*?<\/script>/,
      '<script type="application/json" data-nuxt-logs="nuxt-app">__NUXT_UI_LOGS__</script>',
    )
    .replace(
      /<style>@layer base {\n(?::where\(\.i-lucide\\:[\s\S]*?\n)+}<\/style>/g,
      "<style>@layer base {\n__NUXT_UI_ICON_CSS__\n}</style>",
    )
    .replace(
      /<link rel="modulepreload" as="script" crossorigin href="\/_nuxt[^"]*entry\.async\.js">/g,
      "",
    )
    .replace(/<script type="module" src="\/_nuxt\/@vite\/client" crossorigin><\/script>/g, "")
    .replace(/<script type="module" src="\/_nuxt[^"]*entry\.async\.js" crossorigin><\/script>/g, "")
    .replace(
      /<script>\s*if \(!window\.__NUXT_DEVTOOLS_TIME_METRIC__\) \{[\s\S]*?window\.__NUXT_DEVTOOLS_TIME_METRIC__\.appInit = Date\.now\(\)\s*<\/script>/g,
      "",
    )
    .replace(/(>)(?:[ \t]*\n){2,}(?=<meta name="description")/g, "$1")
    .replace(/\b\d{13}\b/g, "0");
}
