# Marketing Plan

## Recommendation

Position Docracy as durable, versioned document storage for agents: a Postgres-backed system for project memory, governance, and auditability.

Do not lead with "notes," "AI memory," or "vector search." Lead with "document store for agent workflows" and "revision history you can trust."

## Who It Is For

- Agent framework builders who need long-lived state
- LLM app developers who need durable project context
- Infra-minded indie hackers who want a simple Postgres-native stack
- Teams that need auditable project memory and workflow history

## Core Message

- Agents need state that survives context windows and handoffs.
- Docracy gives typed documents, immutable revisions, and repo-owned governance.
- Postgres keeps the system simple, durable, and easy to explain.

## Launch Angles

- "Versioned document memory for agents"
- "Postgres-backed durable context for LLM systems"
- "A bureaucracy store for agentic workflows"

## Channels

1. Hacker News: lead with the technical problem, architecture, and tradeoffs.
2. Reddit: post where agent and devtool builders gather, focusing on practical use cases.
3. YC / founder circles: emphasize speed, persistence, and roadmap clarity.
4. OpenCode / OpenWebUI communities: show the MCP integration story and local-first workflow.

## Launch Sequence

1. Tighten README positioning and examples around the durable document-store pitch.
2. Publish a short technical launch post with one concrete workflow and one architecture diagram.
3. Share a demo showing Init/Create/Read/Query/Update over MCP.
4. Follow up with a devlog explaining why Postgres + revision history is the right baseline.

## Credibility Rules

- Avoid hype language and vague AI claims.
- Show real workflows, schema examples, and exact tool surfaces.
- Lead with constraints, tradeoffs, and what is intentionally deferred.
- Make the Postgres-first architecture explicit.

## Short Version

Docracy should launch as a serious infrastructure tool for agent memory and document governance, with the README and launch posts centered on durability, auditability, and simple integration.
