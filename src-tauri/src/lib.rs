use comrak::{markdown_to_html, ComrakExtensionOptions, ComrakOptions};
use std::fs;
use tauri::{Emitter, Manager};
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let args: Vec<String> = std::env::args().collect();
            let window = app.get_webview_window("main").unwrap();

            if let Some(path) = args.get(1) {
                let _ = window.emit("file_path", path.as_str());
            }

            #[cfg(target_os = "windows")]
            let _ = apply_mica(&window, None);

            // focus jank fix from v1
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
