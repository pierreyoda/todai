use tauri::Manager;

pub mod commands;
pub mod database;
pub mod errors;
pub mod logging;
pub mod state;
pub mod workspaces;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            logging::init(&app.path().app_log_dir()?)?;
            log::info!("Starting todai v{}", app.package_info().version);

            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let app_db_path = data_dir.join(workspaces::APP_DATABASE_FILE_NAME);
            log::info!("Opening app database {}", app_db_path.display());
            let app_db = database::open(app_db_path, &database::APP)?;

            // If there is none or it cannot be opened, the frontend lets the user create or pick one.
            let db = workspaces::open_active(&app_db, jiff::Timestamp::now().as_second())?;
            app.manage(state::AppState::new(app_db, db));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::todos::list_todos,
            commands::todos::list_todos_between,
            commands::todos::list_todo_months,
            commands::todos::create_todo,
            commands::todos::toggle_todo,
            commands::todos::update_todo,
            commands::todos::set_todo_estimate,
            commands::todos::delete_todo,
            commands::tags::list_tags,
            commands::tags::create_tag,
            commands::tags::update_tag,
            commands::tags::delete_tag,
            commands::tags::set_todo_tags,
            commands::workspaces::list_workspaces,
            commands::workspaces::get_active_workspace,
            commands::workspaces::create_workspace,
            commands::workspaces::import_workspace,
            commands::workspaces::switch_to_workspace,
            commands::workspaces::rename_workspace,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
