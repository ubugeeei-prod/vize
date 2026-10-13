// Reviewed whole release-driver continuations; every other source pin remains literal.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";

const sha256 = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
const drivers: Record<
  string,
  {
    original: string;
    current: string;
    inverses: { current: string; original: string; exactOccurrences: number }[];
  }
> = {
  "tools/commands/release/pr.rs": {
    original: "db53a5ec6abf597c01dfb77b27477cfb10d69bdeba7018ab5920278d0a590d2d",
    current: "f6fd587823a20e6536adc7a427b12a71976ec50e0fb909c028322de4cf0b92dc",
    inverses: [
      {
        current:
          '        ["start-stable", policy, cut] => pr_pin::start_stable(policy, cut, &root),\n',
        original: "",
        exactOccurrences: 1,
      },
    ],
  },
  "tools/support/release/pr_pin.rs": {
    original: "273b5f7263feb8ddf5bf132ccf329a388f467bf5e1ab586d866ba2ce8eca904d",
    current: "447a9ef90a63a176b26f7072ba3df970ecf3ea750ec0a158254ff546f21b289b",
    inverses: [
      {
        current: '#[path = "pr_pin_stable.rs"]\nmod stable;\n',
        original: "",
        exactOccurrences: 1,
      },
      {
        current: "pub use start::{resume, start, start_stable};",
        original: "pub use start::{resume, start};",
        exactOccurrences: 1,
      },
      {
        current: "    stable: Option<stable::Stable>,\n",
        original: "",
        exactOccurrences: 1,
      },
      {
        current: "        cut: cut.clone(),\n",
        original: "        cut,\n",
        exactOccurrences: 1,
      },
      {
        current: "        stable: stable::from_body(body, &cut, repository, root)?,\n",
        original: "",
        exactOccurrences: 1,
      },
      {
        current:
          '    if let Some(context) = &source.stable {\n        if marker(body, "vize-release-bump")? != "patch" || stable::target(context, root)? != tag {\n            return Err("Stable source receipt is not the reviewed exact PATCH target".into());\n        }\n    }\n',
        original: "",
        exactOccurrences: 1,
      },
      {
        current: "stable::main_anchor(&source, &main, root)?;",
        original: "first_parent(&source.cut, &main, root)?;",
        exactOccurrences: 2,
      },
    ],
  },
};

export function authenticateCssReleaseDriver(
  filename: string,
  bytes: Buffer,
  expectedOriginalSha256: string,
) {
  const actual = sha256(bytes);
  const driver = Object.hasOwn(drivers, filename) ? drivers[filename] : undefined;
  if (!driver) {
    assert.equal(actual, expectedOriginalSha256, filename);
    return;
  }
  // Authenticate the original reference before accepting either entire known body.
  assert.equal(expectedOriginalSha256, driver.original, filename);
  if (actual === driver.original) return;
  let inverse = bytes.toString("utf8");
  // The reviewed footer parser changes only the entry's cache pin. Authenticate
  // the entire composed driver before reversing that one literal; the existing
  // stable-route inverse must still recover the original producer afterwards.
  if (
    filename === "tools/commands/release/pr.rs" &&
    actual === "f52e21a0b92203079a0a2f59cc509d7fe5864a64689cbde891412a7392bab73a"
  ) {
    const current = "//! Driver source pin: release-bot-envelope-variants-v16.";
    const original = "//! Driver source pin: decorated-integration-body-custody-v15.";
    assert.equal(inverse.split(current).length - 1, 1, filename);
    inverse = inverse.replace(current, original);
  }
  assert.equal(sha256(Buffer.from(inverse)), driver.current, filename);
  for (const step of driver.inverses) {
    assert.equal(inverse.split(step.current).length - 1, step.exactOccurrences, filename);
    inverse = inverse.replaceAll(step.current, step.original);
  }
  assert.equal(sha256(Buffer.from(inverse)), driver.original, filename);
}
