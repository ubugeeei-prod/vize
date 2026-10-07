import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  realpathSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { test } from "node:test";
import {
  collectFiles,
  corpusRoot,
  validateFiles,
} from "../../tools/support/compat/github/canonical-corpus-identity.mjs";
import {
  createCommittedInventory,
  verifyCommittedInventory,
} from "../../tools/support/compat/github/canonical-corpus-inventory.mjs";

const git = (cwd, ...args) => execFileSync("git", args, { cwd, encoding: "utf8" }).trim();
const initialize = (cwd) => {
  mkdirSync(cwd, { recursive: true });
  git(cwd, "init", "--quiet", "--object-format=sha1");
  git(cwd, "config", "user.name", "Fixture addition control");
  git(cwd, "config", "user.email", "fixture@example.invalid");
};
const commit = (cwd) => {
  git(cwd, "add", ".");
  git(cwd, "commit", "--quiet", "-m", "fixture");
};
const write = (root, path, bytes) => {
  mkdirSync(dirname(join(root, path)), { recursive: true });
  writeFileSync(join(root, path), bytes);
};
const baseline = {
  "Original.vue": "<template> wholly valid authored fixture </template>\r\n",
  "nested/Second.vue": "<template>{{ second }}</template>\n",
  "node_modules/Excluded.vue": "<template>dependency</template>\n",
  "_git-worktrees/Excluded.vue": "<template>local checkout</template>\n",
  "README.txt": "All selected Vue fixtures are valid; no old invalid skips exist.\n",
};

function fixture(files = baseline, symlinks = {}) {
  const temporary = realpathSync(mkdtempSync(join(tmpdir(), "canonical-addition-")));
  const root = join(temporary, "root");
  const links = [];
  initialize(root);
  const add = (name, contents, aliases = {}) => {
    const source = join(temporary, "sources", name);
    initialize(source);
    for (const [path, bytes] of Object.entries(contents)) write(source, path, bytes);
    for (const [path, target] of Object.entries(aliases)) {
      mkdirSync(dirname(join(source, path)), { recursive: true });
      symlinkSync(target.replace("$TEMP", temporary), join(source, path));
    }
    commit(source);
    const path = `${corpusRoot}/${name}`;
    git(root, "-c", "protocol.file.allow=always", "submodule", "add", "--quiet", source, path);
    commit(root);
    const link = { path, sha: git(source, "rev-parse", "HEAD") };
    links.push(link);
    return { link, path: join(root, path) };
  };
  const first = add("original", files, symlinks);
  const inventory = () => createCommittedInventory(root, links);
  const observed = () => collectFiles(join(root, corpusRoot));
  const valid = () => {
    const result = inventory();
    assert.deepEqual(verifyCommittedInventory(result.proof, links), result.files);
    assert.match(validateFiles(observed(), result.files), /^[0-9a-f]{64}$/);
    return result;
  };
  return {
    root,
    temporary,
    first,
    links,
    add,
    inventory,
    observed,
    valid,
    cleanup: () => rmSync(temporary, { recursive: true, force: true }),
  };
}

test("committed additions to a pinned fixture and a new gitlink require the complete new corpus", () => {
  const f = fixture();
  try {
    const before = f.valid();
    assert.equal(before.files.length, 2);
    write(f.first.path, "Added.vue", "<template>new valid authored input</template>\n");
    commit(f.first.path);
    f.first.link.sha = git(f.first.path, "rev-parse", "HEAD");
    commit(f.root);
    const expanded = f.valid();
    assert.equal(expanded.files.length, 3);
    assert.throws(() => verifyCommittedInventory(before.proof, f.links));
    f.add("second-fixture", { "New.vue": "<template>new adopter fixture</template>\n" });
    const added = f.valid();
    assert.equal(added.files.length, 4);
    assert.equal(f.links.length, 2);
    assert(added.files.some(([path]) => path === "second-fixture/New.vue"));
    assert.throws(() => verifyCommittedInventory(expanded.proof, f.links));
  } finally {
    f.cleanup();
  }
});

