//! The Vue SFC container.

#![expect(clippy::todo, reason = "skeleton: #6837")]

use vize_l0::Allocator;

use super::{Container, ContainerFormat};

/// Vue single-file components: `<template>`, `<script>`, `<script setup>`,
/// `<style>` and custom blocks. Only the root `<template>` nests same-named
/// tags; every other block ends at the first matching close tag.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Vue;

impl ContainerFormat for Vue {
    fn split<'a>(&self, _allocator: &'a Allocator, _source: &'a str) -> Container<'a> {
        todo!("#6837: move SFC block splitting into vize_l1::container")
    }
}
