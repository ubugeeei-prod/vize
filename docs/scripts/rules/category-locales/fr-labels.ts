export const frenchCategoryTitles = {
  all: "Toutes les règles Patina",
  "type-and-script": "Règles de type et de script",
  html: "Règles HTML",
  accessibility: "Règles d’accessibilité",
  ssr: "Règles SSR",
  "petite-vue": "Règles petite-vue",
  vapor: "Règles Vapor",
  ecosystem: "Règles de l’écosystème",
  "musea-and-css": "Règles Musea et CSS",
  "cross-file": "Règles inter-fichiers",
} satisfies Record<string, string>;

export const frenchCategoryIntro =
  "Chaque règle présente sur cette page son objectif, sa configuration et ses exemples complets Mauvais et Bon, accompagnés de leurs explications. Les lignes surlignées montrent les modifications ; le code copié conserve la source complète. Les limites de prise en charge actuelles sont précisées avant les exemples concernés.";

export const frenchCrossFileIntro =
  "Les 66 exemples de projets réunissent ici les fichiers communs et les exemples complets Mauvais et Bon. Utilisez les fichiers communs des deux côtés. Les 60 codes d’analyse conservent leurs limites réelles : 19 codes CLI (18 paires de sources qualifiées et un projet illustratif avec son graphe réactif conservé), 16 codes de l’analyseur Rust expérimental que cette passe CLI n’émet pas individuellement, et 25 contrats sans producteur actuel. Configurer un identifiant n’active pas un producteur indisponible.";
