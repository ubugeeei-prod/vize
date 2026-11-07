type NuxtComponentTransformOptions = {
  experimental?: { compilerComponentResolution?: boolean };
};

/**
 * Nuxt 5 defaults to resolving template components through Vue compiler node
 * transforms. Vize owns that compiler boundary, so select Nuxt's equivalent
 * source transform before the native compiler runs.
 */
export function selectNuxtSourceComponentTransforms(
  options: NuxtComponentTransformOptions,
  nuxtMajor: number,
): void {
  if (nuxtMajor < 5) return;
  options.experimental ||= {};
  options.experimental.compilerComponentResolution = false;
}
