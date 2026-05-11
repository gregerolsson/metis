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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::{InitCommand, SyncCommand};
    use metis_core::{Design, Document};
    use std::fs;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_create_new_design_no_workspace() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().unwrap();

        if std::env::set_current_dir(temp_dir.path()).is_ok() {
            let result = create_new_design("Test Design", "TEST-V-0001").await;
            assert!(result.is_err());
            assert!(result
                .unwrap_err()
                .to_string()
                .contains("Not in a Metis workspace"));

            let _ = std::env::set_current_dir(original_dir);
        }
    }

    #[tokio::test]
    async fn test_create_new_design_with_workspace() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().unwrap();

        std::env::set_current_dir(temp_dir.path()).unwrap();

        let init_cmd = InitCommand {
            name: Some("Test Project".to_string()),
            preset: None,
            initiatives: None,
            prefix: None,
        };
        init_cmd.execute().await.unwrap();

        // Index the workspace so the vision short code is resolvable.
        SyncCommand {}.execute().await.unwrap();

        let result = create_new_design("Login Screen", "TEST-V-0001").await;
        assert!(
            result.is_ok(),
            "Failed to create design: {:?}",
            result.err()
        );

        let designs_base = temp_dir.path().join(".metis/designs");
        assert!(designs_base.exists(), ".metis/designs/ should exist");

        let design_files: Vec<_> = fs::read_dir(&designs_base)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry.path().is_file()
                    && entry.path().extension().is_some_and(|ext| ext == "md")
            })
            .collect();

        assert_eq!(design_files.len(), 1, "Expected exactly one design file");

        let design_file = design_files[0].path();
        let filename = design_file.file_name().unwrap().to_str().unwrap();
        assert!(
            filename.starts_with("TEST-D-") && filename.ends_with(".md"),
            "Design file should be in short code format, got {}",
            filename
        );

        let content = fs::read_to_string(&design_file).unwrap();
        assert!(content.contains("level: design"));
        assert!(content.contains("title: \"Login Screen\""));
        assert!(content.contains("#design"));
        assert!(content.contains("#phase/discovery"));

        let parsed = Design::from_file(&design_file).await;
        assert!(
            parsed.is_ok(),
            "Failed to parse design: {:?}",
            parsed.err()
        );

        let design = parsed.unwrap();
        assert_eq!(design.title(), "Login Screen");

        std::env::set_current_dir(original_dir).unwrap();
    }
}
