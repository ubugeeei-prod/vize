---
title: "Règles d’accessibilité"
description: "ARIA, interactions au clavier, libellés, repères et alternatives aux médias, pour que les templates fonctionnent avec les technologies d’assistance."
---

# Règles d’accessibilité

ARIA, interactions au clavier, libellés, repères et alternatives aux médias, pour que les templates fonctionnent avec les technologies d’assistance.

Chaque page de règle montre un exemple qui échoue, un exemple qui passe et la configuration qui active la règle.

## Par défaut (25)

Ajoutées par `happy-path`, le préréglage par défaut.

| Règle | Ce qu’elle vérifie | Correction automatique |
| --- | --- | --- |
| <span id="a11y-alt-text"></span>[`a11y/alt-text`](./reference/a11y-alt-text.md) | Exiger un texte alternatif pour les éléments multimédias. |  |
| <span id="a11y-anchor-has-content"></span>[`a11y/anchor-has-content`](./reference/a11y-anchor-has-content.md) | Exiger un contenu accessible pour les éléments de lien. |  |
| <span id="a11y-anchor-is-valid"></span>[`a11y/anchor-is-valid`](./reference/a11y-anchor-is-valid.md) | Imposer un href valide sur les éléments de lien. |  |
| <span id="a11y-aria-props"></span>[`a11y/aria-props`](./reference/a11y-aria-props.md) | Interdire les attributs ARIA invalides. |  |
| <span id="a11y-aria-role"></span>[`a11y/aria-role`](./reference/a11y-aria-role.md) | Exiger un rôle ARIA valide et non abstrait pour les éléments possédant un rôle ARIA. |  |
| <span id="a11y-aria-unsupported-elements"></span>[`a11y/aria-unsupported-elements`](./reference/a11y-aria-unsupported-elements.md) | Interdire les attributs ARIA sur les éléments qui ne les prennent pas en charge. |  |
| <span id="a11y-click-events-have-key-events"></span>[`a11y/click-events-have-key-events`](./reference/a11y-click-events-have-key-events.md) | Exiger des gestionnaires de clavier avec les événements de clic. |  |
| <span id="a11y-form-control-has-label"></span>[`a11y/form-control-has-label`](./reference/a11y-form-control-has-label.md) | Exiger des libellés associés aux contrôles de formulaire. |  |
| <span id="a11y-heading-has-content"></span>[`a11y/heading-has-content`](./reference/a11y-heading-has-content.md) | Exiger un contenu accessible pour les éléments de titre. |  |
| <span id="a11y-iframe-has-title"></span>[`a11y/iframe-has-title`](./reference/a11y-iframe-has-title.md) | Exiger un attribut title sur les éléments iframe. |  |
| <span id="a11y-interactive-supports-focus"></span>[`a11y/interactive-supports-focus`](./reference/a11y-interactive-supports-focus.md) | Exiger que les éléments possédant un rôle interactif puissent recevoir le focus. |  |
| <span id="a11y-label-has-for"></span>[`a11y/label-has-for`](./reference/a11y-label-has-for.md) | Exiger des contrôles de formulaire associés aux libellés. |  |
| <span id="a11y-media-has-caption"></span>[`a11y/media-has-caption`](./reference/a11y-media-has-caption.md) | Exiger des sous-titres pour les éléments multimédias. |  |
| <span id="a11y-mouse-events-have-key-events"></span>[`a11y/mouse-events-have-key-events`](./reference/a11y-mouse-events-have-key-events.md) | Exiger des événements de focus et de perte du focus avec les événements de souris. |  |
| <span id="a11y-no-access-key"></span>[`a11y/no-access-key`](./reference/a11y-no-access-key.md) | Interdire l’utilisation de l’attribut accesskey. |  |
| <span id="a11y-no-aria-hidden-on-focusable"></span>[`a11y/no-aria-hidden-on-focusable`](./reference/a11y-no-aria-hidden-on-focusable.md) | Interdire aria-hidden="true" sur les éléments pouvant recevoir le focus. |  |
| <span id="a11y-no-autofocus"></span>[`a11y/no-autofocus`](./reference/a11y-no-autofocus.md) | Interdire l’utilisation de l’attribut autofocus. |  |
| <span id="a11y-no-distracting-elements"></span>[`a11y/no-distracting-elements`](./reference/a11y-no-distracting-elements.md) | Interdire les éléments distrayants tels que &lt;marquee&gt; et &lt;blink&gt;. |  |
| <span id="a11y-no-i-for-icon"></span>[`a11y/no-i-for-icon`](./reference/a11y-no-i-for-icon.md) | Interdire l’élément &lt;i&gt; pour les icônes. |  |
| <span id="a11y-no-redundant-roles"></span>[`a11y/no-redundant-roles`](./reference/a11y-no-redundant-roles.md) | Interdire les rôles ARIA redondants. | ✓ |
| <span id="a11y-no-refer-to-non-existent-id"></span>[`a11y/no-refer-to-non-existent-id`](./reference/a11y-no-refer-to-non-existent-id.md) | Interdire les références à des identifiants inexistants. |  |
| <span id="a11y-no-role-presentation-on-focusable"></span>[`a11y/no-role-presentation-on-focusable`](./reference/a11y-no-role-presentation-on-focusable.md) | Interdire role="presentation" ou role="none" sur les éléments pouvant recevoir le focus. |  |
| <span id="a11y-no-static-element-interactions"></span>[`a11y/no-static-element-interactions`](./reference/a11y-no-static-element-interactions.md) | Interdire les gestionnaires d’événements sur les éléments statiques. |  |
| <span id="a11y-role-has-required-aria-props"></span>[`a11y/role-has-required-aria-props`](./reference/a11y-role-has-required-aria-props.md) | Exiger les propriétés obligatoires des rôles ARIA. |  |
| <span id="a11y-tabindex-no-positive"></span>[`a11y/tabindex-no-positive`](./reference/a11y-tabindex-no-positive.md) | Interdire les valeurs positives de tabindex. |  |

