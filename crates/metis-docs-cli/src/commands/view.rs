use crate::workspace;
use anyhow::Result;
use clap::Args;
use metis_core::{Application, Database};

#[derive(Args)]
pub struct ViewCommand {
    /// Document short code to view (e.g., PROJ-V-0001)
    pub short_code: String,
}

impl ViewCommand {
    pub async fn execute(&self) -> Result<()> {
        let (workspace_exists, metis_dir) = workspace::has_metis_vault();
        if !workspace_exists {
            anyhow::bail!("Not in a Metis workspace. Run 'metis init' to create one.");
        }
        let metis_dir = metis_dir.unwrap();

        let db_path = metis_dir.join("metis.db");
        let database = Database::new(db_path.to_str().unwrap())
            .map_err(|e| anyhow::anyhow!("Failed to open database: {}", e))?;
        let app = Application::new(database);
        app.sync_directory(&metis_dir)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to sync workspace: {}", e))?;

        let db = Database::new(db_path.to_str().unwrap())
            .map_err(|e| anyhow::anyhow!("Failed to open database: {}", e))?;
        let mut repo = db
            .repository()
            .map_err(|e| anyhow::anyhow!("Failed to open repository: {}", e))?;

        let relative_path = repo
            .resolve_short_code_to_filepath(&self.short_code)
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        let full_path = metis_dir.join(&relative_path);
        let content = std::fs::read_to_string(&full_path)
            .map_err(|e| anyhow::anyhow!("Failed to read {}: {}", full_path.display(), e))?;

        print!("{}", content);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::{CreateCommand, InitCommand, SyncCommand};
    use crate::commands::create::CreateCommands;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_view_command_no_workspace() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();

        if std::env::set_current_dir(temp_dir.path()).is_err() {
            return;
        }

        let cmd = ViewCommand {
            short_code: "TEST-V-0001".to_string(),
        };
        let result = cmd.execute().await;

        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(&original);
        }

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Not in a Metis workspace"));
    }

    #[tokio::test]
    async fn test_view_command_not_found() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let init_cmd = InitCommand {
            name: Some("Test Project".to_string()),
            preset: None,
            initiatives: None,
            prefix: None,
        };
        init_cmd.execute().await.unwrap();

        let cmd = ViewCommand {
            short_code: "TEST-X-9999".to_string(),
        };
        let result = cmd.execute().await;

        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(&original);
        }

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_view_command_outputs_vision_content() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let init_cmd = InitCommand {
            name: Some("Test Project".to_string()),
            preset: None,
            initiatives: None,
            prefix: None,
        };
        init_cmd.execute().await.unwrap();
        SyncCommand {}.execute().await.unwrap();

        let cmd = ViewCommand {
            short_code: "TEST-V-0001".to_string(),
        };
        let result = cmd.execute().await;

        let vision_path = temp_dir.path().join(".metis/vision.md");
        let expected = std::fs::read_to_string(&vision_path).unwrap();

        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(&original);
        }

        assert!(result.is_ok(), "view failed: {:?}", result.err());
        assert!(expected.contains("level: vision"));
    }

    #[tokio::test]
    async fn test_view_command_resolves_nested_task() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let init_cmd = InitCommand {
            name: Some("Test Project".to_string()),
            preset: None,
            initiatives: None,
            prefix: None,
        };
        init_cmd.execute().await.unwrap();

        CreateCommand {
            document_type: CreateCommands::Initiative {
                title: "Test Initiative".to_string(),
                vision: "TEST-V-0001".to_string(),
            },
        }
        .execute()
        .await
        .unwrap();

        let cmd = ViewCommand {
            short_code: "TEST-I-0001".to_string(),
        };
        let result = cmd.execute().await;

        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(&original);
        }

        assert!(result.is_ok(), "view failed: {:?}", result.err());
    }
}
