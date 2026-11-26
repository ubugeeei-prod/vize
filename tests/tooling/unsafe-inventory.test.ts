import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { parse as parseYaml } from "yaml";

// docs/davinci/decisions/2026-11-24-unsafe-model-checking.md: every `unsafe`
// token in product Rust (the module trees of crates/*/src and davinci/*/src,
// without `#[cfg(test)]` modules) is a row of the inventory, every Kani
// harness the inventory names exists and runs in the Davinci Kani workflow, and
// every Creusot proof it names sits in a kernel the Davinci Creusot workflow
// replays.

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const inventoryFile = path.join(
  repoRoot,
  "docs",
  "davinci",
  "decisions",
  "2026-11-24-unsafe-model-checking.md",
);
const workflowFile = path.join(repoRoot, ".github", "workflows", "07-proofs.yml");
const creusotWorkflowFile = path.join(repoRoot, ".github", "workflows", "07-proofs-creusot.yml");
const creusotDir = path.join(repoRoot, "tests", "formal", "creusot");

/** Source text with comments and string/char literals blanked (newlines kept). */
function codeOnly(source: string): string {
  const out = source.split("");
  const blank = (from: number, to: number) => {
    for (let k = from; k < Math.min(to, out.length); k++) if (out[k] !== "\n") out[k] = " ";
  };
  let i = 0;
  while (i < source.length) {
    const c = source[i]!;
    const next = source[i + 1];
    const previous = source[i - 1] ?? "";
    if (c === "/" && next === "/") {
      const end = source.indexOf("\n", i);
      const stop = end === -1 ? source.length : end;
      blank(i, stop);
      i = stop;
    } else if (c === "/" && next === "*") {
      let depth = 1;
      let j = i + 2;
      while (j < source.length && depth > 0) {
        if (source[j] === "/" && source[j + 1] === "*") {
          depth++;
          j += 2;
        } else if (source[j] === "*" && source[j + 1] === "/") {
          depth--;
          j += 2;
        } else j++;
      }
      blank(i, j);
      i = j;
    } else if (
      (c === "r" || c === "b") &&
      (next === '"' || next === "#" || next === "r") &&
      !/\w/u.test(previous)
    ) {
      const raw = /^b?r(#*)"/u.exec(source.slice(i, i + 72));
      if (raw) {
        const close = `"${raw[1]!}`;
        const end = source.indexOf(close, i + raw[0].length);
        const stop = end === -1 ? source.length : end + close.length;
        blank(i, stop);
        i = stop;
      } else i++;
    } else if (c === '"') {
      let j = i + 1;
      while (j < source.length && source[j] !== '"') j += source[j] === "\\" ? 2 : 1;
      blank(i, j + 1);
      i = j + 1;
    } else if (c === "'") {
      const literal = /^'(?:\\(?:x[0-9a-fA-F]{2}|u\{[0-9a-fA-F]+\}|.)|[^\\'\n])'/u.exec(
        source.slice(i, i + 16),
      );
      if (literal) {
        blank(i, i + literal[0].length);
        i += literal[0].length;
      } else i++;
    } else i++;
  }
  return out.join("");
}

/** Whether the outer attributes right above `line` gate the item on tests or verifiers. */
function verificationOnly(lines: string[], line: number): boolean {
  for (let k = line - 1; k >= 0; k--) {
    const text = lines[k]!.trim();
    if (!text.startsWith("#[")) return false;
    if (/^#\[cfg\((?:test|kani|creusot|all\((?:test|kani|creusot)\b)/u.test(text)) return true;
  }
  return false;
}

/** Product files and their blanked code: crate-root module trees minus test-only modules. */
function productFiles(): Map<string, string> {
  const files = new Map<string, string>();
  const visit = (file: string, childDir: string) => {
    const raw = fs.readFileSync(file, "utf8");
    // Blanking is the slow part; a file without the word only needs its
    // line-initial `mod` declarations (one inside a comment fails as missing).
    const mentions = raw.includes("unsafe");
    const code = mentions ? codeOnly(raw) : raw;
    const relative = path.relative(repoRoot, file).split(path.sep).join("/");
    files.set(relative, mentions ? productCode(raw, code) : "");
    const rawLines = raw.split("\n");
    code.split("\n").forEach((text, line) => {
      const declared = /^(?:pub(?:\([^)]*\))?\s+)?mod (\w+);/u.exec(text);
      if (!declared || verificationOnly(rawLines, line)) return;
      const flat = path.join(childDir, `${declared[1]!}.rs`);
      const nested = path.join(childDir, declared[1]!, "mod.rs");
      const target = fs.existsSync(flat) ? flat : nested;
      assert.ok(fs.existsSync(target), `${file} declares missing module ${declared[1]}`);
      const isDirOwner = path.basename(target) === "mod.rs";
      visit(target, isDirOwner ? path.dirname(target) : target.slice(0, -3));
    });
  };
  for (const group of ["crates", "davinci"]) {
    for (const crate of fs.readdirSync(path.join(repoRoot, group)).sort()) {
      // A generated copy compiles its plain crate's source, whose rows cover
      // it (tools/commands/generate/composed-crates.ts).
      if (crate.endsWith("_composed")) continue;
      const src = path.join(repoRoot, group, crate, "src");
      for (const root of ["lib.rs", "main.rs"]) {
        if (fs.existsSync(path.join(src, root))) visit(path.join(src, root), src);
      }
      const bin = path.join(src, "bin");
      if (fs.existsSync(bin)) {
        for (const entry of fs.readdirSync(bin).filter((name) => name.endsWith(".rs"))) {
          visit(path.join(bin, entry), path.join(bin, entry.slice(0, -3)));
        }
      }
    }
  }
  return files;
}

