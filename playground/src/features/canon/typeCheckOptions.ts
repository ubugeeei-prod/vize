import type { Ref } from "vue";
import type { WasmModule } from "../../wasm/index";
import type { ExperimentalOptions } from "../../shared/experimentalFeatures";

export interface UseMonacoTypeCheckOptions {
  source: Ref<string>;
  compiler: () => WasmModule | null;
  strictMode: Ref<boolean>;
  includeVirtualTs: Ref<boolean>;
  checkProps: Ref<boolean>;
  checkEmits: Ref<boolean>;
  checkTemplateBindings: Ref<boolean>;
  useMonacoTs: Ref<boolean>;
  experimentals: Ref<ExperimentalOptions>;
}
