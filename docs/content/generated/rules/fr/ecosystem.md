---
title: "Règles de l’écosystème"
---

# Règles de l’écosystème

Chaque règle présente sur cette page son objectif, sa configuration et ses exemples complets Mauvais et Bon, accompagnés de leurs explications. Les lignes surlignées montrent les modifications ; le code copié conserve la source complète. Les limites de prise en charge actuelles sont précisées avant les exemples concernés.


| Règle | Exemples | Objectif |
| --- | --- | --- |
| [`ecosystem/nuxt-prefer-nuxt-link`](#ecosystem-nuxt-prefer-nuxt-link) | [Mauvais](#ecosystem-nuxt-prefer-nuxt-link-bad) · [Bon](#ecosystem-nuxt-prefer-nuxt-link-good) | Préférer NuxtLink pour les liens internes à l’application |
| [`ecosystem/pinia-prefer-store-to-refs`](#ecosystem-pinia-prefer-store-to-refs) | [Mauvais](#ecosystem-pinia-prefer-store-to-refs-bad) · [Bon](#ecosystem-pinia-prefer-store-to-refs-good) | Préférer storeToRefs() lors de la déstructuration des stores Pinia |
| [`ecosystem/router-link-require-to`](#ecosystem-router-link-require-to) | [Mauvais](#ecosystem-router-link-require-to-bad) · [Bon](#ecosystem-router-link-require-to-good) | Exiger une destination `to` sur les composants RouterLink et NuxtLink |
| [`ecosystem/void-link-require-href`](#ecosystem-void-link-require-href) | [Mauvais](#ecosystem-void-link-require-href-bad) · [Bon](#ecosystem-void-link-require-href-good) | Exiger `href` sur les composants Link de Void Vue |
| [`ecosystem/void-link-valid-method`](#ecosystem-void-link-valid-method) | [Mauvais](#ecosystem-void-link-valid-method-bad) · [Bon](#ecosystem-void-link-valid-method-good) | Valider les props method statiques des composants Link de Void Vue |
| [`ecosystem/vue-i18n-no-missing-key`](#ecosystem-vue-i18n-no-missing-key) | [Mauvais](#ecosystem-vue-i18n-no-missing-key-bad) · [Bon](#ecosystem-vue-i18n-no-missing-key-good) | Signaler les clés vue-i18n statiques absentes des messages locaux du SFC |
| [`ecosystem/vue-router-prefer-named-link`](#ecosystem-vue-router-prefer-named-link) | [Mauvais](#ecosystem-vue-router-prefer-named-link-bad) · [Bon](#ecosystem-vue-router-prefer-named-link-good) | Préférer les objets de routes nommées aux chaînes de chemin statiques dans RouterLink |
| [`ecosystem/vue-router-prefer-named-push`](#ecosystem-vue-router-prefer-named-push) | [Mauvais](#ecosystem-vue-router-prefer-named-push-bad) · [Bon](#ecosystem-vue-router-prefer-named-push-good) | Préférer les objets de routes nommées pour la navigation programmatique de Vue Router |
| [`ecosystem/vue-test-utils-no-html-snapshot`](#ecosystem-vue-test-utils-no-html-snapshot) | [Mauvais](#ecosystem-vue-test-utils-no-html-snapshot-bad) · [Bon](#ecosystem-vue-test-utils-no-html-snapshot-good) | Éviter les instantanés de wrapper.html() dans les tests Vue Test Utils |
| [`nuxt/no-nuxt-config-test-key`](#nuxt-no-nuxt-config-test-key) | [Mauvais](#nuxt-no-nuxt-config-test-key-bad) · [Bon](#nuxt-no-nuxt-config-test-key-good) | Interdire la clé `test` dans la configuration Nuxt |
| [`nuxt/no-page-meta-runtime-values`](#nuxt-no-page-meta-runtime-values) | [Mauvais](#nuxt-no-page-meta-runtime-values-bad) · [Bon](#nuxt-no-page-meta-runtime-values-good) | Interdire les valeurs du contexte d’exécution évaluées immédiatement dans `definePageMeta`, extrait dans un chunk séparé à la compilation et exécuté avant le setup du composant |
| [`nuxt/nuxt-config-keys-order`](#nuxt-nuxt-config-keys-order) | [Mauvais](#nuxt-nuxt-config-keys-order-bad) · [Bon](#nuxt-nuxt-config-keys-order-good) | Préférer l’ordre recommandé des propriétés de configuration Nuxt |
| [`nuxt/prefer-import-meta`](#nuxt-prefer-import-meta) | [Mauvais](#nuxt-prefer-import-meta-bad) · [Bon](#nuxt-prefer-import-meta-good) | Préférer `import.meta.*` à `process.*` |

[Toutes les règles](./all.md) · [Options des règles](/rules/options.md) · [Correspondance de migration ESLint](/rules/migration.md) · [Vérifications du projet](./cross-file.md) · [Attributs entre composants](/rules/project/vue-cross-file-attrs-fallthrough.md)

### `ecosystem/nuxt-prefer-nuxt-link`

Préférer NuxtLink pour les liens internes à l’application

[Mauvais](#ecosystem-nuxt-prefer-nuxt-link-bad) · [Bon](#ecosystem-nuxt-prefer-nuxt-link-good)

Gravité par défaut: `warning`  
Préréglages: `nuxt`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ecosystem/nuxt-prefer-nuxt-link": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-nuxt-prefer-nuxt-link-bad"></span>

**Mauvais**

La destination interne des paramètres utilise un lien ordinaire dans une application Nuxt.

```vue annotate="remove:2"
<template>
  <a href="/settings">Settings</a>
</template>
```

<span id="ecosystem-nuxt-prefer-nuxt-link-good"></span>

**Bon**

NuxtLink gère la même destination interne au moyen du routeur Nuxt.

```vue annotate="add:2"
<template>
  <NuxtLink to="/settings">Settings</NuxtLink>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/nuxt_prefer_nuxt_link.rs#L14) · [Toutes les règles](all.md)

### `ecosystem/pinia-prefer-store-to-refs`

Préférer storeToRefs() lors de la déstructuration des stores Pinia

[Mauvais](#ecosystem-pinia-prefer-store-to-refs-bad) · [Bon](#ecosystem-pinia-prefer-store-to-refs-good)

Gravité par défaut: `warning`  
Préréglages: `ecosystem`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ecosystem/pinia-prefer-store-to-refs": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-pinia-prefer-store-to-refs-bad"></span>

**Mauvais**

Déstructurer `name` directement depuis le store sépare la valeur de son accès réactif au store.

```vue annotate="remove:2"
<script setup lang="ts">
const { name } = useUserStore();
</script>
```

<span id="ecosystem-pinia-prefer-store-to-refs-good"></span>

**Bon**

Le store reste intact et storeToRefs crée une référence réactive pour name.

```vue annotate="add:2,3"
<script setup lang="ts">
const store = useUserStore();
const { name } = storeToRefs(store);
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/pinia_prefer_store_to_refs.rs#L20) · [Toutes les règles](all.md)

### `ecosystem/router-link-require-to`

Exiger une destination `to` sur les composants RouterLink et NuxtLink

[Mauvais](#ecosystem-router-link-require-to-bad) · [Bon](#ecosystem-router-link-require-to-good)

Gravité par défaut: `error`  
Préréglages: `ecosystem`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

Un lien constituant l’unique racine d’un SFC peut hériter de sa destination via les attributs du parent. Cet exemple utilise un lien imbriqué, dont la destination doit être explicite.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ecosystem/router-link-require-to": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-router-link-require-to-bad"></span>

**Mauvais**

Le RouterLink imbriqué n’a pas de destination `to` ; il ne peut pas compter sur la transmission automatique des attributs racines.

```vue annotate="remove:2"
<template>
<nav><RouterLink>Settings</RouterLink></nav>
</template>
```

<span id="ecosystem-router-link-require-to-good"></span>

**Bon**

`to="/settings"` fournit explicitement la destination du lien imbriqué.

```vue annotate="add:2"
<template>
<nav><RouterLink to="/settings">Settings</RouterLink></nav>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/router_link_require_to.rs#L14) · [Toutes les règles](all.md)

### `ecosystem/void-link-require-href`

Exiger `href` sur les composants Link de Void Vue

[Mauvais](#ecosystem-void-link-require-href-bad) · [Bon](#ecosystem-void-link-require-href-good)

Gravité par défaut: `error`  
Préréglages: `ecosystem`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ecosystem/void-link-require-href": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-void-link-require-href-bad"></span>

**Mauvais**

Le Link importé depuis @void/vue omet sa destination href.

```vue annotate="remove:6"
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link>Settings</Link>
</template>
```

<span id="ecosystem-void-link-require-href-good"></span>

**Bon**

Le même Link importé reçoit la destination des paramètres via href.

```vue annotate="add:6"
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link href="/settings">Settings</Link>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/void_link_require_href.rs#L13) · [Toutes les règles](all.md)

### `ecosystem/void-link-valid-method`

Valider les props method statiques des composants Link de Void Vue

[Mauvais](#ecosystem-void-link-valid-method-bad) · [Bon](#ecosystem-void-link-valid-method-good)

Gravité par défaut: `warning`  
Préréglages: `ecosystem`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ecosystem/void-link-valid-method": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-void-link-valid-method-bad"></span>

**Mauvais**

L’action DELETE demande un préchargement, alors que prefetch est destiné aux requêtes de navigation.

```vue annotate="remove:6"
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link href="/posts/1" method="DELETE" prefetch>Delete</Link>
</template>
```

<span id="ecosystem-void-link-valid-method-good"></span>

**Bon**

Supprimer prefetch conserve l’action DELETE sans précharger cette requête non GET.

```vue annotate="add:6"
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link href="/posts/1" method="DELETE">Delete</Link>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/void_link_valid_method.rs#L14) · [Toutes les règles](all.md)

### `ecosystem/vue-i18n-no-missing-key`

Signaler les clés vue-i18n statiques absentes des messages locaux du SFC

[Mauvais](#ecosystem-vue-i18n-no-missing-key-bad) · [Bon](#ecosystem-vue-i18n-no-missing-key-good)

Gravité par défaut: `warning`  
Préréglages: `ecosystem`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ecosystem/vue-i18n-no-missing-key": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-vue-i18n-no-missing-key-bad"></span>

**Mauvais**

Le template demande auth.missing, mais les messages anglais locaux ne déclarent que auth.login.

```vue annotate="remove:1"
<template>{{ $t("auth.missing") }}</template>

<i18n lang="json">
{ "en": { "auth": { "login": "Log in" } } }
</i18n>
```

<span id="ecosystem-vue-i18n-no-missing-key-good"></span>

**Bon**

Le template demande la clé auth.login qui existe dans les messages locaux.

```vue annotate="add:1"
<template>{{ $t("auth.login") }}</template>

<i18n lang="json">
{ "en": { "auth": { "login": "Log in" } } }
</i18n>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/i18n_no_missing_key.rs#L17) · [Toutes les règles](all.md)

### `ecosystem/vue-router-prefer-named-link`

Préférer les objets de routes nommées aux chaînes de chemin statiques dans RouterLink

[Mauvais](#ecosystem-vue-router-prefer-named-link-bad) · [Bon](#ecosystem-vue-router-prefer-named-link-good)

Gravité par défaut: `warning`  
Préréglages: `ecosystem`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Templates et blocs des SFC Vue, avec le contexte du script requis par la règle  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ecosystem/vue-router-prefer-named-link": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-vue-router-prefer-named-link-bad"></span>

**Mauvais**

La destination de RouterLink est un chemin littéral plutôt qu’une route nommée.

```vue annotate="remove:2"
<template>
  <RouterLink to="/settings">Settings</RouterLink>
</template>
```

<span id="ecosystem-vue-router-prefer-named-link-good"></span>

**Bon**

L’objet de route lié identifie la destination par son nom de route settings.

```vue annotate="add:2"
<template>
  <RouterLink :to="{ name: 'settings' }">Settings</RouterLink>
</template>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/vue_router_prefer_named_link.rs#L15) · [Toutes les règles](all.md)

### `ecosystem/vue-router-prefer-named-push`

Préférer les objets de routes nommées pour la navigation programmatique de Vue Router

[Mauvais](#ecosystem-vue-router-prefer-named-push-bad) · [Bon](#ecosystem-vue-router-prefer-named-push-good)

Gravité par défaut: `warning`  
Préréglages: `ecosystem`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ecosystem/vue-router-prefer-named-push": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-vue-router-prefer-named-push-bad"></span>

**Mauvais**

router.push reçoit une chaîne de chemin liée à la graphie actuelle de l’URL.

```vue annotate="remove:2"
<script setup lang="ts">
router.push("/settings");
</script>
```

<span id="ecosystem-vue-router-prefer-named-push-good"></span>

**Bon**

router.push reçoit un objet de route possédant le nom stable settings.

```vue annotate="add:2"
<script setup lang="ts">
router.push({ name: "settings" });
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/vue_router_prefer_named_push.rs#L19) · [Toutes les règles](all.md)

### `ecosystem/vue-test-utils-no-html-snapshot`

Éviter les instantanés de wrapper.html() dans les tests Vue Test Utils

[Mauvais](#ecosystem-vue-test-utils-no-html-snapshot-bad) · [Bon](#ecosystem-vue-test-utils-no-html-snapshot-good)

Gravité par défaut: `warning`  
Préréglages: `ecosystem`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ecosystem/vue-test-utils-no-html-snapshot": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-vue-test-utils-no-html-snapshot-bad"></span>

**Mauvais**

L’assertion capture l’ensemble du HTML du wrapper au lieu de vérifier le comportement attendu.

```vue annotate="remove:2"
<script setup lang="ts">
expect(wrapper.html()).toMatchSnapshot();
</script>
```

<span id="ecosystem-vue-test-utils-no-html-snapshot-good"></span>

**Bon**

L’assertion vérifie que le texte affiché contient Saved.

```vue annotate="add:2"
<script setup lang="ts">
expect(wrapper.text()).toContain("Saved");
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/vue_test_utils_no_html_snapshot.rs#L15) · [Toutes les règles](all.md)

### `nuxt/no-nuxt-config-test-key`

Interdire la clé `test` dans la configuration Nuxt

[Mauvais](#nuxt-no-nuxt-config-test-key-bad) · [Bon](#nuxt-no-nuxt-config-test-key-good)

Gravité par défaut: `error`  
Préréglages: `nuxt`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Fichiers de configuration Nuxt (nuxt.config.ts)  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "nuxt/no-nuxt-config-test-key": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="nuxt-no-nuxt-config-test-key-bad"></span>

**Mauvais**

La configuration Nuxt exportée définit la clé d’identifiant `test` sur le booléen `true`, la forme de configuration obsolète rejetée par cette règle.

`nuxt.config.ts`

```ts annotate="remove:1"
export default defineNuxtConfig({ test: true });
```

<span id="nuxt-no-nuxt-config-test-key-good"></span>

**Bon**

La configuration vide supprime cette propriété booléenne `test`. Cet exemple n’interdit pas un objet de configuration de test.

`nuxt.config.ts`

```ts annotate="add:1"
export default defineNuxtConfig({});
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_nuxt_config_test_key.rs#L16) · [Toutes les règles](all.md)

### `nuxt/no-page-meta-runtime-values`

Interdire les valeurs du contexte d’exécution évaluées immédiatement dans `definePageMeta`, extrait dans un chunk séparé à la compilation et exécuté avant le setup du composant

[Mauvais](#nuxt-no-page-meta-runtime-values-bad) · [Bon](#nuxt-no-page-meta-runtime-values-good)

Gravité par défaut: `error`  
Préréglages: `nuxt`  
Correction automatique: Aucune ; examinez la modification proposée  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "nuxt/no-page-meta-runtime-values": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="nuxt-no-page-meta-runtime-values-bad"></span>

**Mauvais**

`useRoute()` est évalué immédiatement lors de la construction de l’objet `definePageMeta`, alors que la macro déplace ces métadonnées hors du contexte d’exécution du setup.

```vue annotate="remove:2"
<script setup lang="ts">
definePageMeta({ title: useRoute() });
</script>
```

<span id="nuxt-no-page-meta-runtime-values-good"></span>

**Bon**

`validate` reçoit un callback ; son accès à `useRoute().params.id` est donc différé jusqu’à l’exécution de ce callback. La règle distingue les corps de fonctions différés des valeurs de métadonnées évaluées immédiatement.

```vue annotate="add:2"
<script setup lang="ts">
definePageMeta({ validate: () => Boolean(useRoute().params.id) });
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_page_meta_runtime_values.rs#L25) · [Toutes les règles](all.md)

### `nuxt/nuxt-config-keys-order`

Préférer l’ordre recommandé des propriétés de configuration Nuxt

[Mauvais](#nuxt-nuxt-config-keys-order-bad) · [Bon](#nuxt-nuxt-config-keys-order-good)

Gravité par défaut: `error`  
Préréglages: `nuxt`  
Correction automatique: Disponible pour les diagnostics pris en charge  
Champ d’application: Fichiers de configuration Nuxt (nuxt.config.ts)  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "nuxt/nuxt-config-keys-order": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="nuxt-nuxt-config-keys-order-bad"></span>

**Mauvais**

La configuration place `ssr` avant `modules`, inversant leur ordre dans la séquence de clés Nuxt recommandée par la règle.

`nuxt.config.ts`

```ts annotate="remove:1"
export default defineNuxtConfig({ ssr: true, modules: [] });
```

<span id="nuxt-nuxt-config-keys-order-good"></span>

**Bon**

Placer `modules` avant `ssr` préserve les deux valeurs tout en respectant l’ordre prescrit ; la correction change la disposition plutôt que le sens des options.

`nuxt.config.ts`

```ts annotate="add:1"
export default defineNuxtConfig({ modules: [], ssr: true });
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/nuxt_config_keys_order.rs#L24) · [Toutes les règles](all.md)

### `nuxt/prefer-import-meta`

Préférer `import.meta.*` à `process.*`

[Mauvais](#nuxt-prefer-import-meta-bad) · [Bon](#nuxt-prefer-import-meta-good)

Gravité par défaut: `error`  
Préréglages: `nuxt`  
Correction automatique: Disponible pour les diagnostics pris en charge  
Champ d’application: Scripts JS/TS des SFC Vue ; les exemples montrent la forme concernée de l’Options API ou de script setup  
Options: Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.

**Configuration (Vite+)**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "nuxt/prefer-import-meta": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="nuxt-prefer-import-meta-bad"></span>

**Mauvais**

`process.client` utilise un ancien indicateur d’environnement Nuxt que la règle demande de migrer vers `import.meta`.

```vue annotate="remove:2"
<script setup lang="ts">
if (process.client) console.log("browser");
</script>
```

<span id="nuxt-prefer-import-meta-good"></span>

**Bon**

`import.meta.client` conserve la branche réservée au navigateur de manière explicite avec l’indicateur d’environnement de remplacement.

```vue annotate="add:2"
<script setup lang="ts">
if (import.meta.client) console.log("browser");
</script>
```

Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.

[Implémentation](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_import_meta.rs#L18) · [Toutes les règles](all.md)
