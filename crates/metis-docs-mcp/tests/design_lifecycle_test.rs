//! End-to-end smoke test for the design document lifecycle.
//!
//! Exercises every public API touchpoint: initialize_project, create_document,
//! transition_phase (forward + kick-back), archive_document, list_documents,
//! search_documents, edit_document — plus failure cases.

use anyhow::Result;
use metis_mcp_server::tools::*;
use regex::Regex;
use tempfile::TempDir;

fn extract_text(result: &rust_mcp_sdk::schema::CallToolResult) -> Option<String> {
    match result.content.first() {
        Some(rust_mcp_sdk::schema::ContentBlock::TextContent(text_content)) => {
            Some(text_content.text.clone())
        }
        Some(rust_mcp_sdk::schema::ContentBlock::EmbeddedResource(embedded)) => {
            match &embedded.resource {
                rust_mcp_sdk::schema::EmbeddedResourceResource::TextResourceContents(
                    text_resource,
                ) => Some(text_resource.text.clone()),
                _ => None,
            }
        }
        _ => None,
    }
}

fn extract_short_code(result: &rust_mcp_sdk::schema::CallToolResult) -> String {
    let text = extract_text(result).expect("no text in result");
    let re = Regex::new(r"([A-Z]+-[VITASD]-\d{4})").unwrap();
    re.captures(&text)
        .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
        .unwrap_or_else(|| panic!("could not extract short code from: {}", text))
}

async fn setup() -> Result<(TempDir, String, String)> {
    let temp_dir = tempfile::tempdir()?;
    let project_path = temp_dir.path().to_string_lossy().to_string();
    let metis_path = format!("{}/.metis", project_path);

    let init = InitializeProjectTool {
        project_path: project_path.clone(),
        prefix: Some("TEST".to_string()),
    };
    init.call_tool()
        .await
        .map_err(|e| anyhow::anyhow!("init failed: {:?}", e))?;

    Ok((temp_dir, project_path, metis_path))
}

/// Initialize_project already created a vision; find it and walk it to published.
async fn create_published_vision(metis_path: &str) -> Result<String> {
    // Find vision short code from list
    let list = ListDocumentsTool {
        project_path: metis_path.to_string(),
        include_archived: None,
    };
    let res = list
        .call_tool()
        .await
        .map_err(|e| anyhow::anyhow!("list visions: {:?}", e))?;
    let text = extract_text(&res).unwrap_or_default();
    let re = Regex::new(r"([A-Z]+-V-\d{4})").unwrap();
    let vision_code = re
        .captures(&text)
        .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
        .ok_or_else(|| anyhow::anyhow!("no vision found in list output: {}", text))?;

    // draft -> review -> published
    TransitionPhaseTool {
        project_path: metis_path.to_string(),
        short_code: vision_code.clone(),
        phase: None,
        force: None,
    }
    .call_tool()
    .await
    .map_err(|e| anyhow::anyhow!("vision draft->review: {:?}", e))?;
    TransitionPhaseTool {
        project_path: metis_path.to_string(),
        short_code: vision_code.clone(),
        phase: None,
        force: None,
    }
    .call_tool()
    .await
    .map_err(|e| anyhow::anyhow!("vision review->published: {:?}", e))?;

    Ok(vision_code)
}

#[tokio::test]
async fn test_design_creation_under_vision() -> Result<()> {
    let (_tmp, _project, metis) = setup().await?;
    let vision_code = create_published_vision(&metis).await?;

    let create = CreateDocumentTool {
        project_path: metis.clone(),
        document_type: "design".to_string(),
        title: "Onboarding Flow".to_string(),
        parent_id: Some(vision_code.clone()),
        complexity: None,
        stakeholders: None,
        decision_maker: None,
        backlog_category: None,
    };
    let result = create
        .call_tool()
        .await
        .map_err(|e| anyhow::anyhow!("create design: {:?}", e))?;
    let design_code = extract_short_code(&result);
    assert_eq!(design_code, "TEST-D-0001");

    let design_path = std::path::Path::new(&metis)
        .join("designs")
        .join(&design_code)
        .join("design.md");
    assert!(
        design_path.exists(),
        "expected design at {}",
        design_path.display()
    );

    let content = std::fs::read_to_string(&design_path)?;
    assert!(content.contains("level: design"));
    assert!(content.contains(&format!("parent: {}", vision_code)));
    assert!(content.contains("#design"));
    assert!(content.contains("#phase/discovery"));

    Ok(())
}

