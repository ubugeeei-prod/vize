/**
 * JS-based fallback for variant auto-generation.
 *
 * Used when the native Rust binding is not available. Provides simple
 * regex-based prop extraction, minimal art file generation, and a pure
 * JS variant generator.
 */

import path from "node:path";

import type { AutogenOptions, AutogenOutput, GeneratedVariant, PropDefinition } from "./index.js";

/**
 * Simple prop extraction fallback (when native binding not available).
 */
export function extractPropsSimple(source: string): PropDefinition[] {
  const props: PropDefinition[] = [];

  // Match defineProps<{ ... }>() or defineProps({ ... })
  const propsMatch = source.match(/defineProps\s*<\s*\{([^}]*)\}\s*>/s);

  if (propsMatch) {
    const propsBlock = propsMatch[1];
    const propLines = propsBlock.split("\n");

    for (const line of propLines) {
      const propMatch = line.trim().match(/^(\w+)(\?)?:\s*(.+?)\s*;?\s*$/);
      if (propMatch) {
        props.push({
          name: propMatch[1],
          propType: propMatch[3].replace(/,\s*$/, ""),
          required: !propMatch[2],
        });
      }
    }
  }

  return props;
}

/**
 * Minimal art file for components with no props.
 */
export function generateMinimalArt(componentName: string, componentPath: string): string {
  return `<script setup lang="ts">
defineArt(${JSON.stringify(componentPath)}, {
  title: ${JSON.stringify(componentName)},
});
</script>

<art>
  <variant name="Default" default>
    <${componentName} />
  </variant>
</art>
`;
}

/**
 * JS-based variant generation fallback.
 */
export function generateArtFileJs(
  componentPath: string,
  props: PropDefinition[],
  options: AutogenOptions,
): AutogenOutput {
  const componentName = path.basename(componentPath, ".vue");
  const relPath = `./${path.basename(componentPath)}`;
  const maxVariants = options.maxVariants ?? 20;
  const variants: GeneratedVariant[] = [];

  // Default variant
  if (options.includeDefault !== false) {
    const defaultProps: Record<string, unknown> = {};
    for (const prop of props) {
      if (prop.defaultValue !== undefined) {
        defaultProps[prop.name] = prop.defaultValue;
      }
    }
    variants.push({
      name: "Default",
      isDefault: true,
      props: withRequiredPlaceholders(defaultProps, props),
      description: `${componentName} with default props`,
    });
  }

  // Enum variants
  if (options.includeEnumVariants !== false) {
    for (const prop of props) {
      const unionValues = parseUnionType(prop.propType);
      for (const val of unionValues) {
        if (variants.length >= maxVariants) break;
        const name =
          typeof val === "string" ? toPascalCase(val) : `${toPascalCase(prop.name)}_${String(val)}`;
        variants.push({
          name,
          isDefault: false,
          props: withRequiredPlaceholders({ [prop.name]: val }, props),
          description: `${prop.name} = ${JSON.stringify(val)}`,
        });
      }
    }
  }

  // Boolean toggle variants
  if (options.includeBooleanToggles !== false) {
    for (const prop of props) {
      if (variants.length >= maxVariants) break;
      if (prop.propType.toLowerCase() === "boolean") {
        const nonDefault = prop.defaultValue === true ? false : true;
        variants.push({
          name: nonDefault ? toPascalCase(prop.name) : `No${toPascalCase(prop.name)}`,
          isDefault: false,
          props: withRequiredPlaceholders({ [prop.name]: nonDefault }, props),
          description: `${prop.name} = ${nonDefault}`,
        });
      }
    }
  }

  return {
    variants,
    artFileContent: generateArtFileContent(componentName, relPath, variants, props),
    componentName,
  };
}

export function finalizeArtOutput(
  output: AutogenOutput,
  componentPath: string,
  props: PropDefinition[],
): AutogenOutput {
  const variants = output.variants.map((variant) => ({
    ...variant,
    props: withRequiredPlaceholders(variant.props, props),
  }));
  return {
    ...output,
    variants,
    artFileContent: generateArtFileContent(output.componentName, componentPath, variants, props),
  };
}

function generateArtFileContent(
  componentName: string,
  componentPath: string,
  variants: GeneratedVariant[],
  props: PropDefinition[],
): string {
  const fixtures = new Map<string, string>();
  let variantsContent = "";

  for (const variant of variants) {
    const attrs = variant.isDefault ? `name="${variant.name}" default` : `name="${variant.name}"`;
    variantsContent += `  <variant ${attrs}>\n`;

    const propsStr = Object.entries(variant.props)
      .map(([k, v]) => renderPropAttribute(k, v, props, fixtures))
      .join(" ");

    variantsContent += `    <${componentName}${propsStr ? " " + propsStr : ""} />\n`;
    variantsContent += `  </variant>\n\n`;
  }

  const fixtureLines = [...fixtures.entries()].map(
    ([propName, fixture]) =>
      `const ${fixture} = {} as never; // TODO: replace generated fixture for required prop ${JSON.stringify(propName)}`,
  );
  const fixtureBlock = fixtureLines.length > 0 ? `${fixtureLines.join("\n")}\n\n` : "";

  return `<script setup lang="ts">
${fixtureBlock}defineArt(${JSON.stringify(componentPath)}, {
  title: ${JSON.stringify(componentName)},
});
</script>

<art>
${variantsContent}</art>
`;
}

function withRequiredPlaceholders(
  values: Record<string, unknown>,
  props: PropDefinition[],
): Record<string, unknown> {
  const next = { ...values };
  for (const prop of props) {
    if (prop.required && prop.defaultValue === undefined && !(prop.name in next)) {
      next[prop.name] = null;
    }
  }
  return next;
}

function renderPropAttribute(
  name: string,
  value: unknown,
  props: PropDefinition[],
  fixtures: Map<string, string>,
): string {
  const attrName = toKebabCase(name);
  const prop = props.find((candidate) => candidate.name === name);
  if (prop?.required && prop.defaultValue === undefined && value == null) {
    const fixture = fixtureIdentifier(name);
    fixtures.set(name, fixture);
    return `:${attrName}="${fixture}"`;
  }
  if (typeof value === "string") return `${attrName}="${value}"`;
  if (typeof value === "boolean" && value) return attrName;
  if (typeof value === "boolean" && !value) return `:${attrName}="false"`;
  return `:${attrName}="${JSON.stringify(value)}"`;
}

function fixtureIdentifier(propName: string): string {
  const base = propName.replace(/[^\w$]/g, "_");
  return /^[A-Za-z_$]/.test(base) ? `${base}Fixture` : `prop${toPascalCase(base)}Fixture`;
}

export function parseUnionType(typeStr: string): unknown[] {
  const trimmed = typeStr.trim();
  if (!trimmed.includes("|")) return [];

  if (trimmed.includes("'") || trimmed.includes('"')) {
    return trimmed
      .split("|")
      .map((s) => s.trim().replace(/^['"]|['"]$/g, ""))
      .filter((s) => s.length > 0);
  }

  const parts = trimmed.split("|").map((s) => s.trim());
  if (parts.every((p) => !isNaN(Number(p)))) {
    return parts.map(Number);
  }

  return [];
}

export function toPascalCase(str: string): string {
  return str
    .split(/[\s\-_]+/)
    .filter(Boolean)
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join("");
}

function toKebabCase(str: string): string {
  return str.replace(/[A-Z]/g, (char) => `-${char.toLowerCase()}`);
}
