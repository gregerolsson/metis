use super::content::DocumentContent;
use super::helpers::FrontmatterParser;
use super::metadata::DocumentMetadata;
use super::traits::{Document, DocumentTemplate, DocumentValidationError};
use super::types::{DocumentId, DocumentType, Phase, Tag};
use chrono::Utc;
use gray_matter;
use std::path::Path;
use tera::{Context, Tera};

/// A Design captures UI/UX design work as a peer of Initiative under the
/// Vision. Designs flow through `discovery → review → approved` and can be
/// scratched (archived) at any phase. Initiatives and tasks reference designs
/// by short code; the relationship is informational, not enforced.
#[derive(Debug)]
pub struct Design {
    core: super::traits::DocumentCore,
}

impl Design {
    /// Create a new Design document with content rendered from template
    pub fn new(
        title: String,
        parent_id: DocumentId,
        tags: Vec<Tag>,
        archived: bool,
        short_code: String,
    ) -> Result<Self, DocumentValidationError> {
        let template_content = include_str!("content.md");
        Self::new_with_template(
            title,
            parent_id,
            tags,
            archived,
            short_code,
            template_content,
        )
    }

    /// Create a new Design document with a custom template
    pub fn new_with_template(
        title: String,
        parent_id: DocumentId,
        tags: Vec<Tag>,
        archived: bool,
        short_code: String,
        template_content: &str,
    ) -> Result<Self, DocumentValidationError> {
        let metadata = DocumentMetadata::new(short_code);

        let mut tera = Tera::default();
        tera.add_raw_template("design_content", template_content)
            .map_err(|e| {
                DocumentValidationError::InvalidContent(format!("Template error: {}", e))
            })?;

        let mut context = Context::new();
        context.insert("title", &title);

        let rendered_content = tera.render("design_content", &context).map_err(|e| {
            DocumentValidationError::InvalidContent(format!("Template render error: {}", e))
        })?;

        let content = DocumentContent::new(&rendered_content);

        Ok(Self {
            core: super::traits::DocumentCore {
                title,
                metadata,
                content,
                parent_id: Some(parent_id),
                blocked_by: Vec::new(),
                tags,
                archived,
                initiative_id: None, // Designs are not part of the initiative hierarchy
            },
        })
    }

    /// Create a Design from existing data (used when loading from file)
    pub fn from_parts(
        title: String,
        metadata: DocumentMetadata,
        content: DocumentContent,
        parent_id: Option<DocumentId>,
        tags: Vec<Tag>,
        archived: bool,
    ) -> Self {
        Self {
            core: super::traits::DocumentCore {
                title,
                metadata,
                content,
                parent_id,
                blocked_by: Vec::new(),
                tags,
                archived,
                initiative_id: None,
            },
        }
    }

