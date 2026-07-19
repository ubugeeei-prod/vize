---
title: Testes no Mundo Real
description: O Vize entra na fase de Testes do Mundo Real — projetos reais são agora a suíte de testes, com um roteiro claro para a v1.0.0.
---

<!-- Generated translation; source: blog/notes/2026-06-07-real-world-testing.md -->

# Testes no Mundo Real

<div class="blog-post-meta">
<span class="blog-meta-chip">
<span>
<span class="blog-meta-label">Publicado</span>
<span class="blog-meta-value">2026-06-07</span>
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

<video class="blog-post-video" src="/blog/vize-real-world-testing.mp4" controls muted playsinline loop preload="metadata" aria-label="Real World Testing PV"></video>

Vize está entrando em uma nova fase.

Até agora, o desenvolvimento tem se concentrado na implementação de recursos, construção de infraestrutura,
e validação de comportamentos por meio de suítes de testes dedicadas e exemplos sintéticos.

O próximo passo é diferente.

Agora estamos ativamente procurando **projetos do mundo** real para testar o Vize.

## O Objetivo

O objetivo dessa fase é descobrir lacunas de compatibilidade, lacunas de especificação
gargalos de desempenho e casos limites que aparecem apenas em bases de código de produção.

Se você mantém uma aplicação, biblioteca, framework ou ferramenta do Vue, adoraríamos ouvir
sobre sua experiência ao usá-lo com o Vize.

Cada relatório de correção, reprodução, resultado de benchmark e busca de compatibilidade ajuda
aproximar o projeto de sua primeira versão estável.

## Ainda experimental — Correção em primeiro lugar

Vize ainda deve ser considerado experimental. Mudanças que não podem ocorrer, correções
ainda são esperadas, e o comportamento pode diferir do Vue em certos cenários.

O foco desta fase não é o desenvolvimento de recursos. O foco é a correção.
Aplicações do mundo real são a suíte de testes agora. Se você encontrar algo que precise de correção, por favor,
reporte — todo relatório ajuda a melhorar o compilador, a especificação da linguagem e
o ecossistema como um todo.

## Como Ajudar

Estamos aguardando muitos pedidos de conserto e PRs. Também estamos recrutando ativamente projetos de Vue razoavelmente
grandes para usar como bancos de teste — quanto maior e mais real a base de código, mais
valioso é o sinal. Se você mantém (ou conhece) uma aplicação, biblioteca, framework
ou ferramenta substancial do Vue, por favor, abra uma solicitação de correção ou entre em contato para que possamos rodar o Vize contra ela. Relatórios de
correção, reproduções e resultados de benchmarks são todos bem-vindos.

Veja o guia [Testing & Feedback](../../guide/testing.md) sobre como inspecionar a saída no playground
, leia os casos de teste existentes, perfil com `vize check --profile`e ofereça um projeto
como um banco de testes E2E / VRT.

## Roteiro para a v1.0.0

A fase atual é **Testes do Mundo Real**.

Uma vez que o Vize conclua com sucesso essa fase, o projeto seguirá passando por:

- v1.0.0-alpha
- v1.0.0-beta
- v1.0.0-rc
- v1.0.0

As fases alfa, beta e candidata a lançamento terão foco em estabilização, compatibilidade
ecossistema, melhorias de desempenho e garantias de manutenção de longo prazo.

O objetivo não é correr para a 1.0. O objetivo é conquistá-la.

Se você tem interesse em ajudar a moldar o futuro da Vize, agora é o melhor momento para se envolver
.
