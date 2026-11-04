interface RenderContext {
  head: string[];
}
interface NitroApp {
  hooks: { hook(name: "render:html", callback: (html: RenderContext) => void): void };
}

export default function observeStylesheets(nitroApp: NitroApp): void {
  nitroApp.hooks.hook("render:html", (html) => {
    html.head.push(
      '<link rel="stylesheet" href="/_nuxt/ghost-vize-entry.js">',
      '<link rel="stylesheet" href="/_nuxt/assets/stylesheet-control.css?direct">',
      '<link rel="stylesheet" href="/_nuxt/assets/stylesheet-control.css?direct">',
    );
  });
}
