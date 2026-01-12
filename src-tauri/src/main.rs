#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use comrak::{markdown_to_html, ComrakExtensionOptions, ComrakOptions};
use std::{fs, sync::Arc};
use tauri::Manager;
use window_vibrancy::apply_mica;

#[tauri::command]
fn open_markdown(path: String) -> Result<String, String> {
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;

    let options = ComrakOptions {
        extension: ComrakExtensionOptions {
            strikethrough: true,
            table: true,
            autolink: true,
            tasklist: true,
            superscript: true,
            footnotes: true,
            description_lists: true,

            ..ComrakExtensionOptions::default()
        },
        ..ComrakOptions::default()
    };

    let html_output = markdown_to_html(&content, &options);

    Ok(html_output)
}

#[tauri::command]
fn send_markdown_path() -> Result<String, String> {
    let args: Vec<String> = std::env::args().collect();
    if let Some(path) = args.get(1) {
        Ok(path.clone())
    } else {
        Err("Markdown file path not provided.".to_string())
    }
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let args: Vec<String> = std::env::args().collect();
            let window = app.get_window("main").unwrap();
            if let Some(path) = args.get(1) {
                let path_arc = Arc::new(path.clone());
                let app_handle_clone = app.handle();
                // window.set_title(format!("Markdown Viewer - {}", path).as_str());

                app_handle_clone
                    .emit_all("file_path", path.as_str())
                    .unwrap();

                app.listen_global("tauri://window-created", move |_| {
                    let path_clone = path_arc.clone();
                    app_handle_clone
                        .emit_all("file_path", path_clone.as_str())
                        .unwrap();
                });
            }

            #[cfg(target_os = "windows")]
            apply_mica(&window, None)
                .expect("Unsupported platform! 'apply_blur' is only supported on Windows");

            // jank, hopefully will be fixed soon
            // https://github.com/tauri-apps/tauri/discussions/4747
            window.minimize().unwrap();
            window.unminimize().unwrap();
            window.maximize().unwrap();
            window.unmaximize().unwrap();
            window.show().unwrap();
            window.set_focus().unwrap();

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![open_markdown, send_markdown_path])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
