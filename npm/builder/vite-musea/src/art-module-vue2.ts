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
import { assertStaticExpression } from "./native-loader.js";
import { serializeScriptValue } from "./security.js";

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
  play?: string;
  props?: Record<string, unknown>;
  attrs?: Record<string, unknown>;
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
    play,
    props,
    attrs,
  } = context;
  const playExpression = play ? assertStaticExpression(play, "play") : undefined;
  const isolatedPlay = play
    ? `    const __museaVariantPlay = (${playExpression});\n    if (typeof window !== "undefined") window.__museaPlay = __museaVariantPlay;\n`
    : "";
  const sharedPlay = play
    ? `    const { ${scriptSetup?.returnNames.join(", ") ?? ""} } = __museaSharedSetup;\n    const __museaVariantPlay = (${playExpression});\n    if (typeof window !== "undefined") window.__museaPlay = __museaVariantPlay;\n`
    : "";
  const hasBindings = Boolean(props || attrs);
  const variantBindings = hasBindings
    ? `    const __museaVariantBindings = ${serializeScriptValue({ ...attrs, ...props })};\n`
    : "";
  const isolatedReturn = hasBindings
    ? `Object.assign(${setupReturn}, { __museaVariantBindings })`
    : setupReturn;
  const sharedReturn = hasBindings
    ? `{ ...__museaSharedSetup, __museaVariantBindings }`
    : "__museaSharedSetup";

  const boundTemplate = injectVariantBindings(template, componentTagName, hasBindings);
  const escapedTemplate = escapeTemplateLiteral(boundTemplate);
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
${variantBindings}${isolatedPlay}    return ${isolatedReturn};
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
${variantBindings}${sharedPlay}    return ${sharedReturn};
  },
  template: \`${fullTemplate}\`,
});
`;
  }
  if (play || hasBindings) {
    return `
export const ${variantComponentName} = {
  name: '${variantComponentName}',
${components}  setup() {
${variantBindings}${isolatedPlay}    return ${hasBindings ? "{ __museaVariantBindings }" : "{}"};
  },
  template: \`${fullTemplate}\`,
};
`;
  }
  // Without a setup block `components` is empty unless a component tag was
  // resolved, so one template covers both cases.
  return `
export const ${variantComponentName} = {
  name: '${variantComponentName}',
${components}  template: \`${fullTemplate}\`,
};
`;
}

function injectVariantBindings(
  template: string,
  componentTagName: string | undefined,
  hasBindings: boolean,
): string {
  if (!componentTagName || !hasBindings) return template;
  const escaped = componentTagName.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  return template.replace(
    new RegExp(`<${escaped}(?=[\\s/>])`),
    `<${componentTagName} v-bind="__museaVariantBindings"`,
  );
}