## Préréglages plus stricts (5)

Seulement dans `opinionated`, `nuxt` ou `ecosystem`. La colonne Préréglages indique lesquels.

| Règle | Ce qu’elle vérifie | Préréglages | Correction automatique |
| --- | --- | --- | --- |
| <span id="a11y-heading-levels"></span>[`a11y/heading-levels`](./reference/a11y-heading-levels.md) | Interdire de sauter des niveaux de titre. | `nuxt`, `opinionated` |  |
| <span id="a11y-landmark-roles"></span>[`a11y/landmark-roles`](./reference/a11y-landmark-roles.md) | Valider l’emplacement et l’unicité des rôles de zones de repère. | `nuxt`, `opinionated` |  |
| <span id="a11y-placeholder-label-option"></span>[`a11y/placeholder-label-option`](./reference/a11y-placeholder-label-option.md) | Exiger une option d’invite dans un select required. | `nuxt`, `opinionated` |  |
| <span id="a11y-use-list"></span>[`a11y/use-list`](./reference/a11y-use-list.md) | Suggérer des éléments de liste pour les textes ressemblant à des listes à puces. | `nuxt`, `opinionated` |  |
| <span id="vue-use-unique-element-ids"></span>[`vue/use-unique-element-ids`](./reference/vue-use-unique-element-ids.md) | Imposer des identifiants d’éléments uniques avec useId() plutôt que des littéraux statiques. | `nuxt`, `opinionated` |  |

## À activer (1)

Dans aucun préréglage. Activez-les par leur nom dans `vue.linter.rules`.

| Règle | Ce qu’elle vérifie | Correction automatique |
| --- | --- | --- |
| <span id="a11y-img-alt"></span>[`a11y/img-alt`](./reference/a11y-img-alt.md) | Exiger un attribut alt sur les images pour les rendre accessibles. |  |

[Vue d’ensemble des règles](../rules/index.md) · [Options des règles](/rules/options.md) · [Migrer depuis ESLint](/rules/migration.md)