test("omitting a valid Vue file is refused without depending on old invalid fixture skips", () => {
  const f = fixture();
  try {
    const expected = f.valid().files;
    assert.deepEqual(
      expected.map(([path]) => path),
      ["original/Original.vue", "original/nested/Second.vue"],
    );
    rmSync(join(f.first.path, "nested/Second.vue"));
    assert.throws(() => validateFiles(f.observed(), expected));
  } finally {
    f.cleanup();
  }
});

test("same-count replacements, untracked additions and modified authored bytes cannot pass", () => {
  for (const mutation of ["replacement", "untracked", "modified"]) {
    const f = fixture();
    try {
      const expected = f.valid().files;
      if (mutation === "replacement") {
        rmSync(join(f.first.path, "Original.vue"));
        write(f.first.path, "Replacement.vue", baseline["Original.vue"]);
      } else if (mutation === "untracked") {
        write(f.first.path, "Untracked.vue", "<template>not committed</template>\n");
      } else {
        write(f.first.path, "Original.vue", "<template>changed bytes</template>\n");
      }
      const actual = f.observed();
      assert.equal(actual.length, expected.length + Number(mutation === "untracked"));
      assert.throws(() => validateFiles(actual, expected), mutation);
    } finally {
      f.cleanup();
    }
  }
});

test("committed tree proofs reject wrong owners, omitted graphs and authenticated-object forgeries", () => {
  const f = fixture(baseline, { "Alias.vue": "Original.vue" });
  try {
    const { proof } = f.valid();
    assert.equal(proof.schema, "vize.canonical-committed-inventory");
    assert.equal(proof.version, 1);
    const subtree = git(f.first.path, "rev-parse", "HEAD:nested");
    const alias = git(f.first.path, "rev-parse", "HEAD:Alias.vue");
    const mutations = [
      (value) => value.repositories.pop(),
      (value) => value.repositories.push(structuredClone(value.repositories[0])),
      (value) => {
        value.repositories[0].path = "tests/_fixtures/_git/foreign";
      },
      (value) => {
        value.repositories[0].sha = "f".repeat(40);
      },
      (value) => {
        value.repositories[0].objects[0].id = "f".repeat(40);
      },
      (value) => {
        value.repositories[0].objects.find((object) => object.type === "tree").type = "blob";
      },
      (value) => {
        value.repositories[0].objects[0].bytes =
          Buffer.from("forged raw object").toString("base64");
      },
      (value) => {
        value.repositories[0].objects = value.repositories[0].objects.filter(
          (object) => object.type !== "commit",
        );
      },
      (value) => {
        value.repositories[0].objects = value.repositories[0].objects.filter(
          (object) => object.id !== subtree,
        );
      },
      (value) => {
        value.repositories[0].objects = value.repositories[0].objects.filter(
          (object) => object.id !== alias,
        );
      },
      (value) => {
        value.repositories[0].objects.push(structuredClone(value.repositories[0].objects[0]));
      },
    ];
    for (const [index, mutation] of mutations.entries()) {
      const forged = structuredClone(proof);
      mutation(forged);
      assert.throws(() => verifyCommittedInventory(forged, f.links), `Proof mutation ${index}`);
    }
    assert.throws(() => verifyCommittedInventory(proof, [{ ...f.links[0], sha: "e".repeat(40) }]));
    assert.throws(() => verifyCommittedInventory(proof, []));
  } finally {
    f.cleanup();
  }
});

