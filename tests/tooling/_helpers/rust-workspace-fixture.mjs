export function fixture(root = "/repo") {
  const pkg = (name, dependencies = [], directory = `crates/${name}`) => ({
    id: `package:${name}`,
    name,
    manifest_path: `${root}/${directory}/Cargo.toml`,
    dependencies,
  });
  const dependency = (name, kind = null, extra = {}) => ({
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
