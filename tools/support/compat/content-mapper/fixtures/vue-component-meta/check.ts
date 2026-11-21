import assert from "node:assert/strict";
import path from "node:path";
import * as tsAsync from "tsrs/async";
import * as tsAst from "tsrs/ast";
import { getComponentMeta } from "vue-component-meta";

const cwd = process.cwd();
const config = path.join(cwd, "tsconfig.json");
const api = new tsAsync.API({ cwd, runExternalCode: true });
try {
  const snapshot = await api.createSnapshot({ openProjects: [config] });
  try {
    const project = snapshot.getConfiguredProject(config);
    assert.ok(project);
    const component = await getComponentMeta(
      { ...tsAst, ...tsAsync },
      project,
      path.join(cwd, "Comp.vue"),
    );
    const props = Object.fromEntries(component.props.map((prop) => [prop.name, prop.required]));
    assert.deepEqual(
      { name: props.name, kind: props.kind, note: props.note },
      { name: true, kind: true, note: false },
    );
    assert.equal(
      component.props.find((prop) => prop.name === "name")?.description,
      "Display name.",
    );
    const diagnostics = await project.program.getSemanticDiagnostics(
      path.join(cwd, "Consumer.vue"),
    );
    assert.equal(diagnostics.length, 1, JSON.stringify(diagnostics));
    assert.equal(diagnostics[0].startPosition?.line, 6);
    assert.ok(
      diagnostics[0].messageChain?.some((message) =>
        message.text.includes("Property 'name' is missing"),
      ),
      JSON.stringify(diagnostics[0]),
    );
  } finally {
    await snapshot.dispose();
  }
} finally {
  await api.close();
}
