# Repeated attributes under the Vue 2 dialect

`App.vue.txt` reduces `src/components/common/shoplist.vue` from the
vue2-elm Real Project Matrix fixture: `class="" class="shop_title ellipsis"`.

`vue27-render.txt` is `@vue/compiler-sfc@2.7.16` output. Vue 2 accepts the
repeat and keeps the last value (`staticClass: "shop_title ellipsis"`), because
its attrs map is overwritten. Vue 3 reports "Duplicate attribute." as an error.

With `compiler.compatibility.vueVersion` set to `"2"` or `"2.7"`, Vize keeps the
last value and reports a warning. The Vue 3 dialect still rejects the template.