#[tokio::test]
async fn test_design_forward_transitions() -> Result<()> {
    let (_tmp, _project, metis) = setup().await?;
    let vision_code = create_published_vision(&metis).await?;

    let create = CreateDocumentTool {
        project_path: metis.clone(),
        document_type: "design".to_string(),
        title: "Forward".to_string(),
        parent_id: Some(vision_code.clone()),
        complexity: None,
        stakeholders: None,
        decision_maker: None,
        backlog_category: None,
    };
    let design_code = extract_short_code(
        &create
            .call_tool()
            .await
            .map_err(|e| anyhow::anyhow!("{:?}", e))?,
    );

    // discovery -> review
    TransitionPhaseTool {
        project_path: metis.clone(),
        short_code: design_code.clone(),
        phase: None,
        force: None,
    }
    .call_tool()
    .await
    .map_err(|e| anyhow::anyhow!("disc->rev: {:?}", e))?;

    // review -> approved
    TransitionPhaseTool {
        project_path: metis.clone(),
        short_code: design_code.clone(),
        phase: None,
        force: None,
    }
    .call_tool()
    .await
    .map_err(|e| anyhow::anyhow!("rev->app: {:?}", e))?;

    let read = ReadDocumentTool {
        project_path: metis.clone(),
        short_code: design_code.clone(),
    };
    let res = read
        .call_tool()
        .await
        .map_err(|e| anyhow::anyhow!("read: {:?}", e))?;
    let text = extract_text(&res).unwrap_or_default();
    assert!(
        text.contains("approved"),
        "expected approved phase in: {}",
        text
    );

    Ok(())
}

#[tokio::test]
async fn test_design_kickback_and_reapproval() -> Result<()> {
    let (_tmp, _project, metis) = setup().await?;
    let vision_code = create_published_vision(&metis).await?;

    let design_code = extract_short_code(
        &CreateDocumentTool {
            project_path: metis.clone(),
            document_type: "design".to_string(),
            title: "Kicked Back".to_string(),
            parent_id: Some(vision_code.clone()),
            complexity: None,
            stakeholders: None,
            decision_maker: None,
            backlog_category: None,
        }
        .call_tool()
        .await
        .map_err(|e| anyhow::anyhow!("create: {:?}", e))?,
    );

    // discovery -> review
    TransitionPhaseTool {
        project_path: metis.clone(),
        short_code: design_code.clone(),
        phase: None,
        force: None,
    }
    .call_tool()
    .await
    .map_err(|e| anyhow::anyhow!("auto disc->rev: {:?}", e))?;

    // review -> discovery (explicit kick-back)
    TransitionPhaseTool {
        project_path: metis.clone(),
        short_code: design_code.clone(),
        phase: Some("discovery".to_string()),
        force: None,
    }
    .call_tool()
    .await
    .map_err(|e| anyhow::anyhow!("kickback rev->disc: {:?}", e))?;

    // back through review -> approved
    TransitionPhaseTool {
        project_path: metis.clone(),
        short_code: design_code.clone(),
        phase: None,
        force: None,
    }
    .call_tool()
    .await
    .map_err(|e| anyhow::anyhow!("disc->rev again: {:?}", e))?;

    TransitionPhaseTool {
        project_path: metis.clone(),
        short_code: design_code.clone(),
        phase: None,
        force: None,
    }
    .call_tool()
    .await
    .map_err(|e| anyhow::anyhow!("rev->approved: {:?}", e))?;

    Ok(())
}

