// Exact reporter fences 0..2; commands/displayed output are not project files.
export const originalFiles = [
  {
    path: "src/shared.ts",
    source:
      "import { defineComponent } from 'vue'\n\nexport const shared = defineComponent({\n  data() {\n    return { currentDevice: 'pc' }\n  },\n  computed: {\n    isPC(): boolean {\n      return this.currentDevice === 'pc'\n    },\n  },\n  methods: {\n    greet(name: string) {\n      return `hi ${name}`\n    },\n  },\n})\n",
    sha256: "4682d621fbfdfd9ceb1d93e422ab6d83b54a927f4e3c0a2621724947afbccfe3",
  },
  {
    path: "src/base.ts",
    source:
      "import { defineComponent } from 'vue'\n\nexport const base = defineComponent({\n  computed: {\n    fromBase(): number {\n      return 1\n    },\n  },\n})\n",
    sha256: "5f5d1cad14e80f58224306173db52efbebd4c12498d753c80ea328a3f9ad2f9b",
  },
  {
    path: "src/Invalid.vue",
    source:
      "<template>\n  <p :title=\"greet(1)\">{{ isPC.toFixed() }} {{ currentDevice.toFixed() }} {{ fromBase.toUpperCase() }} {{ missing }}</p>\n</template>\n\n<script lang=\"ts\">\nimport { defineComponent } from 'vue'\nimport { base } from './base'\nimport { shared } from './shared'\n\nexport default defineComponent({\n  extends: base,\n  mixins: [shared],\n})\n</script>\n",
    sha256: "c0094aa36df53290e783e5b00d37dd1c3a04a92ced5dce7295a64e43ff5e3f1e",
  },
] as const;
export const wrappedSupplement =
  "<template>\n  <p :title=\"greet('x')\">{{ isPC.toFixed() }} {{ count.toFixed() }} {{ fromBase.toFixed() }}</p>\n</template>\n\n<script lang=\"ts\">\nimport { defineComponent } from 'vue'\nimport { base } from './base'\nimport { shared } from './shared'\n\nexport default defineComponent({\n  extends: (base as typeof base),\n  mixins: [(shared)!],\n  data() {\n    return { count: 1 }\n  },\n})\n</script>\n";
