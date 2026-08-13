import type { VitePluginVueCustomElementOption, VizeOptions } from "../types.ts";
import { createFilter } from "../utils/filter.ts";

export const PLUGIN_VUE_COMPAT_VERSION = "6.0.7";

const DEFAULT_CUSTOM_ELEMENT_PATTERN = /\.ce\.vue$/;

export interface PluginVueCompileOptions {
  styleTrim?: boolean;
  templateCacheHandlers?: boolean;
  templateComments?: boolean;
  templateHoistStatic?: boolean;
  templatePrefixIdentifiers?: boolean;
}

function booleanCompilerOption(value: unknown): boolean | undefined {
  return typeof value === "boolean" ? value : undefined;
}

export function resolvePluginVueCompileOptions(
  options: Pick<VizeOptions, "style" | "template">,
): PluginVueCompileOptions {
  const compilerOptions = options.template?.compilerOptions;
  const resolved: PluginVueCompileOptions = {
    styleTrim: options.style?.trim ?? true,
  };
  const templateCacheHandlers = booleanCompilerOption(compilerOptions?.cacheHandlers);
  const templateComments = booleanCompilerOption(compilerOptions?.comments);
  const templateHoistStatic = booleanCompilerOption(compilerOptions?.hoistStatic);
  const templatePrefixIdentifiers = booleanCompilerOption(compilerOptions?.prefixIdentifiers);
  if (templateCacheHandlers !== undefined) {
    resolved.templateCacheHandlers = templateCacheHandlers;
  }
  if (templateComments !== undefined) {
    resolved.templateComments = templateComments;
  }
  if (templateHoistStatic !== undefined) {
    resolved.templateHoistStatic = templateHoistStatic;
  }
  if (templatePrefixIdentifiers !== undefined) {
    resolved.templatePrefixIdentifiers = templatePrefixIdentifiers;
  }
  return resolved;
}

function resolveCustomElementOption(
  options: Pick<VizeOptions, "customElement" | "features">,
): VitePluginVueCustomElementOption {
  const featureCustomElement = options.features?.customElement;
  if (featureCustomElement) {
    return featureCustomElement;
  }
  if (options.customElement !== undefined) {
    return options.customElement;
  }
  return DEFAULT_CUSTOM_ELEMENT_PATTERN;
}

export function isPluginVueCustomElement(
  options: Pick<VizeOptions, "customElement" | "features">,
  filePath: string,
): boolean {
  const customElement = resolveCustomElementOption(options);
  if (typeof customElement === "boolean") {
    return customElement;
  }
  return createFilter(customElement, undefined)(filePath);
}
