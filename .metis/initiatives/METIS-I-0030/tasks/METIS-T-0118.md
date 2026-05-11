---
id: enable-design-in-presets-and
level: task
title: "Enable Design in presets and enforce Vision parent rule"
short_code: "METIS-T-0118"
created_at: 2026-05-08T10:16:15.352162+00:00
updated_at: 2026-05-11T10:41:35.958406+00:00
parent: METIS-I-0030
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: METIS-I-0030
---

# Enable Design in presets and enforce Vision parent rule

## Parent Initiative

[[METIS-I-0030]]

## Objective

Make `Design` an always-enabled document type in both `streamlined` and `direct` presets, and declare its parent type as `Vision`.

## Depends On

- T-0117 (requires `DocumentType::Design` to exist).

## Files to Modify

### `crates/metis-docs-core/src/domain/configuration.rs`
- Line 43-49 (`is_document_type_allowed`): add `DocumentType::Design` to the always-allowed arm (group with `Vision | Adr | Specification`).
- Line 52-65 (`get_parent_type`): add `DocumentType::Design => Some(DocumentType::Vision)`.
- Line 77-89 (`enabled_document_types`): append `types.push(DocumentType::Design);` after the `Specification` push so designs appear in both streamlined and direct preset listings.

### Tests in same file (`mod tests`)
- Extend `test_document_type_allowed`: assert Design allowed in both presets.
- Extend `test_parent_type_resolution`: assert `streamlined.get_parent_type(Design) == Some(Vision)` and same for direct.
- Extend `test_enabled_document_types`: update both vectors to include `DocumentType::Design`.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `cargo test -p metis-docs-core domain::configuration::tests` passes with the updated assertions.
- [ ] `FlightLevelConfig::streamlined().enabled_document_types()` returns `[Vision, Initiative, Task, Adr, Specification, Design]`.
- [ ] `FlightLevelConfig::direct().enabled_document_types()` returns `[Vision, Task, Adr, Specification, Design]`.

## Implementation Notes

`hierarchy_display()` (line 92-102) intentionally is NOT updated: Design is a peer, not part of the Vision → Initiative → Task chain.

Same caveat as T-0117: in isolation this leaves the workspace non-compiling until T-0119/T-0120 land. Merge as part of the same stack.