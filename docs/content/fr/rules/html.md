---
title: "Règles HTML"
---

# Règles HTML

Chaque règle présente sur cette page son objectif, sa configuration et ses exemples complets Mauvais et Bon, accompagnés de leurs explications. Les lignes surlignées montrent les modifications ; le code copié conserve la source complète. Les limites de prise en charge actuelles sont précisées avant les exemples concernés.


| Règle | Exemples | Objectif |
| --- | --- | --- |
| [`html/deprecated-attr`](https://vizejs.dev/fr/rules/html.html#html-deprecated-attr) | [Mauvais](https://vizejs.dev/fr/rules/html.html#html-deprecated-attr-bad) · [Bon](https://vizejs.dev/fr/rules/html.html#html-deprecated-attr-good) | Interdire les attributs HTML obsolètes |
| [`html/deprecated-element`](https://vizejs.dev/fr/rules/html.html#html-deprecated-element) | [Mauvais](https://vizejs.dev/fr/rules/html.html#html-deprecated-element-bad) · [Bon](https://vizejs.dev/fr/rules/html.html#html-deprecated-element-good) | Interdire les éléments HTML obsolètes |
| [`html/id-duplication`](https://vizejs.dev/fr/rules/html.html#html-id-duplication) | [Mauvais](https://vizejs.dev/fr/rules/html.html#html-id-duplication-bad) · [Bon](https://vizejs.dev/fr/rules/html.html#html-id-duplication-good) | Interdire les identifiants d’éléments en double |
| [`html/no-consecutive-br`](https://vizejs.dev/fr/rules/html.html#html-no-consecutive-br) | [Mauvais](https://vizejs.dev/fr/rules/html.html#html-no-consecutive-br-bad) · [Bon](https://vizejs.dev/fr/rules/html.html#html-no-consecutive-br-good) | Interdire les éléments &lt;br&gt; consécutifs |
| [`html/no-dupe-style-properties`](https://vizejs.dev/fr/rules/html.html#html-no-dupe-style-properties) | [Mauvais](https://vizejs.dev/fr/rules/html.html#html-no-dupe-style-properties-bad) · [Bon](https://vizejs.dev/fr/rules/html.html#html-no-dupe-style-properties-good) | Interdire les propriétés en double dans les attributs de style en ligne |
| [`html/no-duplicate-class`](https://vizejs.dev/fr/rules/html.html#html-no-duplicate-class) | [Mauvais](https://vizejs.dev/fr/rules/html.html#html-no-duplicate-class-bad) · [Bon](https://vizejs.dev/fr/rules/html.html#html-no-duplicate-class-good) | Interdire les noms de classes en double dans un attribut class statique |
| [`html/no-duplicate-dt`](https://vizejs.dev/fr/rules/html.html#html-no-duplicate-dt) | [Mauvais](https://vizejs.dev/fr/rules/html.html#html-no-duplicate-dt-bad) · [Bon](https://vizejs.dev/fr/rules/html.html#html-no-duplicate-dt-good) | Interdire les noms &lt;dt&gt; en double dans &lt;dl&gt; |
| [`html/no-empty-palpable-content`](https://vizejs.dev/fr/rules/html.html#html-no-empty-palpable-content) | [Mauvais](https://vizejs.dev/fr/rules/html.html#html-no-empty-palpable-content-bad) · [Bon](https://vizejs.dev/fr/rules/html.html#html-no-empty-palpable-content-good) | Interdire les éléments vides qui attendent un contenu visible |
| [`html/require-datetime`](https://vizejs.dev/fr/rules/html.html#html-require-datetime) | [Mauvais](https://vizejs.dev/fr/rules/html.html#html-require-datetime-bad) · [Bon](https://vizejs.dev/fr/rules/html.html#html-require-datetime-good) | Exiger l’attribut datetime sur l’élément &lt;time&gt; |

[Toutes les règles](./all.md) · [Options des règles](/rules/options.md) · [Correspondance de migration ESLint](/rules/migration.md) · [Vérifications du projet](./cross-file.md) · [Attributs entre composants](/rules/project/vue-cross-file-attrs-fallthrough.md)
