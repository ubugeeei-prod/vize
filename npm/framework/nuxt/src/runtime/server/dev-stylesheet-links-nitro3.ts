import { sanitizeNuxtDevStylesheetLinks } from "../../dev-html";
import { devAssetBase } from "#vizejs/nuxt/dev-stylesheet-links-config";

interface NuxtHtmlContext {
  head: string[];
}
interface NitroRenderHooks {
  hooks: { hook(name: "render:html", callback: (html: NuxtHtmlContext) => void): void };
}

// The Nuxt renderer owns these head fragments on Nitro 3, including streaming shells.
export default function registerDevStylesheetLinks(nitroApp: NitroRenderHooks): void {
  nitroApp.hooks.hook("render:html", (html) => {
    html.head = [sanitizeNuxtDevStylesheetLinks(html.head.join(""), devAssetBase)];
  });
}
