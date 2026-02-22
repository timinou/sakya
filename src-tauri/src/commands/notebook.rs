use std::path::PathBuf;

use tauri::{AppHandle, Manager};

use crate::error::AppError;
use crate::models::notes::{NoteContent, NotesConfig};
use crate::services::notes as notes_svc;

/// Resolve the notebook base directory: `app_data_dir()/notebook/`.
fn notebook_base(app: &AppHandle) -> Result<PathBuf, AppError> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Io(std::io::Error::other(e.to_string())))?;
    Ok(data_dir.join("notebook"))
}

#[tauri::command]
pub fn get_notebook_config(app: AppHandle) -> Result<NotesConfig, AppError> {
    let base = notebook_base(&app)?;
    notes_svc::get_notes_config(&base)
}

#[tauri::command]
pub fn save_notebook_config(app: AppHandle, config: NotesConfig) -> Result<(), AppError> {
    let base = notebook_base(&app)?;
    notes_svc::save_notes_config(&base, &config)
}

#[tauri::command]
pub fn get_notebook_note(app: AppHandle, slug: String) -> Result<NoteContent, AppError> {
    let base = notebook_base(&app)?;
    notes_svc::get_note(&base, &slug)
}

#[tauri::command]
pub fn save_notebook_note(
    app: AppHandle,
    slug: String,
    title: String,
    body: String,
) -> Result<(), AppError> {
    let base = notebook_base(&app)?;
    notes_svc::save_note(&base, &slug, &title, &body)
}

#[tauri::command]
pub fn create_notebook_note(app: AppHandle, title: String) -> Result<NoteContent, AppError> {
    let base = notebook_base(&app)?;
    notes_svc::create_note(&base, &title)
}

#[tauri::command]
pub fn delete_notebook_note(app: AppHandle, slug: String) -> Result<(), AppError> {
    let base = notebook_base(&app)?;
    notes_svc::delete_note(&base, &slug)
}

#[tauri::command]
pub fn rename_notebook_note(
    app: AppHandle,
    slug: String,
    new_title: String,
) -> Result<NoteContent, AppError> {
    let base = notebook_base(&app)?;
    notes_svc::rename_note(&base, &slug, &new_title)
}

/// Copy a notebook note into a project.
#[tauri::command]
pub fn copy_notebook_to_project(
    app: AppHandle,
    slug: String,
    project_path: String,
) -> Result<NoteContent, AppError> {
    let nb_base = notebook_base(&app)?;
    let proj_base = PathBuf::from(&project_path);
    notes_svc::copy_note(&nb_base, &proj_base, &slug)
}

/// Copy a project note into the notebook.
#[tauri::command]
pub fn copy_project_to_notebook(
    app: AppHandle,
    slug: String,
    project_path: String,
) -> Result<NoteContent, AppError> {
    let nb_base = notebook_base(&app)?;
    let proj_base = PathBuf::from(&project_path);
    notes_svc::copy_note(&proj_base, &nb_base, &slug)
}

/// Move a notebook note into a project (copy + delete from notebook).
#[tauri::command]
pub fn move_notebook_to_project(
    app: AppHandle,
    slug: String,
    project_path: String,
) -> Result<NoteContent, AppError> {
    let nb_base = notebook_base(&app)?;
    let proj_base = PathBuf::from(&project_path);
    notes_svc::move_note(&nb_base, &proj_base, &slug)
}

/// Move a project note into the notebook (copy + delete from project).
#[tauri::command]
pub fn move_project_to_notebook(
    app: AppHandle,
    slug: String,
    project_path: String,
) -> Result<NoteContent, AppError> {
    let nb_base = notebook_base(&app)?;
    let proj_base = PathBuf::from(&project_path);
    notes_svc::move_note(&proj_base, &nb_base, &slug)
}