    /// Create a Design document by reading and parsing a file
    pub async fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, DocumentValidationError> {
        let raw_content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            DocumentValidationError::InvalidContent(format!("Failed to read file: {}", e))
        })?;

        Self::from_content(&raw_content)
    }

    /// Create a Design document from raw file content string
    pub fn from_content(raw_content: &str) -> Result<Self, DocumentValidationError> {
        let parsed = gray_matter::Matter::<gray_matter::engine::YAML>::new().parse(raw_content);

        let frontmatter = parsed.data.ok_or_else(|| {
            DocumentValidationError::MissingRequiredField("frontmatter".to_string())
        })?;

        let fm_map = match frontmatter {
            gray_matter::Pod::Hash(map) => map,
            _ => {
                return Err(DocumentValidationError::InvalidContent(
                    "Frontmatter must be a hash/map".to_string(),
                ))
            }
        };

        let title = FrontmatterParser::extract_string(&fm_map, "title")?;
        let archived = FrontmatterParser::extract_bool(&fm_map, "archived").unwrap_or(false);

        let created_at = FrontmatterParser::extract_datetime(&fm_map, "created_at")?;
        let updated_at = FrontmatterParser::extract_datetime(&fm_map, "updated_at")?;
        let exit_criteria_met =
            FrontmatterParser::extract_bool(&fm_map, "exit_criteria_met").unwrap_or(false);

        let tags = FrontmatterParser::extract_tags(&fm_map)?;

        let level = FrontmatterParser::extract_string(&fm_map, "level")?;
        if level != "design" {
            return Err(DocumentValidationError::InvalidContent(format!(
                "Expected level 'design', found '{}'",
                level
            )));
        }

        let parent_id = FrontmatterParser::extract_string(&fm_map, "parent")
            .ok()
            .map(DocumentId::from);

        let short_code = FrontmatterParser::extract_string(&fm_map, "short_code")?;
        let metadata = DocumentMetadata::from_frontmatter(
            created_at,
            updated_at,
            exit_criteria_met,
            short_code,
        );
        let content = DocumentContent::from_markdown(&parsed.content);

        Ok(Self::from_parts(
            title, metadata, content, parent_id, tags, archived,
        ))
    }

    /// Get the next phase in the Design sequence (auto-advance only).
    /// The `review → discovery` kick-back is intentionally not auto-selected;
    /// it must be requested explicitly via `transition_phase(Some(Discovery))`.
    fn next_phase_in_sequence(current: Phase) -> Option<Phase> {
        use Phase::*;
        match current {
            Discovery => Some(Review),
            Review => Some(Approved),
            Approved => None, // Terminal phase
            _ => None,        // Invalid phase for Design
        }
    }

    /// Update the phase tag in the document's tags
    fn update_phase_tag(&mut self, new_phase: Phase) {
        self.core.tags.retain(|tag| !matches!(tag, Tag::Phase(_)));
        self.core.tags.push(Tag::Phase(new_phase));
        self.core.metadata.updated_at = Utc::now();
    }

    /// Write the Design document to a file
    pub async fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), DocumentValidationError> {
        let content = self.to_content()?;
        std::fs::write(path.as_ref(), content).map_err(|e| {
            DocumentValidationError::InvalidContent(format!("Failed to write file: {}", e))
        })
    }

    /// Convert the Design document to its markdown string representation
    pub fn to_content(&self) -> Result<String, DocumentValidationError> {
        let mut tera = Tera::default();

        tera.add_raw_template("frontmatter", self.frontmatter_template())
            .map_err(|e| {
                DocumentValidationError::InvalidContent(format!("Template error: {}", e))
            })?;

        let mut context = Context::new();
        context.insert("slug", &DocumentId::title_to_slug(self.title()));
        context.insert("title", self.title());
        context.insert("short_code", &self.metadata().short_code);
        context.insert("created_at", &self.metadata().created_at.to_rfc3339());
        context.insert("updated_at", &self.metadata().updated_at.to_rfc3339());
        context.insert("archived", &self.archived().to_string());
        context.insert(
            "exit_criteria_met",
            &self.metadata().exit_criteria_met.to_string(),
        );
        context.insert(
            "parent_id",
            &self
                .parent_id()
                .map(|id| id.to_string())
                .unwrap_or_default(),
        );

        let tag_strings: Vec<String> = self.tags().iter().map(|tag| tag.to_str()).collect();
        context.insert("tags", &tag_strings);

        // Designs are not part of the initiative hierarchy
        context.insert("initiative_id", "NULL");

        let frontmatter = tera.render("frontmatter", &context).map_err(|e| {
            DocumentValidationError::InvalidContent(format!("Frontmatter render error: {}", e))
        })?;

        let content_body = &self.content().body;

        let acceptance_criteria = if let Some(ac) = &self.content().acceptance_criteria {
            format!("\n\n## Acceptance Criteria\n\n{}", ac)
        } else {
            String::new()
        };

        Ok(format!(
            "---\n{}\n---\n\n{}{}",
            frontmatter.trim_end(),
            content_body,
            acceptance_criteria
        ))
    }
}

impl Document for Design {
    fn id(&self) -> DocumentId {
        DocumentId::from_title(self.title())
    }

