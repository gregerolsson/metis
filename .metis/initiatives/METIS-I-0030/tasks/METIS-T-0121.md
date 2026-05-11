---
id: mcp-create-document-handler-for
level: task
title: "MCP create_document handler for design type"
short_code: "METIS-T-0121"
created_at: 2026-05-08T10:16:18.652230+00:00
updated_at: 2026-05-11T10:41:37.829643+00:00
parent: METIS-I-0030
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: METIS-I-0030
---

# MCP create_document handler for design type

## Parent Initiative

[[METIS-I-0030]]

## Objective

Expose design creation through the `create_document` MCP tool.

## Depends On

- T-0120 (`DocumentCreationService::create_design`).

## Files to Modify

### `crates/metis-docs-mcp/src/tools/create_document.rs`
- Line 19 (`mcp_tool` description): append `, design`. New text: `"Create a new Metis document (vision, initiative, task, adr, specification, design). Each document gets a unique short code..."`.
- Line 30 (`document_type` field doc-comment): add `design` to the type list.
- Line 33 (`parent_id` field doc-comment): change to `"Parent document short code (required for initiative, task, specification, design). Omit for backlog items."`.
- Line 130-220 (`match doc_type` block): add a new arm AFTER the `DocumentType::Specification` arm:

````rust
DocumentType::Design => {
    if self.parent_id.is_none() {
        return Err(CallToolError::new(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Design requires a Vision parent. Provide parent_id with a Vision short code.",
        )));
    }
    creation_service
        .create_design(config)
        .await
        .map_err(|e| CallToolError::new(e))?
}
````

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `cargo build -p metis-docs-mcp` succeeds (match is exhaustive over `DocumentType`).
- [ ] `create_document` with `document_type: "design"` and a valid Vision `parent_id` succeeds and returns a `PROJ-D-NNNN` short code.
- [ ] `create_document` with `document_type: "design"` and no `parent_id` returns the "Design requires a Vision parent" error.
- [ ] `create_document` with `document_type: "design"` and a non-Vision parent surfaces the parent-type error from the core layer.

## Implementation Notes

The `enabled_document_types` check at line 84-97 already handles config-disabled types generically — no special handling needed for Design.

The MCP layer does NOT validate the parent's type — that's done in `create_design` (T-0120). The MCP layer only checks for the presence of `parent_id`.