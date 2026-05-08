---
id: documenttype-design-enum-variant
level: task
title: "DocumentType::Design enum variant, shortcode, phases and transitions"
short_code: "METIS-T-0117"
created_at: 2026-05-08T10:16:13.910072+00:00
updated_at: 2026-05-08T10:16:13.910072+00:00
parent: METIS-I-0030
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: METIS-I-0030
---

# DocumentType::Design enum variant, shortcode, phases and transitions

## Parent Initiative

[[METIS-I-0030]]

## Objective

Add the `Design` variant to `DocumentType`, the new `Phase::Approved` variant, the `D` shortcode letter, and the design-specific phase transitions (including the `review → discovery` kick-back). This is the foundation the rest of the initiative builds on.

## Files to Modify

### `crates/metis-docs-core/src/domain/documents/types.rs`
- Line 153-159 (`DocumentType` enum): add `Design` variant.
- Line 161-171 (`Display`): map `Design → "design"`.
- Line 173-186 (`FromStr`): map `"design" → Ok(DocumentType::Design)`.
- Line 192-227 (`valid_transitions_from`): add `Design` arm:
  - `Discovery → vec![Review]`
  - `Review → vec![Approved, Discovery]` (approval forward, kick-back back to Discovery)
  - `Approved → vec![]`
- Line 241-268 (`phase_sequence`): `Design → vec![Discovery, Review, Approved]`.
- Line 272-299 (`Phase` enum): add `Approved` variant near the existing design-related block (lines 291-295). Update the comment to "Initiative / Design phases".
- Line 301-322 (`Phase::Display`): map `Approved → "approved"`.
- Line 364-395 (`Tag::FromStr`): add `"approved" => Ok(Tag::Phase(Phase::Approved))`.

### `crates/metis-docs-core/src/dal/database/configuration_repository.rs`
- Line 163-177 (`generate_short_code` match): add `"design" => "D"` arm.

## Acceptance Criteria

- [ ] `cargo test -p metis-docs-core domain::documents::types::tests` passes including new tests:
  - `DocumentType::Design.valid_transitions_from(Phase::Discovery) == vec![Phase::Review]`
  - `DocumentType::Design.valid_transitions_from(Phase::Review) == vec![Phase::Approved, Phase::Discovery]`
  - `DocumentType::Design.can_transition(Phase::Review, Phase::Discovery)` is true (kick-back)
  - `DocumentType::Design.phase_sequence() == vec![Phase::Discovery, Phase::Review, Phase::Approved]`
  - `DocumentType::from_str("design").unwrap().to_string() == "design"`
  - `"#phase/approved".parse::<Tag>().unwrap() == Tag::Phase(Phase::Approved)` and round-trip via `to_str()`
- [ ] `cargo test -p metis-docs-core dal::database::configuration_repository::tests` passes including a new test asserting `generate_short_code("design")` returns `PREFIX-D-0001` on first call.

## Implementation Notes

This task only changes the type system. After landing, the workspace **will not compile** until T-0119 and T-0120 also land — `match doc_type { ... }` sites in archive/transition/discovery/validation/factory/template services will become non-exhaustive. Treat T-0117 + T-0119 + T-0120 as a single PR or merge them in immediate succession.

`Phase::Approved` is a new global phase variant. Other types keep their own terminal phases (Vision::Published, Specification::Published, Adr::Decided) — Approved is design-specific.

The `review → discovery` kick-back is the ONLY non-forward transition besides `Task::Blocked → Todo/Active`. Keep the forward-only invariant elsewhere.