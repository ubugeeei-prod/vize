type NuxtWithHooks = {
  hook(name: string, callback: (...args: unknown[]) => unknown): void;
};

const VIZE_CONTENT_MAPPER = {
  package: "vize",
  extensions: [".vue"],
} as const;

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function mapsVueFiles(value: unknown): boolean {
  return (
    isRecord(value) &&
    Array.isArray(value.extensions) &&
    value.extensions.some((extension) => extension === ".vue")
  );
}

export function injectVizeContentMapper(tsConfig: unknown): boolean {
  if (!isRecord(tsConfig)) {
    return false;
  }

  const configured = tsConfig.contentMappers;
  if (configured !== undefined && !Array.isArray(configured)) {
    return false;
  }

  const contentMappers = configured ?? [];
  if (contentMappers.some(mapsVueFiles)) {
    return false;
  }

  tsConfig.contentMappers = [
    ...contentMappers,
    { package: VIZE_CONTENT_MAPPER.package, extensions: [...VIZE_CONTENT_MAPPER.extensions] },
  ];
  return true;
}

export function registerNuxtContentMapper(nuxt: NuxtWithHooks, nuxtMajor: number): void {
  if (nuxtMajor === 2) {
    return;
  }

  nuxt.hook("prepare:types", (payload: unknown) => {
    if (isRecord(payload)) {
      injectVizeContentMapper(payload.tsConfig);
    }
  });
}
