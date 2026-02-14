use std::collections::HashMap;
use std::path::PathBuf;

use crate::error::AppError;
use crate::models::entity::{
    EntityField, EntityFrontmatter, EntityInstance, EntitySchema, EntitySummary, FieldType,
    SchemaSummary, SpiderAxis,
};
use crate::services::frontmatter;
use crate::services::slug_service::slugify;
use crate::services::yaml_service::{read_yaml, write_yaml};

/// List all entity schemas in the project's schemas/ directory.
#[tauri::command]
pub fn list_schemas(project_path: String) -> Result<Vec<SchemaSummary>, AppError> {
    let schemas_dir = PathBuf::from(&project_path).join("schemas");

    if !schemas_dir.exists() {
        return Ok(vec![]);
    }

    let mut summaries = Vec::new();
    let entries = std::fs::read_dir(&schemas_dir)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|e| e.to_str()) == Some("yaml") {
            let schema: EntitySchema = read_yaml(&path)?;
            summaries.push(SchemaSummary {
                name: schema.name,
                entity_type: schema.entity_type,
                field_count: schema.fields.len(),
                axis_count: schema.spider_axes.len(),
            });
        }
    }

    summaries.sort_by(|a, b| a.entity_type.cmp(&b.entity_type));
    Ok(summaries)
}

/// Read a single entity schema by type.
#[tauri::command]
pub fn get_schema(project_path: String, schema_type: String) -> Result<EntitySchema, AppError> {
    let schema_path = PathBuf::from(&project_path)
        .join("schemas")
        .join(format!("{}.yaml", schema_type));

    if !schema_path.exists() {
        return Err(AppError::NotFound(format!(
            "Schema not found: {}",
            schema_type
        )));
    }

    read_yaml(&schema_path)
}

/// Save (create or update) an entity schema.
#[tauri::command]
pub fn save_schema(project_path: String, schema: EntitySchema) -> Result<(), AppError> {
    let schema_path = PathBuf::from(&project_path)
        .join("schemas")
        .join(format!("{}.yaml", schema.entity_type));

    write_yaml(&schema_path, &schema)
}

/// Delete an entity schema by type.
#[tauri::command]
pub fn delete_schema(project_path: String, schema_type: String) -> Result<(), AppError> {
    let schema_path = PathBuf::from(&project_path)
        .join("schemas")
        .join(format!("{}.yaml", schema_type));

    if !schema_path.exists() {
        return Err(AppError::NotFound(format!(
            "Schema not found: {}",
            schema_type
        )));
    }

    std::fs::remove_file(&schema_path)?;
    Ok(())
}

// ── Entity Instance Commands ────────────────────────────────────

/// List all entity instances of a given schema type.
#[tauri::command]
pub fn list_entities(
    project_path: String,
    schema_type: String,
) -> Result<Vec<EntitySummary>, AppError> {
    let entities_dir = PathBuf::from(&project_path)
        .join("entities")
        .join(&schema_type);

    if !entities_dir.exists() {
        return Ok(vec![]);
    }

    let mut summaries = Vec::new();
    let entries = std::fs::read_dir(&entities_dir)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|e| e.to_str()) == Some("md") {
            let content = std::fs::read_to_string(&path)?;
            let doc: frontmatter::ParsedDocument<EntityFrontmatter> = frontmatter::parse(&content)?;
            summaries.push(EntitySummary {
                title: doc.frontmatter.title,
                slug: doc.frontmatter.slug,
                schema_type: doc.frontmatter.schema_type,
                tags: doc.frontmatter.tags,
            });
        }
    }

    summaries.sort_by(|a, b| a.title.cmp(&b.title));
    Ok(summaries)
}

/// Read a single entity instance by schema type and slug.
#[tauri::command]
pub fn get_entity(
    project_path: String,
    schema_type: String,
    slug: String,
) -> Result<EntityInstance, AppError> {
    let entity_path = PathBuf::from(&project_path)
        .join("entities")
        .join(&schema_type)
        .join(format!("{}.md", slug));

    if !entity_path.exists() {
        return Err(AppError::NotFound(format!(
            "Entity not found: {}/{}",
            schema_type, slug
        )));
    }

    let content = std::fs::read_to_string(&entity_path)?;
    let doc: frontmatter::ParsedDocument<EntityFrontmatter> = frontmatter::parse(&content)?;

    Ok(EntityInstance {
        title: doc.frontmatter.title,
        slug: doc.frontmatter.slug,
        schema_slug: doc.frontmatter.schema_type,
        tags: doc.frontmatter.tags,
        spider_values: doc.frontmatter.spider_values,
        fields: doc.frontmatter.fields,
        body: doc.body,
    })
}

/// Create a new entity instance with a generated slug.
#[tauri::command]
pub fn create_entity(
    project_path: String,
    schema_type: String,
    title: String,
) -> Result<EntityInstance, AppError> {
    let slug = slugify(&title);
    let entities_dir = PathBuf::from(&project_path)
        .join("entities")
        .join(&schema_type);

    std::fs::create_dir_all(&entities_dir)?;

    let entity_path = entities_dir.join(format!("{}.md", slug));
    if entity_path.exists() {
        return Err(AppError::AlreadyExists(format!(
            "Entity already exists: {}/{}",
            schema_type, slug
        )));
    }

    let fm = EntityFrontmatter {
        title: title.clone(),
        slug: slug.clone(),
        schema_type: schema_type.clone(),
        tags: vec![],
        spider_values: HashMap::new(),
        fields: HashMap::new(),
    };

    let content = frontmatter::serialize(&fm, "")?;
    std::fs::write(&entity_path, content)?;

    Ok(EntityInstance {
        title,
        slug,
        schema_slug: schema_type,
        tags: vec![],
        spider_values: HashMap::new(),
        fields: HashMap::new(),
        body: String::new(),
    })
}

