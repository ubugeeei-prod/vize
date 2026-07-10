/**
 * Preview module and HTML generation for Musea component previews.
 *
 * Generates the JavaScript modules that mount Vue components in preview iframes,
 * as well as the HTML wrapper pages for those previews.
 */

import type { ArtFileInfo } from "../types/index.js";
import type { MuseaVueVersion } from "../types/plugin.js";
import { MUSEA_ADDONS_INIT_CODE } from "./addons.js";

export { generatePreviewHtml } from "./html.js";

export function generatePreviewModule(
  art: ArtFileInfo,
  variantComponentName: string,
  variantName: string,
  cssImports: string[] = [],
  previewSetup: string | null = null,
  vueVersion: MuseaVueVersion = 3,
): string {
  const artModuleId = `virtual:musea-art:${art.path}`;
  const artModuleIdLiteral = JSON.stringify(artModuleId);
  const variantNameLiteral = JSON.stringify(variantName);
  const variantComponentNameLiteral = JSON.stringify(variantComponentName);
  const cssImportStatements = cssImports
    .map((cssPath) => `import ${JSON.stringify(cssPath)};`)
    .join("\n");
  const setupImport = previewSetup
    ? `import __museaPreviewSetup from ${JSON.stringify(previewSetup)};`
    : "";
  const setupCall = previewSetup ? "await __museaPreviewSetup(app);" : "";
  const actionEvents = JSON.stringify(art.metadata.actionEvents ?? []);
  const artStyleId = `musea-art-styles-${art.path.replace(/[^\w-]+/g, "_")}`;
  const artStyleIdLiteral = JSON.stringify(artStyleId);
  const legacyVue = isLegacyVueVersion(vueVersion);
  const vueImport = legacyVue
    ? "import Vue, { reactive } from 'vue';"
    : "import { createApp, reactive, h } from 'vue';";
  const destroyCurrentApp = legacyVue ? "currentApp.$destroy();" : "currentApp.unmount();";
  const mountInitialApp = legacyVue
    ? "const app = new Vue({ render: (h) => h(VariantComponent) });"
    : "const app = createApp(VariantComponent);";
  const mountCurrentApp = legacyVue
    ? "mountVue2App(app);"
    : "app.mount(container);\n    currentApp = app;";
  const mountHelper = legacyVue ? vue2MountHelperSnippet() : "";
  const remountApp = legacyVue ? vue2RemountAppSnippet() : vue3RemountAppSnippet();

  return `
${cssImportStatements}
${setupImport}
${vueImport}
import * as artModule from ${artModuleIdLiteral};

const container = document.getElementById('app');

${MUSEA_ADDONS_INIT_CODE}

let currentApp = null;
const propsOverride = reactive({});
const slotsOverride = reactive({ default: '' });

function ensureArtStyles(styles) {
  const styleId = ${artStyleIdLiteral};
  const existing = document.getElementById(styleId);

  if (!Array.isArray(styles) || styles.length === 0) {
    existing?.remove();
    return;
  }

  const tag = existing ?? document.createElement('style');
  tag.id = styleId;
  tag.textContent = styles.join('\\n\\n');

  if (!existing) {
    document.head.appendChild(tag);
  }
}

function renderError(title, error) {
  container.textContent = '';
  const root = document.createElement('div');
  root.className = 'musea-error';

  const titleEl = document.createElement('div');
  titleEl.className = 'musea-error-title';
  titleEl.textContent = title;
  root.appendChild(titleEl);

  const messageEl = document.createElement('div');
  messageEl.textContent = error instanceof Error ? error.message : String(error);
  root.appendChild(messageEl);

  const stack = error instanceof Error ? error.stack : '';
  if (stack) {
    const stackEl = document.createElement('pre');
    stackEl.textContent = stack;
    root.appendChild(stackEl);
  }

  container.appendChild(root);
}

${mountHelper}

window.__museaSetProps = (props) => {
  // Clear old keys
  for (const key of Object.keys(propsOverride)) {
    delete propsOverride[key];
  }
  Object.assign(propsOverride, props);
};

window.__museaSetSlots = (slots) => {
  for (const key of Object.keys(slotsOverride)) {
    delete slotsOverride[key];
  }
  Object.assign(slotsOverride, slots);
};

async function mount() {
  try {
    // Get the specific variant component
    const VariantComponent = artModule[${variantComponentNameLiteral}];
    const RawComponent = artModule.__component__;

    if (!VariantComponent) {
      throw new Error('Variant component ' + ${variantComponentNameLiteral} + ' not found in art module');
    }

    // Create and mount the app
    ${mountInitialApp}
    ensureArtStyles(artModule.__styles__);
    ${setupCall}
    container.innerHTML = '';
    container.className = 'musea-variant';
    ${mountCurrentApp}

    console.log('[musea-preview] Mounted variant:', ${variantNameLiteral});
    __museaInitAddons(container, ${variantNameLiteral}, ${actionEvents});

    // Override set-props to remount with raw component + props
    const TargetComponent = RawComponent || VariantComponent;
    window.__museaSetProps = (props) => {
      for (const key of Object.keys(propsOverride)) {
        delete propsOverride[key];
      }
      Object.assign(propsOverride, props);
      remountWithProps(TargetComponent);
    };
    window.__museaSetSlots = (slots) => {
      for (const key of Object.keys(slotsOverride)) {
        delete slotsOverride[key];
      }
      Object.assign(slotsOverride, slots);
      remountWithProps(TargetComponent);
    };
  } catch (error) {
    console.error('[musea-preview] Failed to mount:', error);
    renderError('Failed to render component', error);
  }
}

async function remountWithProps(Component) {
  if (currentApp) {
    ${destroyCurrentApp}
  }
  ${remountApp}
  ensureArtStyles(artModule.__styles__);
  ${setupCall}
  container.innerHTML = '';
  ${mountCurrentApp}
}

mount();
`;
}

