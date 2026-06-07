#!/usr/bin/env node
// Convert a Vize TOML test fixture into an equivalent Apple Pkl module, matching
// the hand-authored convention in tests/fixtures/sfc/basic.pkl:
//
//   - leading `#` comment lines become `//` comments
//   - top-level scalars (mode/test_type) stay as plain assignments
//   - arrays of tables become `new Listing { new { ... } }`
//   - multi-line strings use Pkl `"""` blocks whose content is indented to the
//     key's column so Pkl's indentation stripping restores the original text
//     verbatim; a value ending in a newline gets a trailing blank line so the
//     newline round-trips exactly.
//
// The result is byte-exact: the Rust snapshot / expected-output tests pass
// without regenerating snapshots, which is what verifies the conversion.
//
// Usage: node tools/fixtures/toml-to-pkl.mjs <fixture.toml> [more.toml ...]

import { readFileSync, writeFileSync } from "node:fs";
import TOML from "@iarna/toml";

const TQ = '"'.repeat(3);

function leadingComment(text) {
  const out = [];
  for (const line of text.split("\n")) {
    const s = line.trim();
    if (s.startsWith("#")) out.push("//" + s.slice(1));
    else if (s === "") {
      if (out.length) break;
    } else break;
  }
  return out;
}

function pklInlineString(value) {
  // A plain string is fine when there is nothing to escape and no `\(...)`
  // interpolation to worry about; otherwise fall back to a raw `#"..."#` fence.
  if (!/["\\]/.test(value)) return `"${value}"`;
  let n = 1;
  while (value.includes('"' + "#".repeat(n))) n++;
  const fence = "#".repeat(n);
  return `${fence}"${value}"${fence}`;
}

function pklMultiline(value, indent) {
  const pad = " ".repeat(indent);
  const lines = value.split("\n");
  const body = lines.length > 1 && lines[lines.length - 1] === "" ? lines.slice(0, -1) : lines;
  const out = [TQ];
  for (const ln of body) out.push(ln ? pad + ln : "");
  if (value.endsWith("\n")) out.push("");
  out.push(pad + TQ);
  return out;
}

function emitValue(value, indent, out) {
  // out[-1] already ends with `<key> = ` (or `new `); append the value.
  const pad = "  ".repeat(indent);
  const last = out.length - 1;
  if (typeof value === "boolean") {
    out[last] += value ? "true" : "false";
  } else if (typeof value === "number" || typeof value === "bigint") {
    out[last] += String(value);
  } else if (typeof value === "string") {
    if (value.includes("\n")) {
      const keyCol = out[last].length - out[last].replace(/^ +/, "").length;
      const rendered = pklMultiline(value, keyCol);
      out[last] += rendered[0];
      out.push(...rendered.slice(1));
    } else {
      out[last] += pklInlineString(value);
    }
  } else if (Array.isArray(value)) {
    out[last] += "new Listing {";
    for (const child of value) {
      if (child && typeof child === "object" && !Array.isArray(child)) {
        out.push(`${pad}  new {`);
        for (const [k, v] of Object.entries(child)) {
          out.push(`${pad}    ${k} = `);
          emitValue(v, indent + 2, out);
        }
        out.push(`${pad}  }`);
      } else {
        out.push(`${pad}  `);
        emitValue(child, indent + 1, out);
      }
    }
    out.push(`${pad}}`);
  } else if (value && typeof value === "object") {
    out[last] += "new {";
    for (const [k, v] of Object.entries(value)) {
      out.push(`${pad}  ${k} = `);
      emitValue(v, indent + 1, out);
    }
    out.push(`${pad}}`);
  } else {
    throw new Error(`unsupported value: ${JSON.stringify(value)}`);
  }
}

function convert(tomlPath, amends) {
  const text = readFileSync(tomlPath, "utf8");
  const data = TOML.parse(text);
  const out = leadingComment(text);
  if (out.length) out.push("");
  if (amends) {
    // `amends` makes the fixture a typed instance of the schema module: Pkl
    // checks every assigned property/field against it.
    out.push(`amends ${JSON.stringify(amends)}`);
    out.push("");
  }
  for (const [key, value] of Object.entries(data)) {
    if (amends && Array.isArray(value)) {
      // Amend the schema-declared Listing so its `new {}` elements are typed
      // against the element class (e.g. `Case`) rather than `Dynamic`.
      out.push(`${key} {`);
      for (const child of value) {
        if (child && typeof child === "object" && !Array.isArray(child)) {
          out.push("  new {");
          for (const [k, v] of Object.entries(child)) {
            out.push(`    ${k} = `);
            emitValue(v, 2, out);
          }
          out.push("  }");
        } else {
          out.push("  ");
          emitValue(child, 1, out);
        }
      }
      out.push("}");
    } else {
      out.push(`${key} = `);
      emitValue(value, 0, out);
    }
    out.push("");
  }
  while (out.length && out[out.length - 1] === "") out.pop();
  return out.join("\n") + "\n";
}

const rawArgs = process.argv.slice(2);
let amends = null;
const args = [];
for (const a of rawArgs) {
  if (a.startsWith("--amends=")) amends = a.slice("--amends=".length);
  else args.push(a);
}
if (args.length === 0) {
  console.error("usage: node toml-to-pkl.mjs [--amends=Schema.pkl] <fixture.toml> ...");
  process.exit(2);
}
for (const arg of args) {
  const pklPath = arg.replace(/\.toml$/, ".pkl");
  writeFileSync(pklPath, convert(arg, amends));
  console.log(`${arg} -> ${pklPath}`);
}
