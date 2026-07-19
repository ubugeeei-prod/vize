---
title: Docs Blog
description: La documentation Vize peut désormais héberger à la fois des notes de publication et des notes irrégulières.
---

<!-- Generated translation; source: blog/releases/2026-03-26-docs-blog-support.md -->

# Docs Blog

<div class="blog-post-meta">
<span class="blog-meta-chip">
<span>
<span class="blog-meta-label">Publié le</span>
<span class="blog-meta-value">26-03-2026</span>
</span>
</span>
<a class="blog-author-card" href="https://github.com/ubugeeei">
<img src="https://github.com/ubugeeei.png" alt="ubugeeei" />
<span class="blog-author-text">
<span class="blog-meta-label">Auteur</span>
<span class="blog-meta-value">ubugeeei</span>
</span>
</a>
</div>

La documentation Vize peut désormais héberger deux types de publications directement à l’intérieur de `docs/content/blog/`:

- `releases/` pour les modifications expédiées et la communication sur la sortie
- `notes/` pour l’écriture irrégulière telle que les devlogs, les writeups d’architecture et les mises à jour de projets

## Ce qui a changé

- Ajout d’une section **de blog** de premier niveau à la documentation.
- Divisez le flux d’écriture en **notes de sortie** et **notes**.
- Ajout de modèles de démarrage pour que les futurs articles soient faciles à créer et à garder cohérents.

## Pourquoi cela est important

Vize est déjà devenu bien plus qu’un simple package README. Certaines mises à jour appartiennent aux documents de référence, mais d’autres ont besoin d’un espace pour le contexte narratif : ce qui a été lancé, pourquoi cela est important, ce qui est encore expérimental, et où se dirige le projet.

Cette nouvelle structure de blog crée cet espace sans introduire un site séparé ni un second flux de publication.

## Où écrire

- Publications de publication : `docs/content/blog/releases/`
- Publications irrégulières : `docs/content/blog/notes/`
- Modèles : `docs/templates/blog-release.md` et `docs/templates/blog-note.md`
