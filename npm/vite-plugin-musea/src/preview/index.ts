/**
 * Preview module and HTML generation for Musea component previews.
 *
 * Generates the JavaScript modules that mount Vue components in preview iframes,
 * as well as the HTML wrapper pages for those previews.
 */

import type { ArtFileInfo } from "../types/index.js";
import { escapeTemplate } from "../utils.js";
import { MUSEA_ADDONS_INIT_CODE } from "./addons.js";

export { generatePreviewHtml } from "./html.js";

export function generatePreviewModule(
  art: ArtFileInfo,
  variantComponentName: string,
  variantName: string,
  cssImports: string[] = [],
  previewSetup: string | null = null,
): string {
  const artModuleId = `virtual:musea-art:${art.path}`;
  const escapedVariantName = escapeTemplate(variantName);
  const cssImportStatements = cssImports.map((cssPath) => `import '${cssPath}';`).join("\n");
  const setupImport = previewSetup ? `import __museaPreviewSetup from '${previewSetup}';` : "";
  const setupCall = previewSetup ? "await __museaPreviewSetup(app);" : "";
  const actionEvents = JSON.stringify(art.metadata.actionEvents ?? []);
  const artStyleId = `musea-art-styles-${art.path.replace(/[^\w-]+/g, "_")}`;

  return `
${cssImportStatements}
${setupImport}
import { createApp, reactive, h } from 'vue';
import * as artModule from '${artModuleId}';

const container = document.getElementById('app');

${MUSEA_ADDONS_INIT_CODE}

let currentApp = null;
const propsOverride = reactive({});
const slotsOverride = reactive({ default: '' });

function ensureArtStyles(styles) {
  const styleId = '${artStyleId}';
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
    const VariantComponent = artModule['${variantComponentName}'];
    const RawComponent = artModule.__component__;

    if (!VariantComponent) {
      throw new Error('Variant component "${variantComponentName}" not found in art module');
    }

    // Create and mount the app
    const app = createApp(VariantComponent);
    ensureArtStyles(artModule.__styles__);
    ${setupCall}
    container.innerHTML = '';
    container.className = 'musea-variant';
    app.mount(container);
    currentApp = app;

    console.log('[musea-preview] Mounted variant: ${escapedVariantName}');
    __museaInitAddons(container, '${escapedVariantName}', ${actionEvents});

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
    container.innerHTML = \`
      <div class="musea-error">
        <div class="musea-error-title">Failed to render component</div>
        <div>\${error.message}</div>
        <pre>\${error.stack || ''}</pre>
      </div>
    \`;
  }
}

async function remountWithProps(Component) {
  if (currentApp) {
    currentApp.unmount();
  }
  const app = createApp({
    setup() {
      return () => {
        const slotFns = {};
        for (const [name, content] of Object.entries(slotsOverride)) {
          if (content) slotFns[name] = () => h('span', { innerHTML: content });
        }
        return h(Component, { ...propsOverride }, slotFns);
      };
    }
  });
  ensureArtStyles(artModule.__styles__);
  ${setupCall}
  container.innerHTML = '';
  app.mount(container);
  currentApp = app;
}

mount();
`;
}

export function generatePreviewModuleWithProps(
  art: ArtFileInfo,
  variantComponentName: string,
  variantName: string,
  propsOverride: Record<string, unknown>,
  cssImports: string[] = [],
  previewSetup: string | null = null,
): string {
  const artModuleId = `virtual:musea-art:${art.path}`;
  const escapedVariantName = escapeTemplate(variantName);
  const propsJson = JSON.stringify(propsOverride);
  const cssImportStatements = cssImports.map((cssPath) => `import '${cssPath}';`).join("\n");
  const setupImport = previewSetup ? `import __museaPreviewSetup from '${previewSetup}';` : "";
  const setupCall = previewSetup ? "await __museaPreviewSetup(app);" : "";
  const actionEvents = JSON.stringify(art.metadata.actionEvents ?? []);
  const artStyleId = `musea-art-styles-${art.path.replace(/[^\w-]+/g, "_")}`;

  return `
${cssImportStatements}
${setupImport}
import { createApp, h } from 'vue';
import * as artModule from '${artModuleId}';

const container = document.getElementById('app');
const propsOverride = ${propsJson};

${MUSEA_ADDONS_INIT_CODE}

function ensureArtStyles(styles) {
  const styleId = '${artStyleId}';
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

async function mount() {
  try {
    const VariantComponent = artModule['${variantComponentName}'];
    if (!VariantComponent) {
      throw new Error('Variant component "${variantComponentName}" not found');
    }

    const WrappedComponent = {
      render() {
        return h(VariantComponent, propsOverride);
      }
    };

    const app = createApp(WrappedComponent);
    ensureArtStyles(artModule.__styles__);
    ${setupCall}
    container.innerHTML = '';
    container.className = 'musea-variant';
    app.mount(container);
    console.log('[musea-preview] Mounted variant: ${escapedVariantName} with props override');
    __museaInitAddons(container, '${escapedVariantName}', ${actionEvents});
  } catch (error) {
    console.error('[musea-preview] Failed to mount:', error);
    container.innerHTML = '<div class="musea-error"><div class="musea-error-title">Failed to render</div><div>' + error.message + '</div></div>';
  }
}

mount();
`;
}
