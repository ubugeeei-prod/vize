---
title: Serveur MCP
---

<!-- Generated translation; source: integrations/mcp.md -->

# Serveur MCP

> **⚠️ Travaux en cours :** Vize est en développement actif et n’est pas encore prêt pour une utilisation en production. Les capacités des serveurs MCP peuvent changer sans préavis.

Vize fournit un serveur [Model Context Protocol (MCP)](https://modelcontextprotocol.io/) pour les flux de travail de développement alimentés par l’IA. Le serveur MCP fait le lien entre votre galerie de composants (Musea) et les assistants IA, leur permettant de comprendre, naviguer et travailler avec vos composants Vue.

## Installation

Installez `vp` une fois depuis le [Vite+ install guide](https://viteplus.dev/guide/install), puis ajoutez le serveur à votre projet :

```bash
vp install -D @vizejs/musea-mcp-server
```

## Qu’est-ce que le MCP ?

Le protocole de contexte modèle est une norme ouverte permettant de connecter les assistants IA (comme Claude, ChatGPT et d’autres) aux outils de développement. Au lieu que des assistants IA cherchent à deviner votre base de code, MCP offre un accès structuré à de vraies données composantes — props, événements, emplacements, variantes et documentation.

Le serveur MCP de Vize expose les informations des composants provenant de la galerie Musea, donc votre assistant IA a la même compréhension de vos composants qu’un développeur naviguant dans la galerie.

## Capacités

Le serveur MCP fournit les outils suivants aux assistants IA :

### Découverte des composants

- **Listez tous les composants** — Parcourez tous les composants enregistrés avec leurs catégories, tags et statut
- **Composants de recherche** — Trouver les composants par nom, étiquette ou description
- **Obtenir les métadonnées des composants** — Récupérer des informations détaillées sur un composant spécifique

### API composante

- **Props** — Définitions complètes des props avec types, valeurs par défaut et statut requis
- **Événements** — Événements émis avec types de charges utiles
- **Machines à sous** — Machines à sous nommées avec des types de propulsion de machines à sous
- **Expose** — Méthodes et propriétés publiquement exposées

### Informations sur l’histoire

- **Liste des variantes** — Toutes les variantes définies dans les fichiers d’art
- **Source de la variante** — Code modèle pour chaque variante
- **Variante par défaut** — Quelle variante est affichée par défaut

### Jetons de conception

- **Liste des jetons** — Tous les jetons de design provenant du fichier tokens
- **Catégories de jetons** — Couleurs, typographie, espacement, points d’arrêt
- **Résolution des jetons** — Jetons sémantiques résolus à leurs valeurs primitives

## Mise en place

### Avec Claude Code

Ajoutez le serveur MCP à votre configuration de Code Claude :

```json
// .claude/settings.json
{
  "mcpServers": {
    "vize-musea": {
      "command": "vp",
      "args": ["dlx", "@vizejs/musea-mcp-server"]
    }
  }
}
```

### Avec Claude Desktop

Ajoutez à votre configuration Claude Desktop MCP :

```json
{
  "mcpServers": {
    "vize-musea": {
      "command": "vp",
      "args": ["dlx", "@vizejs/musea-mcp-server"]
    }
  }
}
```

### Avec d’autres assistants IA

N’importe quel assistant IA compatible MCP peut utiliser le serveur. Le schéma de configuration est le même — pointer l’assistant vers `vp dlx @vizejs/musea-mcp-server`.

## Cas d’utilisation

### Découverte des composants

Demandez à votre assistant IA de trouver le bon composant :

> « Quels composants de boutons avons-nous ? Montre-moi les variantes de VFButton. »

L’IA peut interroger le serveur MCP pour trouver tous les composants liés aux boutons, leurs accessoires et les variantes disponibles — puis suggérer l’utilisation correcte.

### Génération de code

Générez l’utilisation des composants avec les props appropriés :

> « Créez un formulaire avec nos composants VFInput et VFTextarea, incluant les états d’erreur de validation. »

L’IA connaît les noms exacts des props, les types et les variantes disponibles depuis le serveur MCP, générant un code précis sans halluciner les noms des prop.

### Référence API

Interrogez les API des composants de façon programmatique :

> « Quels accessoires VFNameBadgePreview accepte-t-il ? Quelles sont les valeurs valables pour le rôle utilisateur ? »

L’IA renvoie les vraies définitions de prop à partir de votre base de code, pas des suppositions génériques.

### Documentation Assistance

> « Rédigez la documentation pour notre composant SponsorGrid basée sur ses accessoires et variantes. »

L’IA peut générer une documentation précise en inspectant les métadonnées réelles des composants via MCP.

## Comment ça fonctionne

```
AI Assistant
  ↕ MCP Protocol (JSON-RPC over stdio)
@vizejs/musea-mcp-server
  ↕ Reads art files and component sources
Your Project (*.art.vue files + components)
```

Le serveur MCP :

1. Découvre tous les fichiers `*.art.vue` de votre projet
2. Les analyse en utilisant `vize_musea` pour extraire les métadonnées des composants
3. Expose les métadonnées via des outils MCP
4. Répond en temps réel aux requêtes de l’assistant IA
