// TS-24 builds one library closure twice, with and without default features.
export const stagePackages = [
  "vize_l0",
  "vize_l1",
  "vize_l2",
  "vize_l3",
  "vize_l4",
  "vize_l1_to_l2",
  "vize_l2_to_l3",
] as const;

function defaultBuild(packages: readonly string[]) {
  const args = packages.map((name) => `-p ${name}`).join(" ");
  return `cargo build ${args} --lib --target wasm32-wasip2`;
}

export const defaultLane = defaultBuild(stagePackages);
export const noDefaultLane = `${defaultLane} --no-default-features`;
export const portabilityStep = "Davinci no_std stage-library portability lanes (TS-24)";
export const portabilityRun = `${defaultLane} && ${noDefaultLane}`;

// The extraction fixture predates the move of the DOM emitter into L4.
const previous = defaultBuild(stagePackages.filter((name) => name !== "vize_l4"));
export const portabilityRunBeforeL4 = `${previous} && ${previous} --no-default-features`;