/// Save (update) an existing entity instance.
#[tauri::command]
pub fn save_entity(project_path: String, entity: EntityInstance) -> Result<(), AppError> {
    let entities_dir = PathBuf::from(&project_path)
        .join("entities")
        .join(&entity.schema_slug);

    std::fs::create_dir_all(&entities_dir)?;

    let entity_path = entities_dir.join(format!("{}.md", entity.slug));

    let fm = EntityFrontmatter {
        title: entity.title,
        slug: entity.slug,
        schema_type: entity.schema_slug,
        tags: entity.tags,
        spider_values: entity.spider_values,
        fields: entity.fields,
    };

    let content = frontmatter::serialize(&fm, &entity.body)?;
    std::fs::write(&entity_path, content)?;
    Ok(())
}

/// Delete an entity instance by schema type and slug.
#[tauri::command]
pub fn delete_entity(
    project_path: String,
    schema_type: String,
    slug: String,
) -> Result<(), AppError> {
    let entity_path = PathBuf::from(&project_path)
        .join("entities")
        .join(&schema_type)
        .join(format!("{}.md", slug));

    if !entity_path.exists() {
        return Err(AppError::NotFound(format!(
            "Entity not found: {}/{}",
            schema_type, slug
        )));
    }

    std::fs::remove_file(&entity_path)?;
    Ok(())
}

/// Rename an entity instance (update title and potentially slug/filename).
#[tauri::command]
pub fn rename_entity(
    project_path: String,
    schema_type: String,
    old_slug: String,
    new_title: String,
) -> Result<EntityInstance, AppError> {
    let mut entity = get_entity(project_path.clone(), schema_type.clone(), old_slug.clone())?;
    let new_slug = slugify(&new_title);

    entity.title = new_title;

    if new_slug == old_slug {
        // Same slug — just update the title in place
        entity.slug = new_slug;
        save_entity(project_path, entity.clone())?;
        return Ok(entity);
    }

    // Different slug — write new file, delete old
    entity.slug = new_slug;
    save_entity(project_path.clone(), entity.clone())?;
    delete_entity(project_path, schema_type, old_slug)?;
    Ok(entity)
}

// ── Default Schemas ─────────────────────────────────────────────

/// Returns the 4 rich default entity schemas for new projects.
pub fn default_schemas() -> Vec<EntitySchema> {
    vec![
        character_schema(),
        place_schema(),
        item_schema(),
        idea_schema(),
    ]
}

fn character_schema() -> EntitySchema {
    EntitySchema {
        name: "Character".to_string(),
        entity_type: "character".to_string(),
        icon: Some("user".to_string()),
        color: Some("#7c4dbd".to_string()),
        description: Some(
            "A character in your story — protagonist, antagonist, or supporting cast.".to_string(),
        ),
        fields: vec![
            EntityField {
                name: "role".to_string(),
                label: "Role".to_string(),
                field_type: FieldType::ShortText,
                required: true,
                placeholder: Some("e.g. Protagonist, Antagonist, Mentor".to_string()),
                description: Some("The character's narrative role.".to_string()),
                options: None,
                min: None,
                max: None,
            },
            EntityField {
                name: "age".to_string(),
                label: "Age".to_string(),
                field_type: FieldType::Number,
                required: false,
                placeholder: None,
                description: Some("The character's age.".to_string()),
                options: None,
                min: Some(0.0),
                max: Some(200.0),
            },
            EntityField {
                name: "occupation".to_string(),
                label: "Occupation".to_string(),
                field_type: FieldType::ShortText,
                required: false,
                placeholder: Some("e.g. Detective, Scholar, Farmer".to_string()),
                description: Some("What the character does for a living.".to_string()),
                options: None,
                min: None,
                max: None,
            },
            EntityField {
                name: "personality".to_string(),
                label: "Personality".to_string(),
                field_type: FieldType::LongText,
                required: false,
                placeholder: Some(
                    "Describe their personality traits, quirks, and mannerisms...".to_string(),
                ),
                description: Some("Core personality traits and behavioral patterns.".to_string()),
                options: None,
                min: None,
                max: None,
            },
            EntityField {
                name: "backstory".to_string(),
                label: "Backstory".to_string(),
                field_type: FieldType::LongText,
                required: false,
                placeholder: Some(
                    "What shaped this character before the story begins...".to_string(),
                ),
                description: Some("The character's history and formative experiences.".to_string()),
                options: None,
                min: None,
                max: None,
            },
            EntityField {
                name: "arc".to_string(),
                label: "Character Arc".to_string(),
                field_type: FieldType::LongText,
                required: false,
                placeholder: Some(
                    "How does this character change throughout the story...".to_string(),
                ),
                description: Some(
                    "The character's growth or transformation over the narrative.".to_string(),
                ),
                options: None,
                min: None,
                max: None,
            },
        ],
        spider_axes: vec![
            SpiderAxis {
                name: "Empathy".to_string(),
                min: 0.0,
                max: 10.0,
                default: 5.0,
                description: Some(
                    "How deeply the character understands and shares the feelings of others."
                        .to_string(),
                ),
            },
            SpiderAxis {
                name: "Resilience".to_string(),
                min: 0.0,
                max: 10.0,
                default: 5.0,
                description: Some(
                    "The character's ability to recover from setbacks and adversity.".to_string(),
                ),
            },
            SpiderAxis {
                name: "Ambition".to_string(),
                min: 0.0,
                max: 10.0,
                default: 5.0,
                description: Some(
                    "The strength of the character's drive to achieve their goals.".to_string(),
                ),
            },
            SpiderAxis {
                name: "Honesty".to_string(),
                min: 0.0,
                max: 10.0,
                default: 5.0,
                description: Some(
                    "How truthful and transparent the character is in their dealings.".to_string(),
                ),
            },
            SpiderAxis {
                name: "Confidence".to_string(),
                min: 0.0,
                max: 10.0,
                default: 5.0,
                description: Some(
                    "The character's belief in their own abilities and judgment.".to_string(),
                ),
            },
            SpiderAxis {
                name: "Adaptability".to_string(),
                min: 0.0,
                max: 10.0,
                default: 5.0,
                description: Some(
                    "How well the character adjusts to new situations and challenges.".to_string(),
                ),
            },
        ],
    }
}

