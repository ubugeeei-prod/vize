import { gitObjectId } from "../../../tools/support/compat/github/canonical-corpus-inventory.mjs";
import { sha256 } from "../../../tools/support/compat/github/canonical-corpus-identity.mjs";

export function canonicalWorkerInventory(fileCount = 42998, repositoryCount = 147) {
  const bytes = Buffer.from("<template> fixture </template>\r\n");
  const blob = gitObjectId("blob", bytes);
  const files = [];
  const repositories = [];
  for (let project = 0; project < repositoryCount; project += 1) {
    const prefix = `project-${String(project).padStart(3, "0")}`;
    const entries = [];
    for (let index = project; index < fileCount; index += repositoryCount) {
      const name = `Original-${String(index).padStart(5, "0")}.vue`;
      entries.push(Buffer.from(`100644 ${name}\0`), Buffer.from(blob, "hex"));
      files.push([`${prefix}/${name}`, sha256(bytes), bytes.length, blob]);
    }
    const tree = Buffer.concat(entries);
    const treeId = gitObjectId("tree", tree);
    const commit = Buffer.from(`tree ${treeId}\n\ncomplete fixture control\n`);
    const sha = gitObjectId("commit", commit);
    repositories.push({
      path: `tests/_fixtures/_git/${prefix}`,
      sha,
      objects: [
        { id: sha, type: "commit", bytes: commit.toString("base64") },
        { id: treeId, type: "tree", bytes: tree.toString("base64") },
      ],
    });
  }
  files.sort(([left], [right]) => (left < right ? -1 : left > right ? 1 : 0));
  const proof = { schema: "vize.canonical-committed-inventory", version: 1, repositories };
  return { files, proof, gitlinks: repositories.map(({ path, sha }) => ({ path, sha })) };
}
