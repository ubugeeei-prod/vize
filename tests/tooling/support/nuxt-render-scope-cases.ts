export type CorpusCase = {
  id: string;
  role: string;
  files: Record<string, string>;
  expectedDuplicateIds: string[];
};
export type Spec = CorpusCase & {
  exclusive?: boolean;
  scopeExpected?: boolean;
  patterns?: string[];
  setup?:
    | "ignored"
    | "project-rc"
    | "home-rc"
    | "ancestor-rc"
    | "xdg-rc"
    | "env"
    | "node-env"
    | "decoy-kit"
    | "nested-kit"
    | "nested-dist-kit"
    | "nested-dist-supported";
};

export function scopeCases(corpus: {
  originalFiles: Record<string, string>;
  cases: CorpusCase[];
}): Spec[] {
  const overview =
    "<script setup>import Login from './login.vue';import Signup from './signup.vue'</script><template><Login/><Signup/></template>\n";
  const originals = corpus.originalFiles;
  function changed(id: string, files: Record<string, string>, setup?: Spec["setup"]): Spec {
    return {
      id,
      role: "unknown-or-co-render-must-retain",
      files: { ...originals, ...files },
      expectedDuplicateIds: ["shell", "auth-page"],
      setup,
    };
  }
  const specs: Spec[] = [
    ...corpus.cases.map((entry, index) => ({
      ...entry,
      exclusive: index === 0,
      scopeExpected: index === 0 || entry.id === "layout-and-page",
    })),
    changed("unselected-co-rendering-importer", { "src/pages/overview.vue": overview }),
    changed(
      "ignored-co-rendering-importer",
      { "src/pages/overview.vue": overview, ".gitignore": "src/pages/overview.vue\n" },
      "ignored",
    ),
    changed("page-transition", {
      "src/pages/login.vue":
        "<script setup>definePageMeta({layout:'bare',pageTransition:{mode:'in-out'}})</script><template><section id=\"auth-page\">login</section></template>\n",
    }),
    changed("repeated-layout-slot", {
      "src/layouts/bare.vue": '<template><div id="shell"><slot/><slot/></div></template>\n',
    }),
    changed("looped-layout-slot", {
      "src/layouts/bare.vue":
        '<template><div id="shell"><template v-for="item in [1,2]"><slot/></template></div></template>\n',
    }),
    changed("shadowed-framework-component", {
      "src/components/NuxtPage.vue": "<template><slot/><slot/></template>\n",
    }),
    changed("inline-event-carrier", {
      "src/layouts/bare.vue":
        '<template><div id="shell" onclick="document.body.appendChild(this.cloneNode(true))"><slot/></div></template>\n',
    }),
    ...(
      [
        "project-rc",
        "ancestor-rc",
        "home-rc",
        "xdg-rc",
        "env",
        "node-env",
        "nested-kit",
        "nested-dist-kit",
      ] as const
    ).map((setup) => changed(setup, {}, setup)),
    { ...changed("top-level-kit-decoy", {}, "decoy-kit"), exclusive: true },
    {
      ...changed("nested-dist-supported-genuine-kit", {}, "nested-dist-supported"),
      exclusive: true,
    },
  ];
  specs.find((entry) => entry.id === "unselected-co-rendering-importer")!.patterns = [
    "src/app.vue",
    "src/layouts/bare.vue",
    "src/layouts/default.vue",
    "src/pages/login.vue",
    "src/pages/signup.vue",
  ];
  return specs;
}