/** Code of a product file without its inline test-only modules. */
function productCode(raw: string, code: string): string {
  const rawLines = raw.split("\n");
  const out = code.split("");
  let offset = 0;
  code.split("\n").forEach((text, line) => {
    const inline = /^\s*(?:pub(?:\([^)]*\))?\s+)?mod \w+\s*\{/u.exec(text);
    if (inline && verificationOnly(rawLines, line)) {
      let depth = 0;
      for (let k = offset + text.indexOf("{"); k < out.length; k++) {
        if (code[k] === "{") depth++;
        else if (code[k] === "}" && --depth === 0) break;
        if (out[k] !== "\n") out[k] = " ";
      }
    }
    offset += text.length + 1;
  });
  return out.join("");
}

/** `unsafe` tokens per product file. */
function scannedSites(): Map<string, number> {
  const sites = new Map<string, number>();
  for (const [file, code] of productFiles()) {
    const count = [...code.matchAll(/\bunsafe\b/gu)].length;
    if (count > 0) sites.set(file, count);
  }
  return sites;
}

type Row = { file: string; item: string; sites: number; kind: string; verifier: string };

function inventoryRows(): Row[] {
  const text = fs.readFileSync(inventoryFile, "utf8");
  const section = text.split(/^## /mu).find((part) => part.startsWith("Inventory"));
  assert.ok(section, "the decision record has an Inventory section");
  const rows: Row[] = [];
  for (const line of section.split("\n")) {
    const cells = line.split("|").map((cell) => cell.trim());
    if (cells.length < 8 || !cells[1]!.startsWith("`")) continue;
    rows.push({
      file: cells[1]!.replaceAll("`", ""),
      item: cells[2]!.replaceAll("`", ""),
      sites: Number(cells[3]),
      kind: cells[4]!,
      verifier: cells[6]!,
    });
  }
  return rows;
}

test("every unsafe token in product Rust is an inventory row", () => {
  const scanned = scannedSites();
  assert.ok(scanned.size > 10, "the scanner finds the known unsafe islands");
  const listed = new Map<string, number>();
  for (const row of inventoryRows()) {
    assert.ok(Number.isInteger(row.sites) && row.sites > 0, `${row.file} ${row.item} sites`);
    listed.set(row.file, (listed.get(row.file) ?? 0) + row.sites);
  }
  const unlisted = [...scanned]
    .filter(([file, count]) => listed.get(file) !== count)
    .map(([file, count]) => `${file}: ${count} unsafe token(s), ${listed.get(file) ?? 0} listed`);
  const stale = [...listed.keys()].filter((file) => !scanned.has(file));
  assert.deepEqual(unlisted, [], "add the new unsafe site to the inventory with its verifier");
  assert.deepEqual(stale, [], "remove inventory rows whose unsafe left product code");
});

test("every inventory row names its item, class and verifier", () => {
  for (const row of inventoryRows()) {
    const source = fs.readFileSync(path.join(repoRoot, row.file), "utf8");
    const name = row.item.replace(/^(?:fn|impl|static|extern)\s+/u, "");
    assert.ok(source.includes(name), `${row.file} contains ${row.item}`);
    assert.match(row.kind, /^\((?:a|b|c)\)/u, `${row.file} ${row.item} has a class`);
    assert.match(
      row.verifier,
      /kani `|creusot `|miri `|fuzz `|test `|out of scope/u,
      `${row.file} ${row.item} names a verifier`,
    );
    if (row.kind.startsWith("(a)")) {
      assert.match(
        row.verifier,
        /kani `|creusot `|not model-checked/u,
        `${row.file} ${row.item}: class (a) is Kani-checked, Creusot-proved or says why not`,
      );
    }
  }
});

type Workflow = {
  on: { pull_request: { paths: string[] } };
  jobs: Record<string, { steps: { with?: { args?: string } }[] }>;
};

test("every listed Kani harness exists and the Kani workflow runs it", () => {
  const workflow = parseYaml(fs.readFileSync(workflowFile, "utf8")) as Workflow;
  const args = Object.values(workflow.jobs).flatMap((job) =>
    job.steps.map((step) => step.with?.args ?? ""),
  );
  const harnesses = inventoryRows().flatMap((row) =>
    [...row.verifier.matchAll(/kani `([^`:]+)::(\w+)`/gu)].map((match) => ({
      file: match[1]!,
      name: match[2]!,
      row,
    })),
  );
  assert.ok(harnesses.length > 0, "the inventory names Kani harnesses");
  for (const { file, name, row } of harnesses) {
    const harness = fs.readFileSync(path.join(repoRoot, file), "utf8");
    assert.match(
      harness,
      new RegExp(`#\\[kani::proof\\]\\n(?:#\\[[^\\n]*\\]\\n)*fn ${name}\\(`, "u"),
      `${file} defines proof harness ${name}`,
    );
    assert.match(harness, /kani::cover!/u, `${file} states non-vacuity covers`);
    assert.ok(workflow.on.pull_request.paths.includes(file), `workflow paths include ${file}`);
    assert.ok(
      workflow.on.pull_request.paths.includes(row.file),
      `workflow paths include the kernel ${row.file}`,
    );
    const manifest = `${file.split("/src/")[0]!}/Cargo.toml`;
    assert.ok(
      args.some(
        (line) =>
          line.includes(`--manifest-path ${manifest}`) &&
          [...line.matchAll(/--harness (\w+)/gu)].some((filter) => name.includes(filter[1]!)),
      ),
      `a Davinci Kani job runs ${name} from ${manifest}`,
    );
  }
});

/** Repository-relative proof roots of the Creusot workspace and the modules they declare. */
function creusotKernels(): Set<string> {
  const workspace = fs.readFileSync(path.join(creusotDir, "Cargo.toml"), "utf8");
  const members = [
    ...(workspace.match(/^members = \[([^\]]*)\]$/mu)?.[1] ?? "").matchAll(/"([^"]+)"/gu),
  ];
  const kernels = new Set<string>();
  for (const member of members) {
    const manifest = fs.readFileSync(path.join(creusotDir, member[1]!, "Cargo.toml"), "utf8");
    const lib = manifest.match(/^\[lib\]\npath = "([^"]+)"$/mu)?.[1];
    assert.ok(lib, `${member[1]} names its library root`);
    const root = path.resolve(creusotDir, member[1]!, lib);
    kernels.add(path.relative(repoRoot, root).split(path.sep).join("/"));
    for (const match of fs.readFileSync(root, "utf8").matchAll(/^\s*(?:pub\s+)?mod (\w+);$/gmu)) {
      const target = path.join(path.dirname(root), `${match[1]!}.rs`);
      kernels.add(path.relative(repoRoot, target).split(path.sep).join("/"));
    }
  }
  return kernels;
}

test("every listed Creusot proof is in a replayed kernel", () => {
  const workflow = parseYaml(fs.readFileSync(creusotWorkflowFile, "utf8")) as Workflow;
  const kernels = creusotKernels();
  const proofs = inventoryRows().flatMap((row) =>
    [...row.verifier.matchAll(/creusot `([^`:]+)::([\w:]+)`/gu)].map((match) => ({
      file: match[1]!,
      item: match[2]!,
      row,
    })),
  );
  assert.ok(proofs.length > 0, "the inventory names Creusot proofs");
  for (const { file, item, row } of proofs) {
    assert.ok(kernels.has(file), `${file} is a Creusot proof root or one of its modules`);
    assert.equal(file, row.file, `${row.file} ${row.item} is proved in its own file`);
    const source = fs.readFileSync(path.join(repoRoot, file), "utf8");
    const name = item.split("::").at(-1)!;
    assert.match(source, new RegExp(`\\bfn ${name}\\b`, "u"), `${file} defines ${item}`);
    assert.ok(
      workflow.on.pull_request.paths.includes(file),
      `Creusot workflow paths include ${file}`,
    );
  }
});
