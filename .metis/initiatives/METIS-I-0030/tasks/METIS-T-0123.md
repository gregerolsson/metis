---
id: update-mcp-server-instructions-and
level: task
title: "Update MCP server instructions and preset descriptions"
short_code: "METIS-T-0123"
created_at: 2026-05-08T10:16:20.847354+00:00
updated_at: 2026-05-11T10:41:38.772034+00:00
parent: METIS-I-0030
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: METIS-I-0030
---

# Update MCP server instructions and preset descriptions

## Parent Initiative

[[METIS-I-0030]]

## Objective

Update the static MCP server instructions so agents discover the `design` document type with the correct phase rules, parent constraint, and shortcode.

## Depends On

- T-0117 (phase rules + shortcode letter)
- T-0118 (preset enablement, so the dynamic header lists Design automatically)
- T-0120 (parent rule wording)
- T-0121 (mention in `create_document` reference)

## Files to Modify

### `crates/metis-docs-mcp/instructions.md`

Lines 9-16 (Document Types & Phases table) — add a row near Specification:

````
| **Design** | UI/UX designs (peer of Initiative) | discovery → review → approved | Vision (any phase) |
````

Lines 22-56 (Phase Transition Rules) — add a new block after Specification:

````
**Design**: `discovery → review → approved`
- discovery → review
- review → approved (forward path: design is approved for implementation)
- review → discovery (kick-back: reviewers can send a design back for rework)
- approved → (terminal, except archive)
````

Lines 67-68 (Short Codes legend) — extend with `**D**=Design`:

````
- **V**=Vision, **S**=Specification, **I**=Initiative, **T**=Task, **A**=ADR, **D**=Design
````

Line 101 (`search_documents` `document_type` doc) — add `, design` to the type list.

Line 117 (`create_document` `document_type` doc) — add `, design`.

Line 119 (`create_document` `parent_id` doc) — change to `"Parent short code (required for initiative/task/specification/design)"`.

After line 207 ("Decomposing Initiatives" section) — add a new workflow:

````
### Capturing UI Designs
Designs live alongside initiatives, parented to the vision. Capture all UI design work as design documents — initiatives are not required.

```
create_document:
  document_type: "design"
  title: "Onboarding flow v2"
  parent_id: "PROJ-V-0001"
```

Designs flow through `discovery → review → approved`. Use `transition_phase` with `phase: "discovery"` from review to send a design back for rework. Scratched designs are archived via `archive_document` from any phase.
````

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `instructions.md` Document Types table contains the Design row.
- [ ] Phase Transition Rules section includes a Design block with the kick-back transition explicitly noted.
- [ ] Short Codes legend includes `**D**=Design`.
- [ ] `create_document` reference mentions `design` in the type list and updates the `parent_id` doc.
- [ ] A new "Capturing UI Designs" workflow is present after the "Decomposing Initiatives" section.
- [ ] Restarting the MCP server and reconnecting an agent shows the new instructions in its session-start system reminder.

## Implementation Notes

The dynamic header in `crates/metis-docs-mcp/src/lib.rs:66-110` (`generate_dynamic_instructions`) reflects the new type automatically because it derives "Enabled Document Types" from `FlightLevelConfig::enabled_document_types()`. After T-0118 lands, that list already includes Design — no change to `lib.rs` is required. Only the static `instructions.md` needs editing.

This task is documentation-only; no Rust code changes.