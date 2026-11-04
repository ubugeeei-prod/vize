import { addNitroPlugin, createResolver, defineNuxtModule } from "nuxt/kit";
export default defineNuxtModule({
  setup() {
    const resolver = createResolver(import.meta.url);
    addNitroPlugin({
      nitro2: resolver.resolve("./runtime/stylesheet-observer.nitro2"),
      nitro3: resolver.resolve("./runtime/stylesheet-observer.nitro3"),
    });
  },
});
