import { Resolver } from "@parcel/plugin";
import { resolveVirtualModule } from "./virtual-module.ts";

export default new Resolver({
  resolve({ dependency }) {
    return resolveVirtualModule(dependency);
  },
});