#[tokio::test]
async fn test_design_scratched_archived_at_review() -> Result<()> {
    let (_tmp, _project, metis) = setup().await?;
    let vision_code = create_published_vision(&metis).await?;

    let design_code = extract_short_code(
        &CreateDocumentTool {
            project_path: metis.clone(),
            document_type: "design".to_string(),
            title: "Scratched at Review".to_string(),
            parent_id: Some(vision_code.clone()),
            complexity: None,
            stakeholders: None,
            decision_maker: None,
            backlog_category: None,
        }
        .call_tool()
        .await
        .map_err(|e| anyhow::anyhow!("create: {:?}", e))?,
    );

    TransitionPhaseTool {
        project_path: metis.clone(),
        short_code: design_code.clone(),
        phase: None,
        force: None,
    }
    .call_tool()
    .await
    .map_err(|e| anyhow::anyhow!("disc->rev: {:?}", e))?;

    ArchiveDocumentTool {
        project_path: metis.clone(),
        short_code: design_code.clone(),
    }
    .call_tool()
    .await
    .map_err(|e| anyhow::anyhow!("archive: {:?}", e))?;

    let archived_dir = std::path::Path::new(&metis).join("archived");
    assert!(archived_dir.exists(), "archived dir should exist");

    // list excluding archived: should not contain the design code
    let list = ListDocumentsTool {
        project_path: metis.clone(),
        include_archived: Some(false),
    };
    let res = list
        .call_tool()
        .await
        .map_err(|e| anyhow::anyhow!("list: {:?}", e))?;
    let listed = extract_text(&res).unwrap_or_default();
    assert!(
        !listed.contains(&design_code),
        "archived design should not appear in default list: {}",
        listed
    );

    // list including archived: should contain it
    let list2 = ListDocumentsTool {
        project_path: metis.clone(),
        include_archived: Some(true),
    };
    let res2 = list2
        .call_tool()
        .await
        .map_err(|e| anyhow::anyhow!("list archived: {:?}", e))?;
    let listed2 = extract_text(&res2).unwrap_or_default();
    assert!(
        listed2.contains(&design_code),
        "archived design should appear when include_archived=true: {}",
        listed2
    );

    Ok(())
}

#[tokio::test]
async fn test_design_scratched_archived_at_discovery() -> Result<()> {
    let (_tmp, _project, metis) = setup().await?;
    let vision_code = create_published_vision(&metis).await?;

    let design_code = extract_short_code(
        &CreateDocumentTool {
            project_path: metis.clone(),
            document_type: "design".to_string(),
            title: "Scratched Early".to_string(),
            parent_id: Some(vision_code.clone()),
            complexity: None,
            stakeholders: None,
            decision_maker: None,
            backlog_category: None,
        }
        .call_tool()
        .await
        .map_err(|e| anyhow::anyhow!("create: {:?}", e))?,
    );

    // No transitions — archive directly from discovery
    ArchiveDocumentTool {
        project_path: metis.clone(),
        short_code: design_code.clone(),
    }
    .call_tool()
    .await
    .map_err(|e| anyhow::anyhow!("archive at discovery: {:?}", e))?;

    Ok(())
}

#[tokio::test]
async fn test_design_search_by_type() -> Result<()> {
    let (_tmp, _project, metis) = setup().await?;
    let vision_code = create_published_vision(&metis).await?;

    let design_code = extract_short_code(
        &CreateDocumentTool {
            project_path: metis.clone(),
            document_type: "design".to_string(),
            title: "Searchable Design".to_string(),
            parent_id: Some(vision_code.clone()),
            complexity: None,
            stakeholders: None,
            decision_maker: None,
            backlog_category: None,
        }
        .call_tool()
        .await
        .map_err(|e| anyhow::anyhow!("{:?}", e))?,
    );

    let search = SearchDocumentsTool {
        project_path: metis.clone(),
        query: "Searchable".to_string(),
        document_type: Some("design".to_string()),
        limit: None,
        include_archived: None,
    };
    let res = search
        .call_tool()
        .await
        .map_err(|e| anyhow::anyhow!("search: {:?}", e))?;
    let text = extract_text(&res).unwrap_or_default();
    assert!(
        text.contains(&design_code),
        "search results should include design: {}",
        text
    );

    Ok(())
}

