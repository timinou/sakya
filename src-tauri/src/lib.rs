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
