# Pitfalls: Docracy

**Defined:** 2026-04-05

## 1. Treating documents like raw blobs

**Warning signs:** search quality degrades, metadata is inconsistent, and agents start stuffing everything into content fields.

**Prevention:** require typed fields, status values, and explicit extension metadata from the start.

**Address in:** phase 1, before adapters exist.

## 2. Letting the mirror become the source of truth

**Warning signs:** vector search results disagree with structured reads or revisions drift from the semantic index.

**Prevention:** write to the primary store first, then sync indexes from committed events.

**Address in:** phases 1-2.

## 3. Losing revision history on update

**Warning signs:** old content disappears, rollback is impossible, or update tests only check the latest state.

**Prevention:** every update creates a new revision and archives the prior one automatically.

**Address in:** phase 1.

## 4. Letting metadata grow without rules

**Warning signs:** one-off fields appear per feature, and queries become brittle or impossible to compose.

**Prevention:** define extension policies and keep metadata names indexable and documented.

**Address in:** phase 2.

## 5. Shipping interfaces before invariants

**Warning signs:** CLI/API behavior diverges, adapters duplicate business logic, or tests only cover the wrapper.

**Prevention:** finish the core engine and its tests before building any external surface.

**Address in:** phase 1, enforced through roadmap order.

## 6. Narrowing the system to software-only use

**Warning signs:** terminology and schema choices assume code repositories only.

**Prevention:** keep document types and metadata general enough for non-code workflows.

**Address in:** project-wide guardrail.