#[tokio::test]
async fn test_design_listed_in_documents() -> Result<()> {
    let (_tmp, _project, metis) = setup().await?;
    let vision_code = create_published_vision(&metis).await?;

    let d1 = extract_short_code(
        &CreateDocumentTool {
            project_path: metis.clone(),
            document_type: "design".to_string(),
            title: "Design One".to_string(),
            parent_id: Some(vision_code.clone()),
            complexity: None,
            stakeholders: None,
            decision_maker: None,
            backlog_category: None,
        }
        .call_tool()
        .await
        .map_err(|e| anyhow::anyhow!("{:?}", e))?,
    );

    let d2 = extract_short_code(
        &CreateDocumentTool {
            project_path: metis.clone(),
            document_type: "design".to_string(),
            title: "Design Two".to_string(),
            parent_id: Some(vision_code.clone()),
            complexity: None,
            stakeholders: None,
            decision_maker: None,
            backlog_category: None,
        }
        .call_tool()
        .await
        .map_err(|e| anyhow::anyhow!("{:?}", e))?,
    );

    let list = ListDocumentsTool {
        project_path: metis.clone(),
        include_archived: None,
    };
    let res = list
        .call_tool()
        .await
        .map_err(|e| anyhow::anyhow!("list: {:?}", e))?;
    let text = extract_text(&res).unwrap_or_default();
    assert!(text.contains(&d1), "list should contain {}: {}", d1, text);
    assert!(text.contains(&d2), "list should contain {}: {}", d2, text);

    Ok(())
}

#[tokio::test]
async fn test_design_referenced_from_task() -> Result<()> {
    let (_tmp, _project, metis) = setup().await?;
    let vision_code = create_published_vision(&metis).await?;

    let design_code = extract_short_code(
        &CreateDocumentTool {
            project_path: metis.clone(),
            document_type: "design".to_string(),
            title: "Referenced Design".to_string(),
            parent_id: Some(vision_code.clone()),
            complexity: None,
            stakeholders: None,
            decision_maker: None,
            backlog_category: None,
        }
        .call_tool()
        .await
        .map_err(|e| anyhow::anyhow!("design: {:?}", e))?,
    );

    let initiative_code = extract_short_code(
        &CreateDocumentTool {
            project_path: metis.clone(),
            document_type: "initiative".to_string(),
            title: "Build the thing".to_string(),
            parent_id: Some(vision_code.clone()),
            complexity: Some("m".to_string()),
            stakeholders: None,
            decision_maker: None,
            backlog_category: None,
        }
        .call_tool()
        .await
        .map_err(|e| anyhow::anyhow!("initiative: {:?}", e))?,
    );

    // initiative phase progression to decompose so we can attach a task
    for _ in 0..3 {
        TransitionPhaseTool {
            project_path: metis.clone(),
            short_code: initiative_code.clone(),
            phase: None,
            force: None,
        }
        .call_tool()
        .await
        .map_err(|e| anyhow::anyhow!("init transition: {:?}", e))?;
    }

    let task_code = extract_short_code(
        &CreateDocumentTool {
            project_path: metis.clone(),
            document_type: "task".to_string(),
            title: "Implement reference".to_string(),
            parent_id: Some(initiative_code.clone()),
            complexity: None,
            stakeholders: None,
            decision_maker: None,
            backlog_category: None,
        }
        .call_tool()
        .await
        .map_err(|e| anyhow::anyhow!("task: {:?}", e))?,
    );

    // Read first to satisfy potential read-before-edit guard
    let _ = ReadDocumentTool {
        project_path: metis.clone(),
        short_code: task_code.clone(),
    }
    .call_tool()
    .await
    .map_err(|e| anyhow::anyhow!("read task: {:?}", e))?;

    // Edit task to add a reference to the design
    let reference_text = format!("Implements design {}.", design_code);
    EditDocumentTool {
        project_path: metis.clone(),
        short_code: task_code.clone(),
        search: "# Implement reference".to_string(),
        replace: format!("# Implement reference\n\n{}", reference_text),
        replace_all: None,
    }
    .call_tool()
    .await
    .map_err(|e| anyhow::anyhow!("edit task: {:?}", e))?;

    // Re-read; reference text should round-trip without enrichment
    let res = ReadDocumentTool {
        project_path: metis.clone(),
        short_code: task_code.clone(),
    }
    .call_tool()
    .await
    .map_err(|e| anyhow::anyhow!("re-read task: {:?}", e))?;
    let text = extract_text(&res).unwrap_or_default();
    assert!(
        text.contains(&reference_text),
        "task body should preserve reference text: {}",
        text
    );

    Ok(())
}

