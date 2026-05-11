---
id: support-design-document-type
level: initiative
title: "Support Design Document Type"
short_code: "METIS-I-0030"
created_at: 2026-05-08T10:03:02.932445+00:00
updated_at: 2026-05-08T10:16:09.571578+00:00
parent: METIS-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/active"


exit_criteria_met: false
estimated_complexity: M
initiative_id: support-design-document-type
---

# Support Design Document Type Initiative

## Context **[REQUIRED]**

Metis currently supports five document types: vision, initiative, task, adr, and specification. UI/UX design work has no first-class home — it ends up wedged into specifications or initiatives, both of which carry the wrong lifecycle assumptions. Specifications are living system docs; initiatives are project-scoped commitments. Neither matches how UI design actually flows: explore many ideas, narrow to a candidate, get it approved, then implement (often via tasks under unrelated initiatives). Some designs are scratched outright and never reach implementation.

This initiative adds a `design` document type (shortcode `D`) so a project can capture 100% of its UI designs as peer-of-initiative artifacts under the vision, regardless of whether implementation work has been planned yet. Initiatives and tasks reference designs by short code; the relationship is informational, not enforced.

## Goals & Non-Goals **[REQUIRED]**

**Goals:**
- New `design` document type with shortcode `D` (codes formatted `PREFIX-D-NNNN`).
- Parent must be a published Vision. Designs are top-level peers of initiatives.
- Phase sequence: `discovery → review → approved`.
- Designs are creatable without any initiative existing (supports the "100% of UI designs" goal).
- Enabled by default in both `streamlined` and `direct` presets.
- Scratched designs go through the existing `archive_document` flow; no special phase.
- Initiatives and tasks can reference designs by short code in their content (manual links, no validation).

**Non-Goals:**
- Replacing or deprecating `specification` — specs remain for system/feature specs.
- Storing binary assets (images, .pen files) inside the design document. Designs link out to `.pen` files, screenshots, Figma URLs, etc.
- Enforcing bi-directional links between designs and initiatives/tasks.
- A multi-reviewer approval workflow beyond the `review → approved` phase transition.
- Migrating existing specification or initiative content to the new type.

## Requirements

### Functional Requirements
- REQ-001: `DocumentType::Design` variant with shortcode letter `D`; short codes follow `PREFIX-D-NNNN`.
- REQ-002: Phase sequence `discovery → review → approved`. Valid transitions: `discovery → review`, `review → approved`, `review → discovery` (kick back). Approved is terminal (besides archive).
- REQ-003: Parent must be a Vision document in `published` phase. Validation matches the existing initiative parent rule.
- REQ-004: Enabled in both `streamlined` and `direct` presets — `enabled_document_types()` always includes `Design`.
- REQ-005: Template files under `crates/metis-docs-core/src/templates/design/` with sections suited to UI work: problem framing, target user, mockup/asset links, user flow, design system notes, alternatives explored, open questions.
- REQ-006: Documents are stored under `.metis/designs/<id>/design.md`, mirroring the existing per-type directory layout.
- REQ-007: MCP `create_document` accepts `document_type="design"` with required `parent_id` pointing to a Vision short code.
- REQ-008: `archive_document`, `read_document`, `edit_document`, `transition_phase`, `list_documents`, and `search_documents` work for the new type without per-type changes (or with minimal type-list updates).

### Non-Functional Requirements
- NFR-001: Adding the type does not require a database schema migration if the existing schema treats `document_type` as a string. If an enum/check constraint exists, ship the migration in this initiative.
- NFR-002: All existing presets (`streamlined`, `direct`) continue to work unchanged for projects that don't use the new type.

## Detailed Design

### Touchpoints (verified against current code)
1. `crates/metis-docs-core/src/domain/documents/types.rs:153` — add `Design` variant to `DocumentType`.
2. `crates/metis-docs-core/src/domain/documents/types.rs:192-268` — extend `phase_sequence()` and `valid_transitions_from()` for Design.
3. `crates/metis-docs-core/src/dal/database/configuration_repository.rs:163-177` — map `"design" → "D"` in shortcode generation.
4. `crates/metis-docs-core/src/domain/configuration.rs:43-89` — allow `Design` in both presets; `get_parent_type(Design) → Some(Vision)`.
5. `crates/metis-docs-core/src/templates/design/{frontmatter.yaml,content.md}` — new template files.
6. `crates/metis-docs-core/src/domain/documents/design/mod.rs` — new module mirroring `specification/mod.rs` for template loading and type-specific behavior.
7. `crates/metis-docs-core/src/application/services/document/creation.rs:170-220` — branch for `Design`: require Vision parent, ensure `.metis/designs/` exists, write to `designs/<id>/design.md`.
8. `crates/metis-docs-mcp/src/tools/create_document.rs:130-220` — handle `document_type="design"` (parent validation, no `backlog_category`).
9. `crates/metis-docs-cli/src/commands/create/mod.rs:17-64` and a new `design.rs` sibling — add `metis create design` subcommand mirroring `specification`.
10. Update MCP server instructions / preset descriptions emitted at session start to list the `design` type.

### Cross-cutting
- Search and list tools should work unchanged once the type is registered, but verify SQL filters / type enums don't pin a closed set.
- Confirm whether the SQLite schema uses a `CHECK` constraint on `document_type`. If yes, add a migration; if it stores raw strings, no migration needed.

## Alternatives Considered

- **Use `specification` for UI designs.** Rejected: specs are living system docs (`discovery → drafting → review → published`); designs have a discrete approval lifecycle and can be scratched. Forcing UI work into specs blurs both types.
- **Make `design` a child of `initiative`.** Rejected: the user explicitly wants designs creatable without an initiative, so projects can capture all UI ideas before committing to implementation.
- **Keep designs as ad-hoc files in the repo.** Rejected: loses short-code referencing, phase tracking, and search integration that initiatives/tasks rely on.
- **Allow design parent to be either Vision or Initiative (mirroring specification).** Deferred: simpler to start with Vision-only. Can be revisited if real usage shows demand for initiative-scoped designs.

## Implementation Plan

Discovery (current phase): finalize the touchpoint list, confirm DB-schema implications, and check whether any tooling assumes a closed set of document types.

Design phase: lock the template content for the new type and finalize on-disk layout details.

Decompose: candidate tasks
1. Core enum + shortcode + phase sequence + transitions (with unit tests).
2. Configuration / preset enablement + parent rules.
3. Template files and document module.
4. Creation service branch + on-disk layout (`.metis/designs/`).
5. MCP `create_document` handler + integration test.
6. End-to-end smoke test: create a design under the vision, transition through phases, archive a scratched design, reference from a task.
7. Docs / MCP-instruction updates so agents discover the new type.

Active: implement tasks in order; ship behind the existing preset system so it's available to all projects on upgrade.