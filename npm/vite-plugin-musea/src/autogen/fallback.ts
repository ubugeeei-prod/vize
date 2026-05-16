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
  return `<art title="${componentName}" component="${componentPath}">
  <variant name="Default" default>
    <${componentName} />
  </variant>
</art>

<script setup lang="ts">
import ${componentName} from '${componentPath}'
</script>
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
      props: defaultProps,
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
          props: { [prop.name]: val },
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
          props: { [prop.name]: nonDefault },
          description: `${prop.name} = ${nonDefault}`,
        });
      }
    }
  }

  // Generate art file content
  let content = `<art title="${componentName}" component="${relPath}">\n`;
  for (const variant of variants) {
    const attrs = variant.isDefault ? `name="${variant.name}" default` : `name="${variant.name}"`;
    content += `  <variant ${attrs}>\n`;

    const propsStr = Object.entries(variant.props)
      .map(([k, v]) => {
        if (typeof v === "string") return `${k}="${v}"`;
        if (typeof v === "boolean" && v) return k;
        if (typeof v === "boolean" && !v) return `:${k}="false"`;
        return `:${k}="${JSON.stringify(v)}"`;
      })
      .join(" ");

    content += `    <${componentName}${propsStr ? " " + propsStr : ""} />\n`;
    content += `  </variant>\n\n`;
  }
  content += `</art>\n\n<script setup lang="ts">\nimport ${componentName} from '${relPath}'\n</script>\n`;

  return {
    variants,
    artFileContent: content,
    componentName,
  };
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
