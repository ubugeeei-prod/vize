/**
 * Legacy Vue 2 variant emission.
 *
 * Vue 2 has no `openBlock`/`createElementBlock` runtime, so the compiled Vue 3
 * render functions the SFC pipeline produces cannot load there. Vue 2 galleries
 * keep the runtime-compiled `template:` string they always had — the TypeScript
 * fix in #3857 applies to Vue 3, which is what `.art.vue` with
 * `<script setup lang="ts">` targets.
 */

import { escapeHtml } from "./utils.js";

export function escapeTemplateLiteral(str: string): string {
  return str.replace(/\\/g, "\\\\").replace(/`/g, "\\`").replace(/\$/g, "\\$");
}

export type LegacyVariantContext = {
  variantComponentName: string;
  variantName: string;
  template: string;
  componentTagName?: string;
  componentBindingName: string;
  scriptSetup: { setupBody: string[]; returnNames: string[]; imports: string[] } | null;
  hasSetup: boolean;
  isolatedSetup: boolean;
  setupReturn: string;
  importDeclaresName: (statement: string, name: string) => boolean;
};

export function emitLegacyVariant(context: LegacyVariantContext): string {
  const {
    variantComponentName,
    variantName,
    template,
    componentTagName,
    componentBindingName,
    scriptSetup,
    hasSetup,
    isolatedSetup,
    setupReturn,
    importDeclaresName,
  } = context;

  const escapedTemplate = escapeTemplateLiteral(template);
  const escapedVariantName = escapeTemplateLiteral(escapeHtml(variantName));
  const fullTemplate = `<div data-variant="${escapedVariantName}">${escapedTemplate}</div>`;

  // Runtime-compiled templates use resolveComponent(), which checks the
  // `components` option rather than setup return values.
  const componentNames = new Map<string, string>();
  if (componentTagName) componentNames.set(componentTagName, componentBindingName);
  if (scriptSetup) {
    for (const name of scriptSetup.returnNames)
      if (/^[A-Z]/.test(name) && scriptSetup.imports.some((imp) => importDeclaresName(imp, name)))
        componentNames.set(name, name);
  }
  const components =
    componentNames.size > 0
      ? `  components: { ${[...componentNames]
          .map(([name, value]) => `${JSON.stringify(name)}: ${value}`)
          .join(", ")} },\n`
      : "";

  if (scriptSetup && hasSetup && isolatedSetup) {
    return `
export const ${variantComponentName} = __museaDefineComponent({
  name: '${variantComponentName}',
${components}  setup() {
${scriptSetup.setupBody.join("\n")}
    return ${setupReturn};
  },
  template: \`${fullTemplate}\`,
});
`;
  }
  if (scriptSetup && hasSetup) {
    return `
export const ${variantComponentName} = __museaDefineComponent({
  name: '${variantComponentName}',
${components}  setup() {
    return __museaSharedSetup;
  },
  template: \`${fullTemplate}\`,
});
`;
  }
  if (componentTagName) {
    return `
export const ${variantComponentName} = {
  name: '${variantComponentName}',
${components}  template: \`${fullTemplate}\`,
};
`;
  }
  return `
export const ${variantComponentName} = {
  name: '${variantComponentName}',
  template: \`${fullTemplate}\`,
};
`;
}