    fn document_type(&self) -> DocumentType {
        DocumentType::Design
    }

    fn title(&self) -> &str {
        &self.core.title
    }

    fn metadata(&self) -> &DocumentMetadata {
        &self.core.metadata
    }

    fn content(&self) -> &DocumentContent {
        &self.core.content
    }

    fn core(&self) -> &super::traits::DocumentCore {
        &self.core
    }

    fn can_transition_to(&self, phase: Phase) -> bool {
        if let Ok(current_phase) = self.phase() {
            DocumentType::Design.can_transition(current_phase, phase)
        } else {
            false
        }
    }

    fn parent_id(&self) -> Option<&DocumentId> {
        self.core.parent_id.as_ref()
    }

    fn blocked_by(&self) -> &[DocumentId] {
        &[] // Designs cannot be blocked
    }

    fn validate(&self) -> Result<(), DocumentValidationError> {
        if self.title().trim().is_empty() {
            return Err(DocumentValidationError::InvalidTitle(
                "Design title cannot be empty".to_string(),
            ));
        }

        Ok(())
    }

    fn exit_criteria_met(&self) -> bool {
        false
    }

    fn template(&self) -> DocumentTemplate {
        DocumentTemplate {
            frontmatter: self.frontmatter_template(),
            content: self.content_template(),
            acceptance_criteria: self.acceptance_criteria_template(),
            file_extension: "md",
        }
    }

