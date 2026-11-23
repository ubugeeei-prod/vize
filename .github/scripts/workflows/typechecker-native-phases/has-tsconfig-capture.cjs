console.log(
  Boolean(
    JSON.parse(
      require("node:fs").readFileSync(process.env.RUNNER_TEMP + "/native-source-recipe.json"),
    ).originalEnvironment.VIZE_TSCONFIG_TYPES_CAPTURE,
  ),
);
