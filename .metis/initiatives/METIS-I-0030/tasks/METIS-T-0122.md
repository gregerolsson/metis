---
id: end-to-end-smoke-test-for-design
level: task
title: "End-to-end smoke test for design lifecycle"
short_code: "METIS-T-0122"
created_at: 2026-05-08T10:16:19.678862+00:00
updated_at: 2026-05-11T10:41:38.287574+00:00
parent: METIS-I-0030
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: METIS-I-0030
---

# End-to-end smoke test for design lifecycle

## Parent Initiative

[[METIS-I-0030]]

## Objective

Add an end-to-end test that exercises the full design document lifecycle through the public API surface, verifying the type works for a real user workflow.

## Depends On

- T-0117, T-0118, T-0119, T-0120, T-0121.

## Files to Create

### `crates/metis-docs-mcp/tests/design_lifecycle_test.rs` (new)

Or extend an existing integration test file under `crates/metis-docs-mcp/tests/` if one matches. Look at existing tests there for the workspace-bootstrapping helper pattern and reuse it.

The test must exercise these scenarios (one `#[tokio::test]` per scenario, each in a fresh tempdir):

1. **Bootstrap** — `initialize_project` with prefix `TEST` produces a workspace under tempdir.
2. **Vision setup** — create a Vision; transition draft → review → published (designs do not require published parent, but this exercises the realistic path).
3. **Create design under Vision** — assert short code matches `TEST-D-0001`, file exists at `.metis/designs/TEST-D-0001/design.md`, frontmatter contains `level: design`, `parent: TEST-V-0001`, and `tags` include `#design` and `#phase/discovery`.
4. **Forward transitions** — transition design discovery → review → approved; assert phase is Approved.
5. **Kick-back** — create a second design, transition to Review, then explicitly transition back to Discovery; assert phase is Discovery.
6. **Re-approve after kick-back** — transition the kicked-back design discovery → review → approved; assert Approved.
7. **Scratched design** — create a third design; transition to Review; archive via `archive_document`; assert it's moved to `.metis/archived/` and `list_documents(include_archived=false)` no longer returns it, but `list_documents(include_archived=true)` does.
8. **Search filter** — `search_documents(query="...", document_type="design")` returns the approved design.
9. **List filter** — `list_documents` includes both non-archived designs.
10. **Reference from task** — create an Initiative + Task, edit the task body to reference `TEST-D-0001` via `edit_document`, read it back, assert the reference text is preserved (no validation, just informational round-trip).
11. **Failure cases**:
    - Create Design with no `parent_id` → returns the "Design requires a Vision parent" error.
    - Create Design parented to an Initiative short code → returns the "Design parent must be a Vision" error.
    - Transition Design from Approved to any other phase → returns InvalidPhaseTransition error (Approved is terminal except for archive).

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `cargo test -p metis-docs-mcp design_lifecycle` passes.
- [ ] All 11 scenarios above are exercised in distinct test functions or clearly delimited steps.
- [ ] No flakiness: each test uses its own tempdir, no shared global state.
- [ ] Tests run successfully with `cargo test --workspace`.

## Implementation Notes

Look at existing integration tests under `crates/metis-docs-mcp/tests/` (or `crates/metis-docs-core/tests/` if the integration tests live there). Reuse the workspace-bootstrap helper rather than duplicating the setup.

The "reference from task" scenario (step 10) is intentionally a no-op for the type system — it only verifies that a task body can mention a design's short code without any unwanted validation/enrichment kicking in.

For the "scratched design" archive flow (step 7), test from BOTH `discovery` and `review` phases to confirm archive works regardless of phase (designs can be scratched at any time).