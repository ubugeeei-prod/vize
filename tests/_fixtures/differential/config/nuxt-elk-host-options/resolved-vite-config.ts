// Authored resolved-input reproduction; source versions and limits are in README.md.
export function elkSetupSfcViteConfig(isProduction: boolean) {
  return {
    vue: {
      script: { hoistStatic: undefined },
      isProduction,
      template: {
        compilerOptions: {},
        transformAssetUrls: {
          video: ["src", "poster"],
          source: ["src"],
          img: ["src"],
          image: ["xlink:href", "href"],
          use: ["xlink:href", "href"],
        },
      },
      features: { propsDestructure: true },
      include: [/\.vue$/, /\.setup\.[cm]?[jt]sx?$/],
    },
  };
}
