---
title: Inspecteur de compilateur
---

<!-- Generated translation; source: guide/compiler-inspector.md -->

# Inspecteur de compilateur

L’inspecteur de terrain de jeu collecte le compilateur et les surfaces d’analyse nécessaires pour examiner une `.vue`
reproduction. Il affiche la sortie officielle du compilateur SFC de Vue, la sortie du compilateur Vize, Virtual TS, VIR, ainsi qu’un graphe
cross-file pour les lots locaux.

Ouvrez l’inspecteur depuis l’aire de jeux :

```bash
https://vizejs.dev/play/?tab=inspector
```

L’inspecteur effectue ces vérifications dans le navigateur :

- `@vue/compiler-sfc` pour la sortie de référence
- Vize WASM pour la sortie Vize
- Virtual TS appuyé par Canon pour le fichier sélectionné
- Croquis VIR pour le fichier sélectionné
- Graphes `vize_curator` natifs et métadonnées diff partagées avec la CLI
- Diagnostics croisés pour les fichiers payload
- Sélection de cible DOM ou SSR
- Contrôles optionnels de mode de rendu personnalisé et de mode syntaxe de modèle
- Onglets de sortie complète pour les deux compilateurs
- Un onglet comparatif avec des lignes uniquement Vue et uniquement Vize
- Un lien permalien et un lien de pull request prérempli

## Charges utiles CLI

Utilisez `vize inspector` lorsque la reproduction existe déjà dans un projet local. Un seul fichier produit par défaut une URL
playground :

```bash
vize inspector src/App.vue
```

Les répertoires et globs créent des charges utiles batch. Le terrain de jeu ouvre le lot et permet de
passer d’un fichier à l’autre.

```bash
vize inspector src/components
vize inspector "src/**/*.vue" --target ssr
```

Pour les grands lots, émettez du JSON au lieu d’une URL longue :

```bash
vize inspector "src/**/*.vue" --format json --output inspector-payload.json
```

Pour les agents IA ou le transfert de terminal, émettez le rapport d’agent. Il inclut la charge utile, l’URL du terrain de jeu, les métriques de résumé
et les métadonnées des graphes croisés aux fichiers.

```bash
vize inspector "src/**/*.vue" --format agent --output inspector-agent.json
```

Lors d’une vérification de développement locale, la CLI peut également exécuter directement la comparaison du compilateur. Cela utilise
le compilateur Rust dans le binaire courant et charge `@vue/compiler-sfc` depuis le projet en cours ou
l’espace de travail Vize `node_modules`.

```bash
vize inspector "src/**/*.vue" --format compare --output inspector-compare.json
```

La charge utile et le rapport agent sont générés par `vize_curator`, la même caisse Rust locale uniquement utilisée
par les liaisons WASM du playground pour les métadonnées des diff de graphes et de lignes. Cela permet de maintenir les rapports batch CLI et
'inspection du navigateur alignés tout en laissant le compilateur officiel Vue fonctionner dans le navigateur.

Options utiles :

| Option              | Description                                                   |
| ------------------- | ------------------------------------------------------------- |
| `--target dom`      | Comparer la sortie du compilateur VDOM                        |
| `--target ssr`      | Comparer la sortie du compilateur SSR                         |
| `--format agent`    | Émettre du JSON lisible par agent avec métadonnées de graphe  |
| `--format compare`  | Lancer une comparaison de CLI uniquement développeur avec Vue |
| `--custom-renderer` | Activez le mode de rendu personnalisé dans le terrain de jeu  |
| `--template-syntax` | Choisissez `standard`, `strict`ou `quirks`                    |
| `--max-files <n>`   | Limiter le nombre de fichiers dans une charge utile batch     |
| `--playground-url`  | Écraser l’URL de playground utilisée pour les liens           |

## Flux de travail RP

Lors de l’ouverture d’une PR de parité du compilateur, incluez le permalien de l’inspecteur dans le corps de la PR et ajoutez le
accessoire minimal ou instantané complet qui rend la modification de sortie révisable dans CI. Le lien PR
prérempli est un point de départ ; après avoir poussé votre branche, remplacez la tête de comparaison si GitHub le demande.

Les preuves utiles en relations publiques sont les suivantes :

- Le permalien de l’inspecteur
- La cible choisie et les options
- Le luminaire `.vue` minimisé ou la capture complète
- Contexte virtuel pertinent de TS, VIR ou graphe lorsque la correction traverse les surfaces du compilateur
- La raison pour laquelle la sortie Vize devrait correspondre ou différer intentionnellement de Vue
- La commande de vérification locale qui couvre la surface du compilateur touchée