test("committed file and directory symlinks observe dereferenced regular blobs at every walked path", () => {
  const f = fixture(
    {
      "source.txt": "<template>file symlink target</template>\r\n",
      "actual/Original.vue": "<template>directory symlink target</template>\n",
    },
    { "Alias.vue": "source.txt", "directory-alias": "actual" },
  );
  try {
    const expected = f.valid().files;
    const sourceOid = git(f.first.path, "rev-parse", "HEAD:source.txt");
    const originalOid = git(f.first.path, "rev-parse", "HEAD:actual/Original.vue");
    assert.deepEqual(expected, [
      ["original/Alias.vue", sourceOid],
      ["original/actual/Original.vue", originalOid],
      ["original/directory-alias/Original.vue", originalOid],
    ]);
    const actual = f.observed();
    assert.deepEqual(
      actual.map(([path, , , blob]) => [path, blob]),
      expected,
    );
    assert.equal(actual[0][2], readFileSync(join(f.first.path, "source.txt")).length);
    assert.notEqual(sourceOid, git(f.first.path, "rev-parse", "HEAD:Alias.vue"));
  } finally {
    f.cleanup();
  }
  const physical = fixture(
    {
      "Target.vue": "<template>lexical target</template>\n",
      "deeper/Target.vue": "<template>physical target</template>\n",
      "deeper/actual/.keep": "directory owner\n",
    },
    {
      "directory-alias": "deeper/actual",
      "Alias.vue": "directory-alias/../Target.vue",
      "Repeat.vue": "directory-alias/../../directory-alias/../Target.vue",
    },
  );
  try {
    const actual = physical.observed();
    const target = git(physical.first.path, "rev-parse", "HEAD:deeper/Target.vue");
    assert.notEqual(target, git(physical.first.path, "rev-parse", "HEAD:Target.vue"));
    const { files } = physical.inventory();
    for (const path of ["original/Alias.vue", "original/Repeat.vue"]) {
      assert.equal(actual.find(([name]) => name === path)[3], target);
      assert.equal(
        files.find(([name]) => name === path)[1],
        target,
        "Symlink components must be dereferenced before processing a following '..'",
      );
    }
    assert.match(validateFiles(actual, files), /^[0-9a-f]{64}$/);
  } finally {
    physical.cleanup();
  }
});

test("original dependency-directory exclusions apply before resolving their unselected symlink targets", () => {
  const f = fixture(
    { "Original.vue": "<template>selected authored fixture</template>\n" },
    { node_modules: "$TEMP/excluded-directory", "_git-worktrees": "$TEMP/excluded-directory" },
  );
  try {
    write(
      f.temporary,
      "excluded-directory/Excluded.vue",
      "<template>unselected dependency</template>\n",
    );
    const actual = f.observed();
    assert.deepEqual(
      actual.map(([path]) => path),
      ["original/Original.vue"],
    );
    const expected = f.valid().files;
    assert.deepEqual(
      expected.map(([path]) => path),
      ["original/Original.vue"],
    );
  } finally {
    f.cleanup();
  }
});

test("cross-fixture symlinks dereference only targets owned by another complete pinned Git tree", () => {
  const f = fixture(
    { "Original.vue": "<template>first pinned owner</template>\n" },
    { "Borrowed.vue": "../second-fixture/New.vue", "borrowed-directory": "../second-fixture" },
  );
  try {
    const second = f.add("second-fixture", {
      "New.vue": "<template>second pinned owner</template>\n",
    });
    const expected = f.valid().files;
    const blob = git(second.path, "rev-parse", "HEAD:New.vue");
    assert.equal(expected.length, 4);
    for (const path of [
      "original/Borrowed.vue",
      "original/borrowed-directory/New.vue",
      "second-fixture/New.vue",
    ])
      assert.deepEqual(
        expected.find(([name]) => name === path),
        [path, blob],
      );
    const { proof } = f.inventory();
    assert.throws(() => verifyCommittedInventory(proof, [f.links[0]]));
  } finally {
    f.cleanup();
  }
});

test("cyclic and foreign committed symlinks cannot manufacture a pinned corpus", () => {
  for (const [name, target] of [
    ["cycle", "."],
    ["FileCycle.vue", "FileCycle.vue"],
    ["FileSlash.vue", "Original.vue/"],
    ["FileDotdot.vue", "Original.vue/../Original.vue"],
    ["Foreign.vue", "$TEMP/Outside.vue"],
    ["foreign-directory", "$TEMP/outside-directory"],
    ["Escaping.vue", "../../../../../Outside.vue"],
  ]) {
    const f = fixture({ "Original.vue": "<template>valid</template>\n" }, { [name]: target });
    try {
      write(f.temporary, "Outside.vue", "<template>unowned file</template>\n");
      write(
        f.temporary,
        "outside-directory/Outside.vue",
        "<template>unowned directory</template>\n",
      );
      assert.throws(() => f.inventory(), `${name}: Git proof must reject an unowned target`);
      assert.throws(() => f.observed(), `${name}: worktree manifest must reject the same target`);
    } finally {
      f.cleanup();
    }
  }
});
