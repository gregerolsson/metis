---
id: design-template-files-and-document
level: task
title: "Design template files and document module"
short_code: "METIS-T-0119"
created_at: 2026-05-08T10:16:16.499227+00:00
updated_at: 2026-05-11T10:41:36.338478+00:00
parent: METIS-I-0030
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: METIS-I-0030
---

# Design template files and document module

## Parent Initiative

[[METIS-I-0030]]

## Objective

Add the `Design` document module mirroring `Specification` (parent required, no `blocked_by`, no children), embed the template files, and wire it through the type factory, validation, discovery, and template-loader services so the workspace compiles.

## Depends On

- T-0117 (`DocumentType::Design`, `Phase::Approved`)
- T-0118 (`get_parent_type(Design)`)

## Files to Create

### `crates/metis-docs-core/src/domain/documents/design/mod.rs`
Mirror `crates/metis-docs-core/src/domain/documents/specification/mod.rs` exactly with these substitutions:

- Struct name `Design` instead of `Specification`.
- `level: design` in template/parser. The `from_content` parser at lines 144-150 of specification/mod.rs validates `level == "specification"` — change to `"design"`.
- `next_phase_in_sequence`:
  - `Discovery → Some(Review)`
  - `Review → Some(Approved)` (auto-advance picks Approved, NOT the kick-back; kick-back must be explicit via `transition_phase(Some(Discovery))`)
  - `Approved → None`
- `Document::document_type()` returns `DocumentType::Design`.
- `to_content()` sets `level: design` and `initiative_id: NULL` (designs are not in initiative hierarchy, same convention as Specification).
- All the `from_content` / `to_content` round-trip behavior is otherwise identical to Specification.
- Port the test module verbatim, swapping the type name and updating phase assertions for `Discovery → Review → Approved`.

### `crates/metis-docs-core/src/domain/documents/design/frontmatter.yaml`
Copy `specification/frontmatter.yaml`. Substitutions:
- `level: specification` → `level: design`
- Phase comments: change to `discovery`, `review`, `approved`
- Tag `#specification` → `#design`

### `crates/metis-docs-core/src/domain/documents/design/content.md`
New template suited to UI design work:

````markdown
# {{ title }}

## Problem **[REQUIRED]**
{What user-facing problem this design solves. Who feels it and where in their workflow.}

## Target User **[REQUIRED]**
{Who is this design for? Personas, scenarios, contexts of use.}

## Design Assets **[REQUIRED]**
{Links to .pen files, screenshots, Figma URLs, prototypes. Do not embed binary assets in this file.}

- **Primary mockup**: {path/url}
- **Supporting screens**: {paths/urls}

## User Flow **[REQUIRED]**
{Step-by-step flow through the design. Numbered list or sequence description.}

## Design System Notes **[CONDITIONAL: Reuses or extends design system]**
{Components reused, new patterns introduced, deviations from the design system and why.}

## Alternatives Explored **[CONDITIONAL: Multiple options considered]**
{Other approaches considered and why they were not chosen.}

## Open Questions **[CONDITIONAL: Pending decisions]**
{Outstanding questions that must be answered before approval.}

## Implementation References **[CONDITIONAL: Approved phase]**
{Initiatives/tasks that implement this design. Filled in as work is scheduled.}
````

### `crates/metis-docs-core/src/domain/documents/design/acceptance_criteria.md`
Copy `specification/acceptance_criteria.md`, adjusted for design phases (`discovery`/`review`/`approved`).

### `crates/metis-docs-core/src/templates/design/frontmatter.yaml`
Identical to the domain `frontmatter.yaml`. Two copies are required: `templates/` is the user-overridable copy used by `TemplateLoader`'s fallback chain; `domain/documents/{type}/` is the embedded compile-time copy referenced by `include_str!` (same convention as Specification).

### `crates/metis-docs-core/src/templates/design/content.md`
Identical to the domain `content.md`.

## Files to Modify

### `crates/metis-docs-core/src/domain/documents/mod.rs`
- Line 14: add `pub mod design;` (alphabetical: between `pub mod adr;` line 10 and `pub mod initiative;` line 11 — actually after the existing block, place it where it fits the existing alphabetical-ish order).

### `crates/metis-docs-core/src/lib.rs`
- Line 16-24: add `design::Design,` to the `pub use domain::documents::{ ... }` re-export (place between `adr::Adr` and `initiative::{...}`).

### `crates/metis-docs-core/src/domain/documents/factory.rs`
- Line 44-50 area: add `DocumentType::Design => Design::from_file(path).await.map(...)` arm mirroring the Specification arm.

### `crates/metis-docs-core/src/application/services/template.rs`
- Line 12-42 (`defaults` mod): add `pub mod design { CONTENT, EXIT_CRITERIA }` block using `include_str!` to point at `domain/documents/design/content.md` and `acceptance_criteria.md`.
- Line 181-192 (`get_embedded_template`): add `("design", TemplateType::Content)` and `("design", TemplateType::ExitCriteria)` arms.
- Line 219-260 (`sample_context_for_type`): add `"design" => { context.insert("parent_id", "TEST-V-0001"); }` so template validation has a non-empty parent for rendering.
- Line 336 (`doc_type_letter`): add `"design" => 'D'` arm.

### `crates/metis-docs-core/src/application/services/document/validation.rs`
- Line 171-175: add `DocumentType::Design => match Design::from_file(file_path).await { ... }` arm matching the Specification handling.
- Line 100/107 area: if a match constructs `DocumentInfo` by type, add a Design arm symmetrically.

### `crates/metis-docs-core/src/application/services/document/discovery.rs`
- Line 72: include `DocumentType::Design` in the type list iterated by discovery.
- Line 207-230 and 342-359 areas: add `DocumentType::Design => ...` arms (mirror Specification).
- Line 472-473 (short-code-letter parser): add `"D" => Ok(DocumentType::Design)`.
- Line 501-505 (per-type storage path lookup): add Design arm pointing to `.metis/designs/`.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `cargo build -p metis-docs-core` succeeds (no remaining non-exhaustive match warnings/errors).
- [ ] `cargo test -p metis-docs-core` passes.
- [ ] New unit tests in `domain/documents/design/mod.rs`:
  - `Design::new` succeeds with a Vision parent_id and Discovery phase.
  - `Design::from_content` parses a valid `level: design` document and rejects `level: specification`.
  - `Design::transition_phase(None)` walks Discovery → Review → Approved, then stays at Approved.
  - `Design::transition_phase(Some(Phase::Discovery))` from Review succeeds (kick-back).
  - `Design::transition_phase(Some(Phase::Approved))` from Discovery fails (cannot skip Review).
  - `Design::to_content` round-trips through `from_content` preserving title/phase/parent.
  - `Design::blocked_by()` returns empty.
- [ ] `TemplateLoader::for_workspace(...).load_content_template("design")` returns the embedded template when no project-level override exists.

## Implementation Notes

The two copies of the template (`domain/documents/design/content.md` and `templates/design/content.md`) must stay in sync. The domain copy is `include_str!`-embedded; the templates copy is what users override per-project. This matches the Specification convention.

Design's `level` frontmatter value is `design` (singular), matching the type name from `DocumentType::Display`.

`to_content` sets `initiative_id: NULL` because designs are not in the initiative hierarchy.

Do NOT add an `Approved` arm to the `Tag::FromStr` mapping — already handled by T-0117.