---
title: Docs Blog
description: A documentação do Vize agora pode hospedar tanto as notas de lançamento quanto as irregulares.
---

<!-- Generated translation; source: blog/releases/2026-03-26-docs-blog-support.md -->

# Docs Blog

<div class="blog-post-meta">
<span class="blog-meta-chip">
<span>
<span class="blog-meta-label">Publicado em</span>
<span class="blog-meta-value">26-03-2026</span>
</span>
</span>
<a class="blog-author-card" href="https://github.com/ubugeeei">
<img src="https://github.com/ubugeeei.png" alt="ubugeeei" />
<span class="blog-author-text">
<span class="blog-meta-label">Autor</span>
<span class="blog-meta-value">ubugeeei</span>
</span>
</a>
</div>

A documentação do Vize agora pode hospedar dois tipos de posts diretamente dentro de `docs/content/blog/`:

- `releases/` para mudanças enviadas e comunicação de lançamento
- `notes/` para escrita irregular, como devlogs, redações de arquitetura e atualizações de projetos

## O que mudou

- Adicionei uma seção de **blog** de nível superior à documentação.
- Divida o fluxo de composição em **Notas de Lançamento** e **Notas de** Lançamento.
- Adicionei modelos para iniciantes para que futuras postagens sejam fáceis de criar e manter a consistência.

## Por que isso importa

O Vize já cresceu para mais do que um simples README de pacotes. Algumas atualizações pertencem aos documentos de referência, mas outras precisam de um espaço para contexto narrativo: o que foi lançado, por que isso importa, o que ainda está em fase experimental e para onde o projeto está caminhando.

Essa nova estrutura de blog cria esse espaço sem introduzir um site separado ou um segundo fluxo de trabalho de publicação.

## Onde Escrever

- Publicações de lançamento: `docs/content/blog/releases/`
- Postagens irregulares: `docs/content/blog/notes/`
- Templates: `docs/templates/blog-release.md` e `docs/templates/blog-note.md`
