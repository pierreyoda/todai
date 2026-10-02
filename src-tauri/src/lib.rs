use tauri::Manager;

pub mod commands;
pub mod database;
pub mod errors;
pub mod logging;
pub mod state;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            logging::init(&app.path().app_log_dir()?)?;
            log::info!("Starting todai v{}", app.package_info().version);

            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let db_path = data_dir.join("todai.sqlite3");
            log::info!("Opening database {}", db_path.display());
            let db = database::open(db_path)?;
            app.manage(state::AppState::new(db));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::todos::list_todos,
            commands::todos::create_todo,
            commands::todos::toggle_todo,
            commands::tags::list_tags,
            commands::tags::create_tag,
            commands::tags::update_tag,
            commands::tags::delete_tag,
            commands::tags::set_todo_tags,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
