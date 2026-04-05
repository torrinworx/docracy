---
created: 2026-04-06T03:39:38.306Z
title: Scope Docracy by workspace
area: planning
files:
  - .planning/STATE.md
---

## Problem

Docracy currently feels too tied to the current repository and local machine setup. The longer-term shape is unclear: it may need to scope sessions, documents, and other context to a specific working directory, or introduce an explicit Docracy/group selection model so multiple repositories and multiple clients can coexist without trampling each other's state.

## Solution

Define the canonical scope model for Docracy before the server and multi-client work expands further. Decide whether the unit of isolation is the repo, working directory, or a named Docracy/group, and then align session storage, document visibility, and deployment assumptions around that model.
