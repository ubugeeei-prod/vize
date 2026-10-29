/** Fresh-generation carriers; the 10k binding sentinels isolate empty/builtin/global lookup costs. */
export const scopeBindingCases = [
  {
    shape: "camel",
    count: 32,
    scriptBindings: 32,
  },
  {
    shape: "camel",
    count: 128,
    scriptBindings: 128,
  },
  {
    shape: "camel",
    count: 512,
    scriptBindings: 512,
  },
  {
    shape: "missing",
    count: 32,
    scriptBindings: 32,
  },
  {
    shape: "missing",
    count: 128,
    scriptBindings: 128,
  },
  {
    shape: "missing",
    count: 512,
    scriptBindings: 512,
  },
  {
    shape: "dynamic",
    count: 32,
    scriptBindings: 32,
  },
  {
    shape: "dynamic",
    count: 128,
    scriptBindings: 128,
  },
  {
    shape: "dynamic",
    count: 512,
    scriptBindings: 512,
  },
  {
    shape: "empty",
    count: 32,
    scriptBindings: 32,
  },
  {
    shape: "empty",
    count: 128,
    scriptBindings: 128,
  },
  {
    shape: "empty",
    count: 512,
    scriptBindings: 512,
  },
  {
    shape: "empty",
    count: 0,
    scriptBindings: 10000,
  },
  {
    shape: "builtin",
    count: 32,
    scriptBindings: 10000,
  },
  {
    shape: "global",
    count: 32,
    scriptBindings: 10000,
  },
  {
    shape: "missing",
    count: 32,
    scriptBindings: 10000,
  },
] as const;
