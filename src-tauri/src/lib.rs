mod db;
mod excerpt;
mod note;
mod tag;
mod topic;

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
            note::create_note,
            note::delete_note,
            note::list_notes,
            note::list_timeline,
            note::update_note,
            tag::create_tag,
            tag::delete_tag,
            tag::list_excerpt_tags,
            tag::list_tags,
            tag::list_tags_with_counts,
            tag::set_excerpt_tags,
            tag::update_tag,
            topic::add_excerpt_to_topic,
            topic::create_topic,
            topic::create_topic_node,
            topic::delete_topic,
            topic::delete_topic_node,
            topic::list_topic_excerpts,
            topic::list_topic_nodes,
            topic::list_topics,
            topic::remove_excerpt_from_topic,
            topic::update_topic,
            topic::update_topic_excerpt,
            topic::update_topic_node
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
