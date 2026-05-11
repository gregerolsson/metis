---
id: cli-create-design-subcommand
level: task
title: "CLI `create design` subcommand"
short_code: "METIS-T-0124"
created_at: 2026-05-11T10:37:40.074338+00:00
updated_at: 2026-05-11T10:45:20.071950+00:00
parent: METIS-I-0030
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: METIS-I-0030
---

# CLI `create design` subcommand

## Parent Initiative

[[METIS-I-0030]]

## Objective

Expose design creation through the `metis create design` CLI command, mirroring the existing `create specification` subcommand pattern.

## Depends On

- T-0120 (`DocumentCreationService::create_design`).

## Files to Create

### `crates/metis-docs-cli/src/commands/create/design.rs` (new)

Mirror `crates/metis-docs-cli/src/commands/create/specification.rs`. Body:

````rust
use crate::workspace;
use anyhow::Result;
use metis_core::{
    application::services::document::creation::{DocumentCreationConfig, DocumentCreationService},
    Phase, Tag,
};

/// Create a new Design document with defaults and write to file
pub async fn create_new_design(title: &str, vision: &str) -> Result<()> {
    let (workspace_exists, metis_dir) = workspace::has_metis_vault();
    if !workspace_exists {
        anyhow::bail!("Not in a Metis workspace. Run 'metis init' to create one.");
    }
    let metis_dir = metis_dir.unwrap();

    let creation_service = DocumentCreationService::new(&metis_dir);

    let config = DocumentCreationConfig {
        title: title.to_string(),
        description: None,
        parent_id: Some(vision.into()),
        tags: vec![
            Tag::Label("design".to_string()),
            Tag::Phase(Phase::Discovery),
        ],
        phase: Some(Phase::Discovery),
        complexity: None,
    };

    let result = creation_service
        .create_design(config)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create design: {}", e))?;

    println!("✓ Created Design: {}", result.file_path.display());
    println!("  ID: {}", result.document_id);
    println!("  Short Code: {}", result.short_code);
    println!("  Title: {}", title);
    println!("  Parent: {}", vision);

    Ok(())
}
````

Include a `#[cfg(test)] mod tests` block following the same pattern as `specification.rs` (no-workspace error case + with-workspace round-trip that checks `.metis/designs/{SHORT_CODE}/design.md` exists and parses back via `Design::from_file`).

## Files to Modify

### `crates/metis-docs-cli/src/commands/create/mod.rs`

- Line 1-4: add `mod design;` to the module declarations (keep alphabetical: `adr, design, initiative, specification, task`).
- Line 17-47 (`CreateCommands` enum): add a `Design` variant between `Adr` and `Specification`:

````rust
/// Create a new design document
Design {
    /// Design title
    title: String,
    /// Parent vision short code (e.g., PROJ-V-0001)
    #[arg(short, long)]
    vision: String,
},
````

- Line 51-64 (`match &self.document_type` block): add a `CreateCommands::Design { title, vision }` arm calling `design::create_new_design(title, vision).await?;`.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `cargo build -p metis-docs-cli` succeeds (match is exhaustive over `CreateCommands`).
- [ ] `metis create --help` lists `design` as a subcommand.
- [ ] `metis create design --help` shows the title positional and `--vision` flag.
- [ ] `metis create design "Login screen" --vision PROJ-V-0001` creates `.metis/designs/PROJ-D-NNNN/design.md`, prints the success summary, and the auto-sync at the end of `CreateCommand::execute` indexes the new document.
- [ ] Roundtrip test (`Design::from_file`) passes in the new `tests` module.
- [ ] Running with no Vision parent in the workspace surfaces the parent-not-found error from the core layer.

## Implementation Notes

The CLI does not validate the parent's type — that's done in `create_design` (T-0120). The CLI only requires `--vision` to be present (clap enforces this since the flag is non-optional in the struct).

The auto-sync at `mod.rs:67-69` runs after every create command and will pick up the new design file without any changes.

This task is a sibling to T-0121 (MCP `create_document` handler). Both depend only on T-0120 and can be implemented in either order once T-0120 lands.