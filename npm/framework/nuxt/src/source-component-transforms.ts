type NuxtComponentTransformOptions = {
  experimental?: { compilerComponentResolution?: boolean };
};

/**
 * Nuxt can resolve template components, lazy hydration included, through Vue
 * compiler node transforms. Nuxt 4.7 does so by default on every compatibility
 * version. Vize owns that compiler boundary and runs no Vue node transforms, so
 * select Nuxt's equivalent source transforms before the native compiler runs.
 * Older Nuxt releases ignore the key.
 */
export function selectNuxtSourceComponentTransforms(options: NuxtComponentTransformOptions): void {
  options.experimental ||= {};
  options.experimental.compilerComponentResolution = false;
}