fn place_schema() -> EntitySchema {
    EntitySchema {
        name: "Place".to_string(),
        entity_type: "place".to_string(),
        icon: Some("map-pin".to_string()),
        color: Some("#2e8b57".to_string()),
        description: Some(
            "A location in your story — from vast landscapes to intimate rooms.".to_string(),
        ),
        fields: vec![
            EntityField {
                name: "type".to_string(),
                label: "Type".to_string(),
                field_type: FieldType::Select,
                required: true,
                placeholder: None,
                description: Some("The kind of place this is.".to_string()),
                options: Some(vec![
                    "city".to_string(),
                    "town".to_string(),
                    "village".to_string(),
                    "building".to_string(),
                    "wilderness".to_string(),
                    "other".to_string(),
                ]),
                min: None,
                max: None,
            },
            EntityField {
                name: "era".to_string(),
                label: "Era".to_string(),
                field_type: FieldType::ShortText,
                required: false,
                placeholder: Some("e.g. Medieval, Modern, Futuristic".to_string()),
                description: Some("The time period this place belongs to.".to_string()),
                options: None,
                min: None,
                max: None,
            },
            EntityField {
                name: "atmosphere".to_string(),
                label: "Atmosphere".to_string(),
                field_type: FieldType::LongText,
                required: false,
                placeholder: Some(
                    "Describe the mood, sounds, smells, and feeling of this place...".to_string(),
                ),
                description: Some(
                    "The sensory and emotional qualities of the location.".to_string(),
                ),
                options: None,
                min: None,
                max: None,
            },
            EntityField {
                name: "significance".to_string(),
                label: "Significance".to_string(),
                field_type: FieldType::LongText,
                required: false,
                placeholder: Some("Why does this place matter to the story...".to_string()),
                description: Some("The narrative importance of this location.".to_string()),
                options: None,
                min: None,
                max: None,
            },
            EntityField {
                name: "description".to_string(),
                label: "Description".to_string(),
                field_type: FieldType::LongText,
                required: false,
                placeholder: Some("A detailed description of this place...".to_string()),
                description: Some("Physical description and notable features.".to_string()),
                options: None,
                min: None,
                max: None,
            },
        ],
        spider_axes: vec![
            SpiderAxis {
                name: "Familiarity".to_string(),
                min: 0.0,
                max: 10.0,
                default: 5.0,
                description: Some(
                    "How well-known or recognisable this place is to the characters.".to_string(),
                ),
            },
            SpiderAxis {
                name: "Safety".to_string(),
                min: 0.0,
                max: 10.0,
                default: 5.0,
                description: Some(
                    "How safe or dangerous this place feels to those within it.".to_string(),
                ),
            },
            SpiderAxis {
                name: "Beauty".to_string(),
                min: 0.0,
                max: 10.0,
                default: 5.0,
                description: Some(
                    "The aesthetic appeal or visual impact of this location.".to_string(),
                ),
            },
            SpiderAxis {
                name: "Isolation".to_string(),
                min: 0.0,
                max: 10.0,
                default: 5.0,
                description: Some(
                    "How remote or cut off this place is from the rest of the world.".to_string(),
                ),
            },
            SpiderAxis {
                name: "History".to_string(),
                min: 0.0,
                max: 10.0,
                default: 5.0,
                description: Some(
                    "The depth of historical or cultural significance this place carries."
                        .to_string(),
                ),
            },
            SpiderAxis {
                name: "Emotional Weight".to_string(),
                min: 0.0,
                max: 10.0,
                default: 5.0,
                description: Some(
                    "The emotional resonance this place holds for the characters and story."
                        .to_string(),
                ),
            },
        ],
    }
}

