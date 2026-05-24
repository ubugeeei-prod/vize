import type { DefineComponent } from "vue";

// vue-select v4 beta ships JavaScript without declarations in this package version.
// @ts-ignore
import RawVSelect from "vue-select";

export default RawVSelect as DefineComponent<Record<string, unknown>, {}, any>;
