import type { AutogenOptions, AutogenOutput, PropDefinition } from "./index.js";

export interface NativePropDefinition {
  name: string;
  propType: string;
  required: boolean;
  defaultValue?: unknown;
}

export interface NativeAutogenConfig {
  maxVariants?: number;
  includeDefault?: boolean;
  includeBooleanToggles?: boolean;
  includeEnumVariants?: boolean;
  includeBoundaryValues?: boolean;
  includeEmptyStrings?: boolean;
}

export interface NativeGeneratedVariant {
  name: string;
  isDefault: boolean;
  props: Record<string, unknown>;
  description?: string;
}

export interface NativeAutogenOutput {
  variants: NativeGeneratedVariant[];
  artFileContent: string;
  componentName: string;
}

export function toNativePropDefinitions(props: PropDefinition[]): NativePropDefinition[] {
  return props.map((prop) => ({
    name: prop.name,
    propType: prop.propType,
    required: prop.required,
    defaultValue: prop.defaultValue,
  }));
}

export function toNativeAutogenConfig(options: AutogenOptions): NativeAutogenConfig {
  return {
    maxVariants: options.maxVariants,
    includeDefault: options.includeDefault,
    includeBooleanToggles: options.includeBooleanToggles,
    includeEnumVariants: options.includeEnumVariants,
    includeBoundaryValues: options.includeBoundaryValues,
    includeEmptyStrings: options.includeEmptyStrings,
  };
}

export function fromNativeOutput(output: NativeAutogenOutput): AutogenOutput {
  return {
    variants: output.variants.map((variant) => ({
      name: variant.name,
      isDefault: variant.isDefault,
      props: variant.props,
      description: variant.description,
    })),
    artFileContent: output.artFileContent,
    componentName: output.componentName,
  };
}
