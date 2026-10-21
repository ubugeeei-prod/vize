import type { CargoMetadata } from "../../../tools/support/compat/github/plan-affected-rust.ts";

type FixturePackage = CargoMetadata["packages"][number];
type FixtureDependency = FixturePackage["dependencies"][number] & {
  name: string;
  rename?: string;
  optional?: boolean;
  target?: string;
};
type DependencyExtras = Pick<FixtureDependency, "rename" | "optional" | "target">;

export function fixture(root = "/repo"): CargoMetadata {
  const pkg = (
    name: string,
    dependencies: FixtureDependency[] = [],
    directory = `crates/${name}`,
  ): FixturePackage => ({
    id: `package:${name}`,
    name,
    manifest_path: `${root}/${directory}/Cargo.toml`,
    dependencies,
  });
  const dependency = (
    name: string,
    kind: string | null = null,
    extra: DependencyExtras = {},
  ): FixtureDependency => ({
    name,
    path: `${root}/crates/${name}`,
    kind,
    ...extra,
  });
  const packages = [
    pkg("syntax"),
    pkg("compiler", [dependency("syntax", null, { rename: "parse", optional: true })]),
    pkg("consumer", [dependency("compiler", "build", { target: "cfg(windows)" })]),
    pkg("tests", [dependency("consumer", "dev")], "tests/shared_support"),
    pkg("unrelated"),
  ];
  return {
    packages,
    workspace_members: packages.map((current) => current.id),
    workspace_root: root,
  };
}