fn item_schema() -> EntitySchema {
    EntitySchema {
        name: "Item".to_string(),
        entity_type: "item".to_string(),
        icon: Some("gem".to_string()),
        color: Some("#c28a1e".to_string()),
        description: Some(
            "An object of significance — weapons, artifacts, documents, or personal effects."
                .to_string(),
        ),
        fields: vec![
            EntityField {
                name: "type".to_string(),
                label: "Type".to_string(),
                field_type: FieldType::Select,
                required: true,
                placeholder: None,
                description: Some("The category of this item.".to_string()),
                options: Some(vec![
                    "weapon".to_string(),
                    "tool".to_string(),
                    "artifact".to_string(),
                    "clothing".to_string(),
                    "document".to_string(),
                    "other".to_string(),
                ]),
                min: None,
                max: None,
            },
            EntityField {
                name: "owner".to_string(),
                label: "Owner".to_string(),
                field_type: FieldType::ShortText,
                required: false,
                placeholder: Some("Who currently possesses this item...".to_string()),
                description: Some("The current or most notable owner of this item.".to_string()),
                options: None,
                min: None,
                max: None,
            },
            EntityField {
                name: "origin".to_string(),
                label: "Origin".to_string(),
                field_type: FieldType::LongText,
                required: false,
                placeholder: Some("Where did this item come from...".to_string()),
                description: Some("The history of how this item came to exist.".to_string()),
                options: None,
                min: None,
                max: None,
            },
            EntityField {
                name: "significance".to_string(),
                label: "Significance".to_string(),
                field_type: FieldType::LongText,
                required: false,
                placeholder: Some("Why does this item matter to the story...".to_string()),
                description: Some("The narrative importance of this item.".to_string()),
                options: None,
                min: None,
                max: None,
            },
            EntityField {
                name: "description".to_string(),
                label: "Description".to_string(),
                field_type: FieldType::LongText,
                required: false,
                placeholder: Some("A detailed description of this item...".to_string()),
                description: Some("Physical appearance and notable features.".to_string()),
                options: None,
                min: None,
                max: None,
            },
        ],
        spider_axes: vec![
            SpiderAxis {
                name: "Sentimental Value".to_string(),
                min: 0.0,
                max: 10.0,
                default: 5.0,
                description: Some(
                    "How emotionally important this item is to its owner or the story.".to_string(),
                ),
            },
            SpiderAxis {
                name: "Rarity".to_string(),
                min: 0.0,
                max: 10.0,
                default: 5.0,
                description: Some(
                    "How unique or hard to find this item is in the story's world.".to_string(),
                ),
            },
            SpiderAxis {
                name: "Power".to_string(),
                min: 0.0,
                max: 10.0,
                default: 5.0,
                description: Some("The item's inherent power, utility, or influence.".to_string()),
            },
            SpiderAxis {
                name: "Age".to_string(),
                min: 0.0,
                max: 10.0,
                default: 5.0,
                description: Some("How old and weathered this item is.".to_string()),
            },
            SpiderAxis {
                name: "Condition".to_string(),
                min: 0.0,
                max: 10.0,
                default: 5.0,
                description: Some("The physical state and preservation of this item.".to_string()),
            },
            SpiderAxis {
                name: "Narrative Importance".to_string(),
                min: 0.0,
                max: 10.0,
                default: 5.0,
                description: Some(
                    "How central this item is to the plot and story progression.".to_string(),
                ),
            },
        ],
    }
}

