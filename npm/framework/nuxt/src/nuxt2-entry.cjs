"use strict";

const moduleMeta = { name: "@vizejs/nuxt", configKey: "vize" };
const moduleDefaults = {
  checker: false,
  lint: true,
  musea: false,
  nuxtMusea: { route: { path: "/" } },
};

let modulePromise;

function loadNuxtModule() {
  modulePromise ||= import("./index.mjs").then((loaded) => loaded.default);
  return modulePromise;
}

function isPlainRecord(value) {
  return value != null && typeof value === "object" && !Array.isArray(value);
}

function mergePlainRecords(...values) {
  const result = {};

  for (const value of values) {
    if (!value) continue;
    for (const [key, nextValue] of Object.entries(value)) {
      const currentValue = result[key];
      result[key] =
        isPlainRecord(currentValue) && isPlainRecord(nextValue)
          ? mergePlainRecords(currentValue, nextValue)
          : nextValue;
    }
  }

  return result;
}

function resolveModuleOptions(inlineOptions = {}, nuxt) {
  return mergePlainRecords(moduleDefaults, nuxt?.options?.vize, inlineOptions);
}

const vizeNuxtModule = Object.assign(
  async function vizeNuxtModule(inlineOptions = {}, nuxtArg) {
    const loaded = await loadNuxtModule();
    return loaded.call(this, inlineOptions, nuxtArg);
  },
  {
    getMeta: () => moduleMeta,
    getOptions: resolveModuleOptions,
    meta: moduleMeta,
    defaults: moduleDefaults,
  },
);

module.exports = vizeNuxtModule;
module.exports.default = vizeNuxtModule;
