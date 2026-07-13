import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const registryPath = path.join(root, "tests", "_fixtures", "vue-ecosystem-fixtures.json");

interface FixtureProject {
  id: string;
  fixturePath: string;
  repository: string;
  license: { spdx: string; files: string[] };
}

interface FixtureRegistry {
  projects: FixtureProject[];
}

const requestedFixtures = [
  ["vue-element-admin", "https://github.com/PanJiaChen/vue-element-admin", "MIT"],
  ["element", "https://github.com/ElemeFE/element", "MIT"],
  ["lx-music-desktop", "https://github.com/lyswhut/lx-music-desktop", "Apache-2.0"],
  ["uni-app", "https://github.com/dcloudio/uni-app", "Apache-2.0"],
  ["vue2-elm", "https://github.com/bailicangdu/vue2-elm", "GPL-2.0-only"],
  ["filebrowser", "https://github.com/filebrowser/filebrowser", "Apache-2.0"],
  ["docsify", "https://github.com/docsifyjs/docsify", "MIT"],
  ["dashy", "https://github.com/lissy93/dashy", "MIT"],
  ["vue-devtools-v6", "https://github.com/vuejs/devtools-v6", "MIT"],
  ["vant", "https://github.com/youzan/vant", "MIT"],
  ["vuepress", "https://github.com/vuejs/vuepress", "MIT"],
  [
    "automa",
    "https://github.com/AutomaApp/automa",
    "AGPL-3.0-only AND LicenseRef-Automa-Commercial",
  ],
  ["vue-pure-admin", "https://github.com/pure-admin/vue-pure-admin", "MIT"],
  ["vue-manage-system", "https://github.com/lin-xin/vue-manage-system", "MIT"],
  ["vitepress", "https://github.com/vuejs/vitepress", "MIT"],
  ["vux", "https://github.com/airyland/vux", "MIT"],
  ["koel", "https://github.com/koel/koel", "MIT"],
  ["better-scroll", "https://github.com/ustbhuangyi/better-scroll", "MIT"],
  ["mint-ui", "https://github.com/ElemeFE/mint-ui", "MIT"],
  ["scalar", "https://github.com/scalar/scalar", "MIT"],
  ["soybean-admin", "https://github.com/soybeanjs/soybean-admin", "MIT"],
  ["zy-player", "https://github.com/Hunlongyu/ZY-Player", "MIT"],
  ["bootstrap-vue", "https://github.com/bootstrap-vue/bootstrap-vue", "MIT"],
  [
    "habitica",
    "https://github.com/HabitRPG/habitica",
    "GPL-3.0-only AND CC-BY-SA-3.0 AND CC-BY-NC-SA-3.0",
  ],
  ["tiny-rdm", "https://github.com/tiny-craft/tiny-rdm", "GPL-3.0-only"],
  ["mealie", "https://github.com/mealie-recipes/mealie", "AGPL-3.0-only"],
  ["mall-admin-web", "https://github.com/macrozheng/mall-admin-web", "Apache-2.0"],
  ["douyin", "https://github.com/zyronon/douyin", "GPL-3.0-only"],
  ["vuestic-admin", "https://github.com/epicmaxco/vuestic-admin", "MIT"],
  ["vue-storefront", "https://github.com/vuestorefront/vue-storefront", "MIT"],
  ["vue-virtual-scroller", "https://github.com/Akryum/vue-virtual-scroller", "MIT"],
  ["vue-echarts", "https://github.com/ecomfe/vue-echarts", "MIT"],
  ["gridea", "https://github.com/getgridea/gridea", "MIT"],
] as const;

test("requested classic and production fixtures stay pinned and licensed", () => {
  const registry = JSON.parse(fs.readFileSync(registryPath, "utf8")) as FixtureRegistry;
  const gitmodules = fs.readFileSync(path.join(root, ".gitmodules"), "utf8");

  for (const [id, repository, spdx] of requestedFixtures) {
    const fixturePath = `tests/_fixtures/_git/${id}`;
    const project = registry.projects.find((candidate) => candidate.id === id);
    assert.ok(project, `${id} should be registered`);
    assert.equal(project.fixturePath, fixturePath);
    assert.equal(project.repository, repository);
    assert.equal(project.license.spdx, spdx);

    const section = [
      `[submodule "${fixturePath}"]`,
      `\tpath = ${fixturePath}`,
      `\turl = ${repository}`,
      "\tshallow = true",
    ].join("\n");
    assert.ok(gitmodules.includes(section), `${id} should stay shallow in CI`);
  }
});