fn idea_schema() -> EntitySchema {
    EntitySchema {
        name: "Idea".to_string(),
        entity_type: "idea".to_string(),
        icon: Some("lightbulb".to_string()),
        color: Some("#3a7bd5".to_string()),
        description: Some(
            "A thematic concept, motif, or narrative idea to explore in your writing.".to_string(),
        ),
        fields: vec![
            EntityField {
                name: "category".to_string(),
                label: "Category".to_string(),
                field_type: FieldType::Select,
                required: true,
                placeholder: None,
                description: Some("The kind of idea this represents.".to_string()),
                options: Some(vec![
                    "theme".to_string(),
                    "motif".to_string(),
                    "conflict".to_string(),
                    "symbolism".to_string(),
                    "other".to_string(),
                ]),
                min: None,
                max: None,
            },
            EntityField {
                name: "status".to_string(),
                label: "Status".to_string(),
                field_type: FieldType::Select,
                required: true,
                placeholder: None,
                description: Some("How developed this idea currently is.".to_string()),
                options: Some(vec![
                    "seed".to_string(),
                    "developing".to_string(),
                    "mature".to_string(),
                    "implemented".to_string(),
                ]),
                min: None,
                max: None,
            },
            EntityField {
                name: "related_themes".to_string(),
                label: "Related Themes".to_string(),
                field_type: FieldType::LongText,
                required: false,
                placeholder: Some("What other themes or ideas does this connect to...".to_string()),
                description: Some(
                    "Connections to other thematic elements in the story.".to_string(),
                ),
                options: None,
                min: None,
                max: None,
            },
            EntityField {
                name: "description".to_string(),
                label: "Description".to_string(),
                field_type: FieldType::LongText,
                required: false,
                placeholder: Some(
                    "Describe this idea and how it manifests in the story...".to_string(),
                ),
                description: Some("A detailed exploration of this idea.".to_string()),
                options: None,
                min: None,
                max: None,
            },
        ],
        spider_axes: vec![
            SpiderAxis {
                name: "Originality".to_string(),
                min: 0.0,
                max: 10.0,
                default: 5.0,
                description: Some(
                    "How fresh or novel this idea is compared to conventional approaches."
                        .to_string(),
                ),
            },
            SpiderAxis {
                name: "Emotional Impact".to_string(),
                min: 0.0,
                max: 10.0,
                default: 5.0,
                description: Some(
                    "The emotional resonance this idea carries for readers.".to_string(),
                ),
            },
            SpiderAxis {
                name: "Plot Relevance".to_string(),
                min: 0.0,
                max: 10.0,
                default: 5.0,
                description: Some("How directly this idea connects to the main plot.".to_string()),
            },
            SpiderAxis {
                name: "Thematic Depth".to_string(),
                min: 0.0,
                max: 10.0,
                default: 5.0,
                description: Some(
                    "The intellectual and philosophical richness of this idea.".to_string(),
                ),
            },
            SpiderAxis {
                name: "Versatility".to_string(),
                min: 0.0,
                max: 10.0,
                default: 5.0,
                description: Some(
                    "How many ways this idea can manifest across the narrative.".to_string(),
                ),
            },
            SpiderAxis {
                name: "Clarity".to_string(),
                min: 0.0,
                max: 10.0,
                default: 5.0,
                description: Some(
                    "How clearly this idea can be communicated to the reader.".to_string(),
                ),
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::setup_test_dir;

    // ── list_schemas ────────────────────────────────────────────────

    #[test]
    fn list_schemas_empty_directory() {
        let dir = setup_test_dir();
        let schemas_dir = dir.path().join("schemas");
        std::fs::create_dir_all(&schemas_dir).unwrap();

        let result = list_schemas(dir.path().to_str().unwrap().to_string()).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn list_schemas_no_schemas_directory() {
        let dir = setup_test_dir();

        let result = list_schemas(dir.path().to_str().unwrap().to_string()).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn list_schemas_returns_all_default_schemas() {
        let dir = setup_test_dir();
        let schemas_dir = dir.path().join("schemas");
        std::fs::create_dir_all(&schemas_dir).unwrap();

        // Write all default schemas
        for schema in default_schemas() {
            let path = schemas_dir.join(format!("{}.yaml", schema.entity_type));
            write_yaml(&path, &schema).unwrap();
        }

        let summaries = list_schemas(dir.path().to_str().unwrap().to_string()).unwrap();
        assert_eq!(summaries.len(), 4);

        // Sorted alphabetically by entity_type
        assert_eq!(summaries[0].entity_type, "character");
        assert_eq!(summaries[1].entity_type, "idea");
        assert_eq!(summaries[2].entity_type, "item");
        assert_eq!(summaries[3].entity_type, "place");
    }

    #[test]
    fn list_schemas_returns_correct_counts() {
        let dir = setup_test_dir();
        let schemas_dir = dir.path().join("schemas");
        std::fs::create_dir_all(&schemas_dir).unwrap();

        for schema in default_schemas() {
            let path = schemas_dir.join(format!("{}.yaml", schema.entity_type));
            write_yaml(&path, &schema).unwrap();
        }

        let summaries = list_schemas(dir.path().to_str().unwrap().to_string()).unwrap();

        let character = summaries
            .iter()
            .find(|s| s.entity_type == "character")
            .unwrap();
        assert_eq!(character.field_count, 6);
        assert_eq!(character.axis_count, 6);

        let place = summaries.iter().find(|s| s.entity_type == "place").unwrap();
        assert_eq!(place.field_count, 5);
        assert_eq!(place.axis_count, 6);

        let item = summaries.iter().find(|s| s.entity_type == "item").unwrap();
        assert_eq!(item.field_count, 5);
        assert_eq!(item.axis_count, 6);

        let idea = summaries.iter().find(|s| s.entity_type == "idea").unwrap();
        assert_eq!(idea.field_count, 4);
        assert_eq!(idea.axis_count, 6);
    }

    #[test]
    fn list_schemas_ignores_non_yaml_files() {
        let dir = setup_test_dir();
        let schemas_dir = dir.path().join("schemas");
        std::fs::create_dir_all(&schemas_dir).unwrap();

        // Write one valid schema
        let schema = character_schema();
        write_yaml(&schemas_dir.join("character.yaml"), &schema).unwrap();

        // Write a non-yaml file
        std::fs::write(schemas_dir.join("README.md"), "# Schemas").unwrap();

        let summaries = list_schemas(dir.path().to_str().unwrap().to_string()).unwrap();
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].entity_type, "character");
    }

    // ── get_schema ──────────────────────────────────────────────────

    #[test]
    fn get_schema_reads_character_schema() {
        let dir = setup_test_dir();
        let schemas_dir = dir.path().join("schemas");
        std::fs::create_dir_all(&schemas_dir).unwrap();

        let original = character_schema();
        write_yaml(&schemas_dir.join("character.yaml"), &original).unwrap();

        let schema = get_schema(
            dir.path().to_str().unwrap().to_string(),
            "character".to_string(),
        )
        .unwrap();

        assert_eq!(schema.name, "Character");
        assert_eq!(schema.entity_type, "character");
        assert_eq!(schema.color, Some("#7c4dbd".to_string()));
        assert_eq!(schema.fields.len(), 6);
        assert_eq!(schema.spider_axes.len(), 6);

        // Verify field details
        assert_eq!(schema.fields[0].name, "role");
        assert_eq!(schema.fields[0].field_type, FieldType::ShortText);
        assert!(schema.fields[0].required);

        assert_eq!(schema.fields[1].name, "age");
        assert_eq!(schema.fields[1].field_type, FieldType::Number);
        assert_eq!(schema.fields[1].min, Some(0.0));
        assert_eq!(schema.fields[1].max, Some(200.0));

        // Verify spider axis details
        assert_eq!(schema.spider_axes[0].name, "Empathy");
        assert_eq!(schema.spider_axes[0].min, 0.0);
        assert_eq!(schema.spider_axes[0].max, 10.0);
        assert_eq!(schema.spider_axes[0].default, 5.0);
        assert!(schema.spider_axes[0].description.is_some());
    }

    #[test]
    fn get_schema_nonexistent_returns_not_found() {
        let dir = setup_test_dir();
        let schemas_dir = dir.path().join("schemas");
        std::fs::create_dir_all(&schemas_dir).unwrap();

        let result = get_schema(
            dir.path().to_str().unwrap().to_string(),
            "nonexistent".to_string(),
        );

        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(
            err_msg.contains("not found") || err_msg.contains("Not found"),
            "Expected 'not found' error, got: {}",
            err_msg
        );
    }

    #[test]
    fn get_schema_reads_place_schema() {
        let dir = setup_test_dir();
        let schemas_dir = dir.path().join("schemas");
        std::fs::create_dir_all(&schemas_dir).unwrap();

        let original = place_schema();
        write_yaml(&schemas_dir.join("place.yaml"), &original).unwrap();

        let schema = get_schema(
            dir.path().to_str().unwrap().to_string(),
            "place".to_string(),
        )
        .unwrap();

        assert_eq!(schema.name, "Place");
        assert_eq!(schema.color, Some("#2e8b57".to_string()));

        // Verify Select field has options
        let type_field = &schema.fields[0];
        assert_eq!(type_field.name, "type");
        assert_eq!(type_field.field_type, FieldType::Select);
        let options = type_field.options.as_ref().unwrap();
        assert!(options.contains(&"city".to_string()));
        assert!(options.contains(&"wilderness".to_string()));
    }

    #[test]
    fn get_schema_reads_item_schema() {
        let dir = setup_test_dir();
        let schemas_dir = dir.path().join("schemas");
        std::fs::create_dir_all(&schemas_dir).unwrap();

        let original = item_schema();
        write_yaml(&schemas_dir.join("item.yaml"), &original).unwrap();

        let schema =
            get_schema(dir.path().to_str().unwrap().to_string(), "item".to_string()).unwrap();

        assert_eq!(schema.name, "Item");
        assert_eq!(schema.color, Some("#c28a1e".to_string()));
        assert_eq!(schema.spider_axes[0].name, "Sentimental Value");
    }

    #[test]
    fn get_schema_reads_idea_schema() {
        let dir = setup_test_dir();
        let schemas_dir = dir.path().join("schemas");
        std::fs::create_dir_all(&schemas_dir).unwrap();

        let original = idea_schema();
        write_yaml(&schemas_dir.join("idea.yaml"), &original).unwrap();

        let schema =
            get_schema(dir.path().to_str().unwrap().to_string(), "idea".to_string()).unwrap();

        assert_eq!(schema.name, "Idea");
        assert_eq!(schema.color, Some("#3a7bd5".to_string()));
        assert_eq!(schema.fields.len(), 4);

        // Verify category Select options
        let category_field = &schema.fields[0];
        let options = category_field.options.as_ref().unwrap();
        assert!(options.contains(&"theme".to_string()));
        assert!(options.contains(&"symbolism".to_string()));
    }

    // ── save_schema ─────────────────────────────────────────────────

    #[test]
    fn save_schema_writes_and_round_trips() {
        let dir = setup_test_dir();
        let schemas_dir = dir.path().join("schemas");
        std::fs::create_dir_all(&schemas_dir).unwrap();

        let schema = character_schema();
        save_schema(dir.path().to_str().unwrap().to_string(), schema.clone()).unwrap();

        let loaded = get_schema(
            dir.path().to_str().unwrap().to_string(),
            "character".to_string(),
        )
        .unwrap();

        assert_eq!(loaded.name, schema.name);
        assert_eq!(loaded.entity_type, schema.entity_type);
        assert_eq!(loaded.color, schema.color);
        assert_eq!(loaded.fields.len(), schema.fields.len());
        assert_eq!(loaded.spider_axes.len(), schema.spider_axes.len());
    }

    #[test]
    fn save_schema_creates_schemas_directory() {
        let dir = setup_test_dir();
        // Don't create schemas/ directory — write_yaml should handle it

        let schema = character_schema();
        save_schema(dir.path().to_str().unwrap().to_string(), schema).unwrap();

        let loaded = get_schema(
            dir.path().to_str().unwrap().to_string(),
            "character".to_string(),
        )
        .unwrap();
        assert_eq!(loaded.name, "Character");
    }

    #[test]
    fn save_schema_overwrites_existing() {
        let dir = setup_test_dir();
        let schemas_dir = dir.path().join("schemas");
        std::fs::create_dir_all(&schemas_dir).unwrap();

        // Write original
        let schema = character_schema();
        save_schema(dir.path().to_str().unwrap().to_string(), schema).unwrap();

        // Overwrite with modified version
        let mut modified = character_schema();
        modified.name = "Modified Character".to_string();
        modified.color = Some("#ff0000".to_string());
        save_schema(dir.path().to_str().unwrap().to_string(), modified).unwrap();

        let loaded = get_schema(
            dir.path().to_str().unwrap().to_string(),
            "character".to_string(),
        )
        .unwrap();
        assert_eq!(loaded.name, "Modified Character");
        assert_eq!(loaded.color, Some("#ff0000".to_string()));
    }

    // ── delete_schema ───────────────────────────────────────────────

    #[test]
    fn delete_schema_removes_file() {
        let dir = setup_test_dir();
        let schemas_dir = dir.path().join("schemas");
        std::fs::create_dir_all(&schemas_dir).unwrap();

        let schema = character_schema();
        save_schema(dir.path().to_str().unwrap().to_string(), schema).unwrap();

        assert!(schemas_dir.join("character.yaml").exists());

        delete_schema(
            dir.path().to_str().unwrap().to_string(),
            "character".to_string(),
        )
        .unwrap();

        assert!(!schemas_dir.join("character.yaml").exists());
    }

    #[test]
    fn delete_schema_nonexistent_returns_not_found() {
        let dir = setup_test_dir();
        let schemas_dir = dir.path().join("schemas");
        std::fs::create_dir_all(&schemas_dir).unwrap();

        let result = delete_schema(
            dir.path().to_str().unwrap().to_string(),
            "nonexistent".to_string(),
        );

        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(
            err_msg.contains("not found") || err_msg.contains("Not found"),
            "Expected 'not found' error, got: {}",
            err_msg
        );
    }

    #[test]
    fn delete_schema_then_list_excludes_deleted() {
        let dir = setup_test_dir();
        let schemas_dir = dir.path().join("schemas");
        std::fs::create_dir_all(&schemas_dir).unwrap();

        // Write two schemas
        let char_schema = character_schema();
        let place_sch = place_schema();
        save_schema(dir.path().to_str().unwrap().to_string(), char_schema).unwrap();
        save_schema(dir.path().to_str().unwrap().to_string(), place_sch).unwrap();

        // Delete one
        delete_schema(
            dir.path().to_str().unwrap().to_string(),
            "character".to_string(),
        )
        .unwrap();

        let summaries = list_schemas(dir.path().to_str().unwrap().to_string()).unwrap();
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].entity_type, "place");
    }

    // ── default_schemas ─────────────────────────────────────────────

    #[test]
    fn default_schemas_returns_four_schemas() {
        let schemas = default_schemas();
        assert_eq!(schemas.len(), 4);

        let types: Vec<&str> = schemas.iter().map(|s| s.entity_type.as_str()).collect();
        assert!(types.contains(&"character"));
        assert!(types.contains(&"place"));
        assert!(types.contains(&"item"));
        assert!(types.contains(&"idea"));
    }

    #[test]
    fn default_schemas_all_have_colors() {
        for schema in default_schemas() {
            assert!(
                schema.color.is_some(),
                "Schema {} missing color",
                schema.entity_type
            );
        }
    }

    #[test]
    fn default_schemas_all_have_descriptions() {
        for schema in default_schemas() {
            assert!(
                schema.description.is_some(),
                "Schema {} missing description",
                schema.entity_type
            );
        }
    }

    #[test]
    fn default_schemas_all_axes_have_descriptions() {
        for schema in default_schemas() {
            for axis in &schema.spider_axes {
                assert!(
                    axis.description.is_some(),
                    "Schema {} axis {} missing description",
                    schema.entity_type,
                    axis.name
                );
            }
        }
    }

    #[test]
    fn default_schemas_all_serialise_round_trip() {
        let dir = setup_test_dir();
        let schemas_dir = dir.path().join("schemas");
        std::fs::create_dir_all(&schemas_dir).unwrap();

        for schema in default_schemas() {
            let path = schemas_dir.join(format!("{}.yaml", schema.entity_type));
            write_yaml(&path, &schema).unwrap();
            let loaded: EntitySchema = read_yaml(&path).unwrap();
            assert_eq!(loaded.name, schema.name);
            assert_eq!(loaded.entity_type, schema.entity_type);
            assert_eq!(loaded.fields.len(), schema.fields.len());
            assert_eq!(loaded.spider_axes.len(), schema.spider_axes.len());
        }
    }

    // ── create_entity ───────────────────────────────────────────────

    #[test]
    fn create_entity_creates_file_and_returns_instance() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        let result = create_entity(
            pp.clone(),
            "character".to_string(),
            "Frodo Baggins".to_string(),
        )
        .unwrap();

        assert_eq!(result.title, "Frodo Baggins");
        assert_eq!(result.slug, "frodo-baggins");
        assert_eq!(result.schema_slug, "character");
        assert!(result.tags.is_empty());
        assert!(result.spider_values.is_empty());
        assert!(result.fields.is_empty());
        assert!(result.body.is_empty());

        // Verify file exists on disk
        let entity_path = dir.path().join("entities/character/frodo-baggins.md");
        assert!(entity_path.exists());
    }

    #[test]
    fn create_entity_directory_created() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        // entities/character/ does not exist yet
        assert!(!dir.path().join("entities/character").exists());

        create_entity(pp, "character".to_string(), "Gandalf".to_string()).unwrap();

        assert!(dir.path().join("entities/character").exists());
    }

    #[test]
    fn create_entity_duplicate_slug_returns_already_exists() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        create_entity(
            pp.clone(),
            "character".to_string(),
            "Frodo Baggins".to_string(),
        )
        .unwrap();
        let result = create_entity(pp, "character".to_string(), "Frodo Baggins".to_string());

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Already exists"),
            "Expected 'Already exists' error, got: {}",
            err_msg
        );
    }

    #[test]
    fn create_entity_special_characters_in_title_get_slugified() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        let result = create_entity(
            pp,
            "character".to_string(),
            "O'Brien & Friends!".to_string(),
        )
        .unwrap();

        assert_eq!(result.slug, "o-brien-friends");
        assert_eq!(result.title, "O'Brien & Friends!");
        assert!(dir
            .path()
            .join("entities/character/o-brien-friends.md")
            .exists());
    }

    // ── list_entities ───────────────────────────────────────────────

    #[test]
    fn list_entities_empty_when_no_entities() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        let result = list_entities(pp, "character".to_string()).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn list_entities_returns_all_entities_of_type() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        create_entity(pp.clone(), "character".to_string(), "Frodo".to_string()).unwrap();
        create_entity(pp.clone(), "character".to_string(), "Gandalf".to_string()).unwrap();
        create_entity(pp.clone(), "character".to_string(), "Aragorn".to_string()).unwrap();

        let result = list_entities(pp, "character".to_string()).unwrap();
        assert_eq!(result.len(), 3);

        // Sorted by title
        assert_eq!(result[0].title, "Aragorn");
        assert_eq!(result[1].title, "Frodo");
        assert_eq!(result[2].title, "Gandalf");
    }

    #[test]
    fn list_entities_doesnt_return_other_types() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        create_entity(pp.clone(), "character".to_string(), "Frodo".to_string()).unwrap();
        create_entity(pp.clone(), "place".to_string(), "The Shire".to_string()).unwrap();

        let characters = list_entities(pp.clone(), "character".to_string()).unwrap();
        assert_eq!(characters.len(), 1);
        assert_eq!(characters[0].title, "Frodo");

        let places = list_entities(pp, "place".to_string()).unwrap();
        assert_eq!(places.len(), 1);
        assert_eq!(places[0].title, "The Shire");
    }

    // ── get_entity ──────────────────────────────────────────────────

    #[test]
    fn get_entity_reads_created_entity_correctly() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        create_entity(
            pp.clone(),
            "character".to_string(),
            "Frodo Baggins".to_string(),
        )
        .unwrap();

        let entity = get_entity(pp, "character".to_string(), "frodo-baggins".to_string()).unwrap();

        assert_eq!(entity.title, "Frodo Baggins");
        assert_eq!(entity.slug, "frodo-baggins");
        assert_eq!(entity.schema_slug, "character");
        assert!(entity.tags.is_empty());
        assert!(entity.body.is_empty());
    }

    #[test]
    fn get_entity_nonexistent_returns_not_found() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        let result = get_entity(pp, "character".to_string(), "nonexistent".to_string());

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Not found") || err_msg.contains("not found"),
            "Expected 'not found' error, got: {}",
            err_msg
        );
    }

    // ── save_entity ─────────────────────────────────────────────────

    #[test]
    fn save_entity_updates_fields_and_body() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        let mut entity =
            create_entity(pp.clone(), "character".to_string(), "Frodo".to_string()).unwrap();

        entity.tags = vec!["hobbit".to_string(), "ringbearer".to_string()];
        entity.body = "A brave hobbit from the Shire.\n".to_string();
        entity.fields.insert(
            "role".to_string(),
            serde_json::Value::String("Protagonist".to_string()),
        );
        entity.spider_values.insert("Courage".to_string(), 8.5);

        save_entity(pp.clone(), entity).unwrap();

        let loaded = get_entity(pp, "character".to_string(), "frodo".to_string()).unwrap();
        assert_eq!(loaded.tags, vec!["hobbit", "ringbearer"]);
        assert_eq!(loaded.body, "A brave hobbit from the Shire.\n");
        assert_eq!(
            loaded.fields.get("role"),
            Some(&serde_json::Value::String("Protagonist".to_string()))
        );
        assert_eq!(loaded.spider_values.get("Courage"), Some(&8.5));
    }

    #[test]
    fn save_entity_round_trips_frontmatter_correctly() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        let mut entity =
            create_entity(pp.clone(), "item".to_string(), "Sting".to_string()).unwrap();

        entity.tags = vec!["weapon".to_string(), "elvish".to_string()];
        entity.body = "An elvish short sword that glows blue.\n".to_string();
        entity.fields.insert(
            "type".to_string(),
            serde_json::Value::String("weapon".to_string()),
        );
        entity.fields.insert(
            "owner".to_string(),
            serde_json::Value::String("Bilbo".to_string()),
        );
        entity.spider_values.insert("Power".to_string(), 7.0);
        entity.spider_values.insert("Rarity".to_string(), 9.0);

        save_entity(pp.clone(), entity.clone()).unwrap();

        let loaded = get_entity(pp, "item".to_string(), "sting".to_string()).unwrap();

        assert_eq!(loaded.title, "Sting");
        assert_eq!(loaded.slug, "sting");
        assert_eq!(loaded.schema_slug, "item");
        assert_eq!(loaded.tags, vec!["weapon", "elvish"]);
        assert_eq!(loaded.body, "An elvish short sword that glows blue.\n");
        assert_eq!(loaded.fields.len(), 2);
        assert_eq!(loaded.spider_values.len(), 2);
        assert_eq!(loaded.spider_values.get("Power"), Some(&7.0));
        assert_eq!(loaded.spider_values.get("Rarity"), Some(&9.0));
    }

    // ── delete_entity ───────────────────────────────────────────────

    #[test]
    fn delete_entity_removes_file() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        create_entity(pp.clone(), "character".to_string(), "Boromir".to_string()).unwrap();
        let entity_path = dir.path().join("entities/character/boromir.md");
        assert!(entity_path.exists());

        delete_entity(pp, "character".to_string(), "boromir".to_string()).unwrap();
        assert!(!entity_path.exists());
    }

    #[test]
    fn delete_entity_nonexistent_returns_not_found() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        let result = delete_entity(pp, "character".to_string(), "nonexistent".to_string());

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Not found") || err_msg.contains("not found"),
            "Expected 'not found' error, got: {}",
            err_msg
        );
    }

    // ── rename_entity ───────────────────────────────────────────────

    #[test]
    fn rename_entity_updates_title_and_slug() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        create_entity(pp.clone(), "character".to_string(), "Strider".to_string()).unwrap();

        let renamed = rename_entity(
            pp.clone(),
            "character".to_string(),
            "strider".to_string(),
            "Aragorn Son of Arathorn".to_string(),
        )
        .unwrap();

        assert_eq!(renamed.title, "Aragorn Son of Arathorn");
        assert_eq!(renamed.slug, "aragorn-son-of-arathorn");
        assert_eq!(renamed.schema_slug, "character");

        // Old file should be gone
        assert!(!dir.path().join("entities/character/strider.md").exists());
        // New file should exist
        assert!(dir
            .path()
            .join("entities/character/aragorn-son-of-arathorn.md")
            .exists());

        // Should be retrievable by new slug
        let loaded = get_entity(
            pp,
            "character".to_string(),
            "aragorn-son-of-arathorn".to_string(),
        )
        .unwrap();
        assert_eq!(loaded.title, "Aragorn Son of Arathorn");
    }

    #[test]
    fn rename_entity_same_slug_just_updates_title() {
        let dir = setup_test_dir();
        let pp = dir.path().to_str().unwrap().to_string();

        // "Frodo" slugifies to "frodo"
        create_entity(pp.clone(), "character".to_string(), "Frodo".to_string()).unwrap();

        // "FRODO" also slugifies to "frodo" — same slug
        let renamed = rename_entity(
            pp.clone(),
            "character".to_string(),
            "frodo".to_string(),
            "FRODO".to_string(),
        )
        .unwrap();

        assert_eq!(renamed.title, "FRODO");
        assert_eq!(renamed.slug, "frodo");

        // File still exists at same path
        assert!(dir.path().join("entities/character/frodo.md").exists());

        // Title updated in file
        let loaded = get_entity(pp, "character".to_string(), "frodo".to_string()).unwrap();
        assert_eq!(loaded.title, "FRODO");
    }
}
