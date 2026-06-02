import assert from "node:assert/strict";
import { classifyVitePluginRequest, rewriteViteImportMetaGlobBase } from "@vizejs/native";

{
  const request = classifyVitePluginRequest("/src/pages/Home.vue?definePage");
  assert.equal(request.path, "/src/pages/Home.vue");
  assert.equal(request.querySuffix, "?definePage");
  assert.equal(request.hasDefinePageQuery, true);
  assert.equal(request.hasMacroQuery, false);
  assert.equal(request.isVueSfcPath, true);
}

{
  const request = classifyVitePluginRequest("\0/src/pages/Home.vue.ts?macro=true");
  assert.equal(request.strippedVirtualPath, "/src/pages/Home.vue");
  assert.equal(request.normalizedVuePath, "\0/src/pages/Home.vue");
  assert.equal(request.isMacroVirtualId, true);
  assert.equal(request.isVueSfcPath, true);
  assert.equal(request.isVizeVirtual, true);
  assert.equal(request.isVizeSsrVirtual, false);
  assert.equal(request.vizeVirtualPath, "/src/pages/Home.vue");
}

{
  const request = classifyVitePluginRequest("\0vize-ssr:/src/pages/Home.vue.ts?used=true");
  assert.equal(request.isVizeVirtual, true);
  assert.equal(request.isVizeSsrVirtual, true);
  assert.equal(request.vizeVirtualPath, "/src/pages/Home.vue");
}

{
  const request = classifyVitePluginRequest("/@fs/src/entry.js?import");
  assert.equal(request.normalizedFsId, "/src/entry.js?import");
}

{
  const request = classifyVitePluginRequest(
    "/src/App.vue?vue&type=style&index=2&lang=scss&module&scoped=data-v-test",
  );
  assert.equal(request.isVueStyleQuery, true);
  assert.equal(request.styleLang, "scss");
  assert.equal(request.styleIndex, 2);
  assert.equal(request.styleScoped, "data-v-test");
  assert.equal(request.hasStyleModule, true);
  assert.equal(request.styleVirtualSuffix, ".module.scss");
}

{
  assert.equal(classifyVitePluginRequest("/src/Foo.client.vue").boundaryKind, "client");
  assert.equal(classifyVitePluginRequest("/src/Foo.server.vue").boundaryKind, "server");
  assert.equal(classifyVitePluginRequest("/src/Foo.vue").boundaryKind ?? undefined, undefined);
}

{
  const code = `const modules = import.meta.glob(["./demos/*.vue", "!../legacy/*.vue"], { eager: true });`;
  assert.equal(
    rewriteViteImportMetaGlobBase(code, "/project/src/App.vue", "/project"),
    `const modules = import.meta.glob(["/src/demos/*.vue", "!/legacy/*.vue"], { eager: true });`,
  );
}
