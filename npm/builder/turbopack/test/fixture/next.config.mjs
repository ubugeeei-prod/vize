import { withVize } from "@vizejs/turbopack-plugin";

export default withVize(
  {
    agentRules: false,
    reactStrictMode: false,
    productionBrowserSourceMaps: true,
    generateBuildId: () => "vize-turbopack-consumer",
  },
  {
    root: import.meta.dirname,
    styleLoaders: {
      css: [{ loader: `${import.meta.dirname}/once-loader.cjs` }],
      scss: [{ loader: "sass-loader", options: { sourceMap: true } }],
    },
  },
);
