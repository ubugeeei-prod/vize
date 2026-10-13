declare module "virtual:vize-svg-namespace-8624/runtime" {
  export { createApp, createVaporApp, nextTick } from "vue";
}
declare module "virtual:vize-svg-namespace-8624/vapor" {
  const component: Parameters<typeof import("vue").createVaporApp>[0];
  export default component;
  export const emittedSha256: string;
}
declare module "virtual:vize-svg-namespace-8624/vdom" {
  const component: Parameters<typeof import("vue").createApp>[0];
  export default component;
  export const emittedSha256: string;
}
declare module "virtual:vize-svg-namespace-8624/state-vapor" {
  export const state: Record<string, unknown>;
}
declare module "virtual:vize-svg-namespace-8624/state-vdom" {
  export const state: Record<string, unknown>;
}
declare module "virtual:vize-svg-namespace-8624/controls" {
  const row: {
    context: Record<string, unknown>;
    steps: { patch: Record<string, unknown> }[];
    boundIds: string[];
    expectedTrees: unknown[];
    expectedNamespacePhases: string[][];
    expectedClassPhases: string[][];
    expectedColorPhases: string[][];
  };
  export default row;
  export const custody: {
    sourceSha256: string;
    controlsSha256: string;
    runtimeVersion: string;
    checkout: string;
    bindingSha256: string;
    productProvider: string;
    oracleProvider: string;
    publicationQualification: false;
  };
}
