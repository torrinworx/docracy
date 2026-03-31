---
created: 2026-04-06T00:06:59.114Z
title: Design vector mirror contract
area: planning
files:
  - README.md
  - .planning/PROJECT.md
---

## Problem

Vector mirroring is a future direction for Docracy, but the contract is still unclear. If a vector store mirrors document state, it should probably mirror the full durable state of documents while also deciding what parts are actually embedded. That raises open questions around non-text content and whether v1 should require vectorized content to be text-serializable.

## Solution

TBD. Define a future vector mirror contract that separates mirrored document state from embedding payloads, and decide whether early support should be limited to content that can be serialized to text.