function isLegacyVueVersion(version: MuseaVueVersion | undefined): boolean {
  return (
    version === 0.11 || version === 1 || version === 2 || version === "2.7" || version === "legacy"
  );
}

function vue2MountHelperSnippet(): string {
  return `function mountVue2App(app) {
  container.innerHTML = '';
  const mountPoint = document.createElement('div');
  container.appendChild(mountPoint);
  app.$mount(mountPoint);
  currentApp = app;
}`;
}

function vue2RemountAppSnippet(): string {
  return `const app = new Vue({
    render(h) {
      const slotFns = {};
      const children = [];
      for (const [name, content] of Object.entries(slotsOverride)) {
        if (!content) continue;
        const slot = () => [h('span', String(content))];
        slotFns[name] = slot;
        if (name === 'default') children.push(...slot());
      }
      return h(Component, { props: { ...propsOverride }, scopedSlots: slotFns }, children);
    }
  });`;
}

function vue3RemountAppSnippet(): string {
  return `const app = createApp({
    setup() {
      return () => {
        const slotFns = {};
        for (const [name, content] of Object.entries(slotsOverride)) {
          if (content) slotFns[name] = () => h('span', String(content));
        }
        return h(Component, { ...propsOverride }, slotFns);
      };
    }
  });`;
}

export function generatePreviewModuleWithProps(
  art: ArtFileInfo,
  variantComponentName: string,
  variantName: string,
  propsOverride: Record<string, unknown>,
  cssImports: string[] = [],
  previewSetup: string | null = null,
  vueVersion: MuseaVueVersion = 3,
): string {
  const artModuleId = `virtual:musea-art:${art.path}`;
  const artModuleIdLiteral = JSON.stringify(artModuleId);
  const variantNameLiteral = JSON.stringify(variantName);
  const variantComponentNameLiteral = JSON.stringify(variantComponentName);
  const propsJson = JSON.stringify(propsOverride);
  const cssImportStatements = cssImports
    .map((cssPath) => `import ${JSON.stringify(cssPath)};`)
    .join("\n");
  const setupImport = previewSetup
    ? `import __museaPreviewSetup from ${JSON.stringify(previewSetup)};`
    : "";
  const setupCall = previewSetup ? "await __museaPreviewSetup(app);" : "";
  const actionEvents = JSON.stringify(art.metadata.actionEvents ?? []);
  const artStyleId = `musea-art-styles-${art.path.replace(/[^\w-]+/g, "_")}`;
  const artStyleIdLiteral = JSON.stringify(artStyleId);
  const legacyVue = isLegacyVueVersion(vueVersion);
  const vueImport = legacyVue ? "import Vue from 'vue';" : "import { createApp, h } from 'vue';";
  const wrappedComponent = legacyVue
    ? `const WrappedComponent = {
      render(h) {
        return h(VariantComponent, { props: propsOverride });
      }
    };`
    : `const WrappedComponent = {
      render() {
        return h(VariantComponent, propsOverride);
      }
    };`;
  const appCreate = legacyVue
    ? "const app = new Vue(WrappedComponent);"
    : "const app = createApp(WrappedComponent);";
  const appMount = legacyVue
    ? "const mountPoint = document.createElement('div');\n    container.appendChild(mountPoint);\n    app.$mount(mountPoint);"
    : "app.mount(container);";

  return `
${cssImportStatements}
${setupImport}
${vueImport}
import * as artModule from ${artModuleIdLiteral};

const container = document.getElementById('app');
const propsOverride = ${propsJson};

${MUSEA_ADDONS_INIT_CODE}

function ensureArtStyles(styles) {
  const styleId = ${artStyleIdLiteral};
  const existing = document.getElementById(styleId);

  if (!Array.isArray(styles) || styles.length === 0) {
    existing?.remove();
    return;
  }

  const tag = existing ?? document.createElement('style');
  tag.id = styleId;
  tag.textContent = styles.join('\\n\\n');

  if (!existing) {
    document.head.appendChild(tag);
  }
}

function renderError(title, error) {
  container.textContent = '';
  const root = document.createElement('div');
  root.className = 'musea-error';

  const titleEl = document.createElement('div');
  titleEl.className = 'musea-error-title';
  titleEl.textContent = title;
  root.appendChild(titleEl);

  const messageEl = document.createElement('div');
  messageEl.textContent = error instanceof Error ? error.message : String(error);
  root.appendChild(messageEl);

  container.appendChild(root);
}

async function mount() {
  try {
    const VariantComponent = artModule[${variantComponentNameLiteral}];
    if (!VariantComponent) {
      throw new Error('Variant component ' + ${variantComponentNameLiteral} + ' not found');
    }

    ${wrappedComponent}

    ${appCreate}
    ensureArtStyles(artModule.__styles__);
    ${setupCall}
    container.innerHTML = '';
    container.className = 'musea-variant';
    ${appMount}
    console.log('[musea-preview] Mounted variant with props override:', ${variantNameLiteral});
    __museaInitAddons(container, ${variantNameLiteral}, ${actionEvents});
  } catch (error) {
    console.error('[musea-preview] Failed to mount:', error);
    renderError('Failed to render', error);
  }
}

mount();
`;
}
