---
created: 2026-04-05T23:35:54.193Z
title: Add specialty init context
area: planning
files:
  - README.md
  - crates/core/src/service.rs
  - crates/core/src/governance.rs
---

## Problem

`Init` currently loads the repo governance bundle plus all active `context` documents. There is an open idea about whether agents should also be able to request a more specialized init context, or a task-specific seed context, without turning the whole system into an opinionated workflow engine.

## Solution

TBD. Explore a lightweight, interface-agnostic way to support specialty context selection or init-scoped context hints while keeping the general document store model intact.