    fn frontmatter_template(&self) -> &'static str {
        include_str!("frontmatter.yaml")
    }

    fn content_template(&self) -> &'static str {
        include_str!("content.md")
    }

    fn acceptance_criteria_template(&self) -> &'static str {
        include_str!("acceptance_criteria.md")
    }

    fn transition_phase(
        &mut self,
        target_phase: Option<Phase>,
    ) -> Result<Phase, DocumentValidationError> {
        let current_phase = self.phase()?;

        let new_phase = match target_phase {
            Some(phase) => {
                if !self.can_transition_to(phase) {
                    return Err(DocumentValidationError::InvalidPhaseTransition {
                        from: current_phase,
                        to: phase,
                    });
                }
                phase
            }
            None => match Self::next_phase_in_sequence(current_phase) {
                Some(next) => next,
                None => return Ok(current_phase), // Already at terminal phase
            },
        };

        self.update_phase_tag(new_phase);
        Ok(new_phase)
    }

    fn core_mut(&mut self) -> &mut super::traits::DocumentCore {
        &mut self.core
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_design_new_under_vision() {
        let design = Design::new(
            "Onboarding Flow".to_string(),
            DocumentId::new("TEST-V-0001"),
            vec![Tag::Phase(Phase::Discovery)],
            false,
            "TEST-D-0001".to_string(),
        )
        .unwrap();

        assert_eq!(design.title(), "Onboarding Flow");
        assert_eq!(design.document_type(), DocumentType::Design);
        assert_eq!(design.phase().unwrap(), Phase::Discovery);
        assert_eq!(design.parent_id().unwrap().to_string(), "TEST-V-0001");
    }

    #[test]
    fn test_design_from_content_valid() {
        let content = r##"---
id: onboarding-flow
level: design
title: "Onboarding Flow"
short_code: TEST-D-0001
created_at: 2026-05-08T00:00:00Z
updated_at: 2026-05-08T00:00:00Z
parent: TEST-V-0001
archived: false

tags:
  - "#design"
  - "#phase/discovery"

exit_criteria_met: false
---

# Onboarding Flow

## Problem
"##;

        let design = Design::from_content(content).unwrap();
        assert_eq!(design.title(), "Onboarding Flow");
        assert_eq!(design.document_type(), DocumentType::Design);
        assert_eq!(design.phase().unwrap(), Phase::Discovery);
        assert_eq!(design.parent_id().unwrap().to_string(), "TEST-V-0001");
    }

    #[test]
    fn test_design_from_content_rejects_specification_level() {
        let content = r##"---
id: not-a-design
level: specification
title: "Not a Design"
short_code: TEST-S-0001
created_at: 2026-05-08T00:00:00Z
updated_at: 2026-05-08T00:00:00Z
parent: TEST-V-0001
archived: false

tags:
  - "#specification"
  - "#phase/discovery"

exit_criteria_met: false
---
"##;

        let result = Design::from_content(content);
        assert!(result.is_err());
        match result.unwrap_err() {
            DocumentValidationError::InvalidContent(msg) => {
                assert!(msg.contains("Expected level 'design'"));
            }
            _ => panic!("Expected InvalidContent error"),
        }
    }

    #[test]
    fn test_design_transition_phase_auto() {
        let mut design = Design::new(
            "Test Design".to_string(),
            DocumentId::new("TEST-V-0001"),
            vec![Tag::Phase(Phase::Discovery)],
            false,
            "TEST-D-0001".to_string(),
        )
        .unwrap();

        let new_phase = design.transition_phase(None).unwrap();
        assert_eq!(new_phase, Phase::Review);
        assert_eq!(design.phase().unwrap(), Phase::Review);

        let new_phase = design.transition_phase(None).unwrap();
        assert_eq!(new_phase, Phase::Approved);
        assert_eq!(design.phase().unwrap(), Phase::Approved);

        // Auto from Approved stays at Approved
        let new_phase = design.transition_phase(None).unwrap();
        assert_eq!(new_phase, Phase::Approved);
    }

    #[test]
    fn test_design_kickback_review_to_discovery() {
        let mut design = Design::new(
            "Test Design".to_string(),
            DocumentId::new("TEST-V-0001"),
            vec![Tag::Phase(Phase::Review)],
            false,
            "TEST-D-0001".to_string(),
        )
        .unwrap();

        // Explicit kick-back must succeed
        let result = design.transition_phase(Some(Phase::Discovery)).unwrap();
        assert_eq!(result, Phase::Discovery);
        assert_eq!(design.phase().unwrap(), Phase::Discovery);
    }

    #[test]
    fn test_design_cannot_skip_review() {
        let mut design = Design::new(
            "Test Design".to_string(),
            DocumentId::new("TEST-V-0001"),
            vec![Tag::Phase(Phase::Discovery)],
            false,
            "TEST-D-0001".to_string(),
        )
        .unwrap();

        let result = design.transition_phase(Some(Phase::Approved));
        assert!(result.is_err());
        assert_eq!(design.phase().unwrap(), Phase::Discovery);
    }

    #[test]
    fn test_design_to_content_roundtrip() {
        let design = Design::new(
            "My Design".to_string(),
            DocumentId::new("TEST-V-0001"),
            vec![
                Tag::Label("design".to_string()),
                Tag::Phase(Phase::Discovery),
            ],
            false,
            "TEST-D-0001".to_string(),
        )
        .unwrap();

        let content = design.to_content().unwrap();

        let design2 = Design::from_content(&content).unwrap();
        assert_eq!(design2.title(), "My Design");
        assert_eq!(design2.document_type(), DocumentType::Design);
        assert_eq!(design2.phase().unwrap(), Phase::Discovery);
        assert_eq!(design2.parent_id().unwrap().to_string(), "TEST-V-0001");
    }

    #[test]
    fn test_design_blocked_by_empty() {
        let design = Design::new(
            "Test Design".to_string(),
            DocumentId::new("TEST-V-0001"),
            vec![Tag::Phase(Phase::Discovery)],
            false,
            "TEST-D-0001".to_string(),
        )
        .unwrap();

        assert_eq!(design.blocked_by().len(), 0);
    }

    #[test]
    fn test_design_empty_title_validation() {
        let design = Design::from_parts(
            "".to_string(),
            DocumentMetadata::new("TEST-D-0001".to_string()),
            DocumentContent::new("content"),
            Some(DocumentId::new("TEST-V-0001")),
            vec![Tag::Phase(Phase::Discovery)],
            false,
        );

        assert!(design.validate().is_err());
    }
}
