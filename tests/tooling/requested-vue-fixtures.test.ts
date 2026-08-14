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
  ["dbx", "https://github.com/t8y2/dbx", "Apache-2.0"],
  ["vue-material", "https://github.com/vuematerial/vue-material", "MIT"],
  ["datav", "https://github.com/DataV-Team/DataV", "MIT"],
  ["buefy", "https://github.com/buefy/buefy", "MIT"],
  ["cube-ui", "https://github.com/didi/cube-ui", "Apache-2.0"],
  ["youtube-dl-gui", "https://github.com/jely2002/youtube-dl-gui", "AGPL-3.0-only"],
  ["solidtime", "https://github.com/solidtime-io/solidtime", "AGPL-3.0-only"],
  ["crater", "https://github.com/crater-invoice-inc/crater", "AGPL-3.0-only"],
  ["vue-native-core", "https://github.com/GeekyAnts/vue-native-core", "MIT"],
  ["muse-ui", "https://github.com/museui/muse-ui", "MIT"],
  ["inertia", "https://github.com/inertiajs/inertia", "MIT"],
  ["gui-for-singbox", "https://github.com/GUI-for-Cores/GUI.for.SingBox", "GPL-3.0-only"],
  ["vue-fabric-editor", "https://github.com/ikuaitu/vue-fabric-editor", "MIT"],
  ["vue-grid-layout", "https://github.com/jbaysolutions/vue-grid-layout", "MIT"],
  ["splayer", "https://github.com/SPlayer-Dev/SPlayer", "AGPL-3.0-only"],
  ["vuelidate", "https://github.com/vuelidate/vuelidate", "MIT"],
  ["vuetorrent", "https://github.com/VueTorrent/VueTorrent", "GPL-3.0-only"],
  ["vue-multiselect", "https://github.com/shentao/vue-multiselect", "MIT"],
  ["frpc-desktop", "https://github.com/luckjiawei/frpc-desktop", "MIT"],
  ["v-charts", "https://github.com/ElemeFE/v-charts", "MIT"],
  ["music-website", "https://github.com/Yin-Hongwei/music-website", "CC-BY-NC-4.0"],
  ["vue-flow", "https://github.com/bcakmakoglu/vue-flow", "MIT"],
  ["mavon-editor", "https://github.com/hinesboy/mavonEditor", "MIT"],
  ["nutui", "https://github.com/jd-opensource/nutui", "MIT"],
  ["nativescript-vue", "https://github.com/nativescript-vue/nativescript-vue", "MIT"],
  [
    "sigma-file-manager",
    "https://github.com/aleksey-hoffman/sigma-file-manager",
    "GPL-3.0-or-later",
  ],
  ["gogocode", "https://github.com/thx/gogocode", "MIT"],
  ["vue-chartjs", "https://github.com/apertureless/vue-chartjs", "MIT"],
  ["vuesax", "https://github.com/lusaxweb/vuesax", "MIT"],
  ["cssgridgenerator", "https://github.com/sdras/cssgridgenerator", "MIT"],
  ["varlet", "https://github.com/varletjs/varlet", "MIT"],
  ["vue-select", "https://github.com/sagalbot/vue-select", "MIT"],
  ["vue-cropper", "https://github.com/xyxiao001/vue-cropper", "MIT"],
  ["vue-draggable-next", "https://github.com/SortableJS/vue.draggable.next", "MIT"],
  ["vue-js-modal", "https://github.com/euvl/vue-js-modal", "MIT"],
  ["vue-bits", "https://github.com/DavidHDev/vue-bits", "MIT AND LicenseRef-Commons-Clause-1.0"],
  ["vue-netcore", "https://github.com/cq-panda/Vue.NetCore", "MIT"],
  ["vue-draggable-plus", "https://github.com/Alfred-Skyblue/vue-draggable-plus", "MIT"],
  ["tdesign", "https://github.com/Tencent/tdesign", "MIT"],
  ["epic-spinners", "https://github.com/epicmaxco/epic-spinners", "MIT"],
  ["portal-vue", "https://github.com/LinusBorg/portal-vue", "MIT"],
  ["vuestic-ui", "https://github.com/epicmaxco/vuestic-ui", "MIT"],
  ["piclist", "https://github.com/Kuingsmile/PicList", "MIT"],
  ["vue-draggable-resizable", "https://github.com/mauricius/vue-draggable-resizable", "MIT"],
  ["pinry", "https://github.com/pinry/pinry", "BSD-2-Clause"],
  ["vonic", "https://github.com/wangdahoo/vonic", "MIT"],
  ["laravel-breeze", "https://github.com/laravel/breeze", "MIT"],
  ["frappe-crm", "https://github.com/frappe/crm", "AGPL-3.0-only"],
  ["v-viewer", "https://github.com/mirari/v-viewer", "MIT"],
  ["antares", "https://github.com/antares-sql/antares", "MIT"],
  ["heyui", "https://github.com/heyui/heyui", "MIT"],
  ["vue-data-ui", "https://github.com/graphieros/vue-data-ui", "MIT"],
  ["splitpanes", "https://github.com/antoniandre/splitpanes", "MIT"],
  ["tailwind-config-viewer", "https://github.com/rogden/tailwind-config-viewer", "MIT"],
  ["frappe-builder", "https://github.com/frappe/builder", "AGPL-3.0-only"],
  ["vue-uploader", "https://github.com/simple-uploader/vue-uploader", "MIT"],
  ["vant-demo", "https://github.com/vant-ui/vant-demo", "MIT"],
  ["vue-dropzone", "https://github.com/rowanwins/vue-dropzone", "MIT"],
  ["multiple-select", "https://github.com/wenzhixin/multiple-select", "MIT"],
  ["alexandrie", "https://github.com/Smaug6739/Alexandrie", "MIT"],
  ["vue-fullpage-js", "https://github.com/alvarotrigo/vue-fullpage.js", "GPL-3.0-only"],
  ["arco-design-pro-vue", "https://github.com/arco-design/arco-design-pro-vue", "MIT"],
  ["vue-datepicker", "https://github.com/Vuepic/vue-datepicker", "MIT"],
  ["layoutit-grid", "https://github.com/layoutit/layoutit-grid", "MIT"],
  ["jellyfin-vue", "https://github.com/jellyfin/jellyfin-vue", "GPL-3.0-only"],
  ["lew-ui", "https://github.com/lewkamtao/lew-ui", "MIT"],
  ["vue-sonner", "https://github.com/xiaoluoboding/vue-sonner", "MIT"],
  ["element-plus-x", "https://github.com/element-plus-x/Element-Plus-X", "MIT"],
  ["vue-vine", "https://github.com/vue-vine/vue-vine", "MIT"],
  ["vue-calendar", "https://github.com/jinzhe/vue-calendar", "MIT"],
  ["prevue", "https://github.com/open-source-labs/PreVue", "MIT"],
  ["bym-vue-echarts", "https://github.com/bym110/vue-echarts", "NONE"],
  [
    "vue-apexcharts",
    "https://github.com/apexcharts/vue-apexcharts",
    "LicenseRef-ApexCharts-Community-or-Commercial",
  ],
  ["vue-lottie", "https://github.com/chenqingspring/vue-lottie", "MIT"],
  ["vue-cal-v4", "https://github.com/antoniandre/vue-cal-v4", "MIT"],
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
