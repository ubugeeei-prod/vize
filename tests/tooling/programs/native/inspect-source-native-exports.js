const nativeEntry = process.argv[2];

const api = await import(nativeEntry);
console.log(
  JSON.stringify({
    cjs: typeof api.default.typeProjectInputs,
    esm: typeof api.typeProjectInputs,
    result: api.typeProjectInputs?.("/project/tsconfig.json", "7") ?? null,
  }),
);
