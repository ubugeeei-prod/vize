import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { defineConfig } from "vite-plus";
import { oxContent, defineTheme, defaultTheme } from "@ox-content/vite-plugin";
import { resolvePuppeteerExecutablePath } from "./browser-path.js";
import { buildDocsBackgroundScript, createDocsBackgroundHtml } from "./theme/background";

const puppeteerExecutablePath = resolvePuppeteerExecutablePath();
if (puppeteerExecutablePath) {
  process.env.PUPPETEER_EXECUTABLE_PATH = puppeteerExecutablePath;
}

const themeDir = resolve(import.meta.dirname, "theme");
const themeCss = readFileSync(resolve(themeDir, "style.css"), "utf-8");
const themeJs = buildDocsBackgroundScript(themeDir);

export default defineConfig({
  plugins: [
    oxContent({
      srcDir: "content",
      outDir: "dist",

      ogImage: true,
      ogImageOptions: {
        template: resolve(themeDir, "og.vue"),
        vuePlugin: "vizejs",
        width: 1200,
        height: 630,
        cache: true,
      },

      ssg: {
        siteName: "Vize",
        siteUrl: "https://vizejs.dev",
        generateOgImage: true,
        theme: defineTheme({
          extends: defaultTheme,

          colors: {
            primary: "#121212",
            primaryHover: "#333333",
            background: "#e6e2d6",
            backgroundAlt: "#dedad0",
            text: "#121212",
            textMuted: "#5a5750",
            border: "#ccc8bc",
            codeBackground: "#1a1a1a",
            codeText: "#e8e4dc",
          },

          darkColors: {
            primary: "#e8e4dc",
            primaryHover: "#ffffff",
            background: "#161616",
            backgroundAlt: "#1c1c1c",
            text: "#e8e4dc",
            textMuted: "#8a8780",
            border: "#1e1e1e",
            codeBackground: "#0f0f0f",
            codeText: "#e8e4dc",
          },

          fonts: {
            sans: '"Helvetica Neue", Helvetica, Arial, system-ui, sans-serif',
            mono: '"JetBrains Mono", ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, monospace',
          },

          header: {
            logo: "/logo.svg",
            logoDark: "/logo-light.svg",
            logoWidth: 68,
            logoHeight: 34,
          },

          footer: {
            message:
              'Released under the <a href="https://opensource.org/licenses/MIT">MIT License</a>.',
            copyright: `Copyright &copy; 2024-${new Date().getFullYear()} ubugeeei`,
          },

          socialLinks: {
            github: "https://github.com/ubugeeei-prod/vize",
          },

          embed: {
            head: [
              '<link rel="icon" href="/mv.svg" type="image/svg+xml">',
              '<link rel="preconnect" href="https://fonts.googleapis.com">',
              '<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>',
              '<link href="https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;500;600&display=swap" rel="stylesheet">',
              '<script src="https://cdn.jsdelivr.net/npm/three@0.160.0/build/three.min.js"></script>',
              "<script>if(!localStorage.getItem('theme')){localStorage.setItem('theme','light')}</script>",
              // Progressive enhancement: the markdown pipeline sanitizes raw <video>
              // tags, so author videos as plain `.mp4` links in markdown and upgrade
              // them to inline players client-side. The link stays as a no-JS fallback.
              // Upstream: https://github.com/ubugeeei-prod/ox-content/issues/340 (allow <video>).
              `<script>
document.addEventListener("DOMContentLoaded", function () {
  document.querySelectorAll('a[href$=".mp4"]').forEach(function (a) {
    var video = document.createElement("video");
    video.src = a.getAttribute("href");
    video.controls = true;
    video.muted = true;
    video.playsInline = true;
    video.loop = true;
    video.preload = "metadata";
    video.style.cssText = "width:100%;max-width:760px;display:block;margin:1.5rem auto;border-radius:8px";
    var block = a.closest("p");
    (block && block.textContent.trim() === a.textContent.trim() ? block : a).replaceWith(video);
  });
});
</script>`,
            ].join("\n"),
            headerAfter: createDocsBackgroundHtml(),
          },

          css: themeCss,
          js: themeJs,
        }),
      },

      highlight: false,
      mermaid: true,
      // Keep source tree clean; this site does not use Ox Content's API docs generator.
      docs: false,
    }),
  ],

  server: {
    port: 4200,
  },
  preview: {
    port: 4200,
  },
  build: {
    outDir: "dist",
  },
});