#[tokio::test]
async fn test_design_without_parent_fails() -> Result<()> {
    let (_tmp, _project, metis) = setup().await?;
    // No need to publish vision; the failure is about missing parent_id altogether

    let create = CreateDocumentTool {
        project_path: metis.clone(),
        document_type: "design".to_string(),
        title: "Orphan".to_string(),
        parent_id: None,
        complexity: None,
        stakeholders: None,
        decision_maker: None,
        backlog_category: None,
    };

    let result = create.call_tool().await;
    assert!(result.is_err(), "design without parent_id should fail");
    let err_msg = format!("{:?}", result.unwrap_err());
    assert!(
        err_msg.contains("Design requires a Vision parent"),
        "unexpected error: {}",
        err_msg
    );

    Ok(())
}

#[tokio::test]
async fn test_design_with_initiative_parent_fails() -> Result<()> {
    let (_tmp, _project, metis) = setup().await?;
    let vision_code = create_published_vision(&metis).await?;

    let initiative_code = extract_short_code(
        &CreateDocumentTool {
            project_path: metis.clone(),
            document_type: "initiative".to_string(),
            title: "Some Initiative".to_string(),
            parent_id: Some(vision_code),
            complexity: Some("s".to_string()),
            stakeholders: None,
            decision_maker: None,
            backlog_category: None,
        }
        .call_tool()
        .await
        .map_err(|e| anyhow::anyhow!("init: {:?}", e))?,
    );

    let result = CreateDocumentTool {
        project_path: metis.clone(),
        document_type: "design".to_string(),
        title: "Bad parent".to_string(),
        parent_id: Some(initiative_code),
        complexity: None,
        stakeholders: None,
        decision_maker: None,
        backlog_category: None,
    }
    .call_tool()
    .await;

    assert!(
        result.is_err(),
        "design with initiative parent should fail"
    );
    let err = format!("{:?}", result.unwrap_err());
    assert!(
        err.contains("Design parent must be a Vision"),
        "unexpected error: {}",
        err
    );

    Ok(())
}

#[tokio::test]
async fn test_design_approved_is_terminal() -> Result<()> {
    let (_tmp, _project, metis) = setup().await?;
    let vision_code = create_published_vision(&metis).await?;

    let design_code = extract_short_code(
        &CreateDocumentTool {
            project_path: metis.clone(),
            document_type: "design".to_string(),
            title: "Approved terminal".to_string(),
            parent_id: Some(vision_code),
            complexity: None,
            stakeholders: None,
            decision_maker: None,
            backlog_category: None,
        }
        .call_tool()
        .await
        .map_err(|e| anyhow::anyhow!("{:?}", e))?,
    );

    // discovery -> review
    TransitionPhaseTool {
        project_path: metis.clone(),
        short_code: design_code.clone(),
        phase: None,
        force: None,
    }
    .call_tool()
    .await
    .map_err(|e| anyhow::anyhow!("{:?}", e))?;

    // review -> approved
    TransitionPhaseTool {
        project_path: metis.clone(),
        short_code: design_code.clone(),
        phase: None,
        force: None,
    }
    .call_tool()
    .await
    .map_err(|e| anyhow::anyhow!("{:?}", e))?;

    // approved -> discovery (or any other) must fail
    let result = TransitionPhaseTool {
        project_path: metis.clone(),
        short_code: design_code.clone(),
        phase: Some("discovery".to_string()),
        force: None,
    }
    .call_tool()
    .await;

    assert!(
        result.is_err(),
        "transitioning out of approved should fail"
    );

    Ok(())
}
