mod db;
mod excerpt;
mod tag;

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
            excerpt::update_excerpt,
            tag::create_tag,
            tag::delete_tag,
            tag::list_excerpt_tags,
            tag::list_tags,
            tag::set_excerpt_tags,
            tag::update_tag
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
