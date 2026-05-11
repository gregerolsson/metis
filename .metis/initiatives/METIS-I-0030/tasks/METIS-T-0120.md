---
id: creation-service-branch-and-metis
level: task
title: "Creation service branch and .metis/designs/ on-disk layout"
short_code: "METIS-T-0120"
created_at: 2026-05-08T10:16:17.358861+00:00
updated_at: 2026-05-11T10:41:37.118978+00:00
parent: METIS-I-0030
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: METIS-I-0030
---

# Creation service branch and .metis/designs/ on-disk layout

## Parent Initiative

[[METIS-I-0030]]

## Objective

Add `DocumentCreationService::create_design`, wire `Design` into the workspace transition and archive services, create the `.metis/designs/` directory tree on first design creation, and validate that the parent_id resolves to a Vision.

## Depends On

- T-0117, T-0118, T-0119.

## Files to Modify

### `crates/metis-docs-core/src/application/services/document/creation.rs`
- Line 8: add `Design` to the `crate::{Adr, Database, Initiative, MetisError, Specification, Task, Vision}` import list.
- Add a new method `pub async fn create_design(&self, config: DocumentCreationConfig) -> Result<CreationResult>`. Model on `create_specification` (lines 517-583), with these differences:
  - Storage path: `self.workspace_dir.join("designs").join(&short_code).join("design.md")` (NOT `specifications/`).
  - `generate_short_code("design")`.
  - Default phase: `Phase::Discovery`.
  - Default tag label: `Tag::Label("design".to_string())`.
  - **Validate parent**:
    1. Require `config.parent_id` to be `Some(_)`. Error: `"Design requires a Vision parent. Provide parent_id with a Vision short code."`
    2. Look up the parent in the database via `Database::repository().find_by_short_code(parent_id)` (mirror the lookup at lines 242-272 of `create_task_with_config`).
    3. Assert `parent.document_type == "vision"`. Error on mismatch: `"Design parent must be a Vision (got {actual}). Designs cannot be parented to {actual}."`

### `crates/metis-docs-core/src/application/services/workspace/transition.rs`
- Line 110-135: add `DocumentType::Design => { let design = Design::from_file(file_path).await.map_err(...)?; Ok(design.phase()?) }` arm.
- Line 145-223: add `DocumentType::Design => { let mut design = Design::from_file(file_path).await...?; design.transition_phase(Some(target_phase))...; design.to_file(file_path).await...?; }` arm. Use `doc_type: "design"` in the `InvalidPhaseTransition` error mapping. Default fallback phase for `unwrap_or` should be `Phase::Discovery`.

### `crates/metis-docs-core/src/application/services/workspace/archive.rs`
- Line 60-89 (single-file archive): add `DocumentType::Design => { let mut design = Design::from_file(file_path).await...?; design.core_mut().archived = true; design.to_file(file_path).await...?; }` arm.
- Line 139-149 (`match doc_type` for hierarchy archival): include `Design` in the `Vision | Task | Adr | Specification` group — designs have no children, archive as single file.
- Line 360-368 area and 460-499 area: add Design arms to the remaining exhaustive matches (mirror Specification handling).
- Line 464-467 (short-code-letter recovery): add `"D" => Ok(DocumentType::Design)`.

### `crates/metis-docs-core/src/application/services/database.rs`
- Line 119: add `DocumentType::Design` to the iterated `DocumentType` list.

### Tests

Add to the `tests` mod in `creation.rs`:
- `test_create_design_under_vision`: create Vision, create Design parented to it (Vision can be in any phase). Assert file exists at `.metis/designs/{short_code}/design.md`, parent_id correct, default phase is Discovery, tag includes `#design`.
- `test_create_design_without_parent_fails`: assert error message contains `"Design requires a Vision parent"`.
- `test_create_design_with_initiative_parent_fails`: create Initiative, attempt to create Design parented to the initiative, assert error message contains `"Design parent must be a Vision"`.
- `test_create_design_in_direct_preset`: assert design creation works with `FlightLevelConfig::direct()` (designs are always enabled regardless of preset).

Add to the `tests` mod in `transition.rs`:
- `test_design_kickback_review_to_discovery`: create Design (Discovery), transition to Review, then explicitly transition back to Discovery; assert phase is Discovery.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `cargo test -p metis-docs-core` passes including the new tests.
- [ ] Creating a design produces `.metis/designs/{PREFIX}-D-NNNN/design.md` with frontmatter `level: design`, `parent: {VISION-CODE}`, `tags: ["#design", "#phase/discovery"]`.
- [ ] Creating a Design without `parent_id` fails with a clear error mentioning Vision.
- [ ] Creating a Design with a non-Vision `parent_id` fails with a clear error.
- [ ] `archive_document` on a design moves it to the archived directory with the file intact.
- [ ] `transition_phase` on a design walks discovery → review → approved AND supports the explicit review → discovery kick-back.

## Implementation Notes

Vision parent validation happens at creation time by looking up the parent in the DB and checking its `document_type`. This is stricter than Specification (which accepts any parent) and mirrors the Initiative parent-validation pattern.

Designs do NOT need to validate the parent Vision is in `published` phase. Match the existing Initiative behavior — published-phase enforcement is not done at the application layer in core.

The `.metis/designs/` directory is created on first design via `fs::create_dir_all`; no separate "init" step needed.