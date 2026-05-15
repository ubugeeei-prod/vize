import { defineNitroPlugin } from "nitropack/runtime";
import { sanitizeNuxtDevStylesheetLinks } from "../../dev-html";
import { devAssetBase } from "#vizejs/nuxt/dev-stylesheet-links-config";

export default defineNitroPlugin((nitroApp) => {
  nitroApp.hooks.hook("render:response", (response) => {
    if (typeof response?.body !== "string" || !response.body.includes("<link")) {
      return;
    }

    response.body = sanitizeNuxtDevStylesheetLinks(response.body, devAssetBase);
  });
});
