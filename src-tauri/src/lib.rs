mod db;
mod excerpt;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let state = db::open_database(app.handle())?;
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            excerpt::archive_excerpt,
            excerpt::create_excerpt,
            excerpt::delete_excerpt,
            excerpt::get_database_path,
            excerpt::list_excerpts,
            excerpt::update_excerpt
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
