mod commands;
mod error;
mod models;
mod services;

#[cfg(test)]
mod test_helpers;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::project::create_project,
            commands::project::open_project,
            commands::project::save_project_manifest,
            commands::entity::list_schemas,
            commands::entity::get_schema,
            commands::entity::save_schema,
            commands::entity::delete_schema,
            commands::entity::list_entities,
            commands::entity::get_entity,
            commands::entity::create_entity,
            commands::entity::save_entity,
            commands::entity::delete_entity,
            commands::entity::rename_entity,
            commands::manuscript::get_manuscript_config,
            commands::manuscript::save_manuscript_config,
            commands::manuscript::get_chapter,
            commands::manuscript::save_chapter,
            commands::manuscript::create_chapter,
            commands::manuscript::delete_chapter,
            commands::manuscript::reorder_chapters,
            commands::manuscript::rename_chapter,
            commands::notes::get_notes_config,
            commands::notes::save_notes_config,
            commands::notes::get_note,
            commands::notes::save_note,
            commands::notes::create_note,
            commands::notes::delete_note,
            commands::notes::rename_note,
            commands::search::search_project,
            commands::search::resolve_wiki_link,
            commands::search::find_backlinks,
            commands::sessions::start_session,
            commands::sessions::end_session,
            commands::sessions::get_sessions,
            commands::sessions::get_session_stats,
            commands::compile::compile_manuscript,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn greet_with_name() {
        let result = greet("World");
        assert_eq!(result, "Hello, World! You've been greeted from Rust!");
    }

    #[test]
    fn greet_with_empty_name() {
        let result = greet("");
        assert_eq!(result, "Hello, ! You've been greeted from Rust!");
    }

    #[test]
    fn greet_with_special_characters() {
        let result = greet("O'Brien & Friends <html>");
        assert_eq!(
            result,
            "Hello, O'Brien & Friends <html>! You've been greeted from Rust!"
        );
    }

    #[test]
    fn greet_with_unicode() {
        let result = greet("世界");
        assert_eq!(result, "Hello, 世界! You've been greeted from Rust!");
    }
}
