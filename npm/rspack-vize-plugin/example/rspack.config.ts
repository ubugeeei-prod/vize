import { defineConfig } from "@rspack/cli";
import { rspack } from "@rspack/core";
import { VizePlugin } from "@vizejs/rspack-plugin";

const isDev = process.env.NODE_ENV === "development";
const isProduction = !isDev;

// Target browsers, see: https://github.com/browserslist/browserslist
const targets = ["last 2 versions", "> 0.2%", "not dead", "Firefox ESR"];

export default defineConfig({
  entry: {
    index: "./src/main.ts",
  },
  output: {
    clean: true,
  },
  resolve: {
    extensions: ["...", ".ts", ".tsx", ".jsx", ".vue"],
  },
  experiments: {
    css: true,
  },
  module: {
    rules: [
      // Vue SFC rule — VizePlugin auto-clones CSS rules for style sub-requests
      {
        test: /\.vue$/,
        loader: "@vizejs/rspack-plugin/loader",
      },
      // JSX / TSX components via Vize
      {
        test: /\.[jt]sx$/,
        exclude: /node_modules/,
        loader: "@vizejs/rspack-plugin/jsx-loader",
      },
      // TypeScript / JavaScript via SWC
      {
        test: /\.ts$/,
        exclude: /node_modules/,
        loader: "builtin:swc-loader",
        options: {
          jsc: {
            parser: {
              syntax: "typescript",
            },
          },
        },
        type: "javascript/auto",
      },
    ],
  },
  plugins: [
    new VizePlugin({
      isProduction,
      css: { native: true },
    }),
    new rspack.HtmlRspackPlugin({
      template: "./index.html",
    }),
  ],
  optimization: {
    minimizer: [
      new rspack.SwcJsMinimizerRspackPlugin(),
      new rspack.LightningCssMinimizerRspackPlugin({
        minimizerOptions: { targets },
      }),
    ],
  },
  devServer: {
    hot: true,
  },
});
