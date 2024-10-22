use std::sync::Arc;

use error::AppResult;
use log::{error, info};
use tauri::{generate_handler, AppHandle, WebviewWindowBuilder};

mod error;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(generate_handler![test])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn test(app: AppHandle) -> AppResult<()> {
    let search_webview_window =
        WebviewWindowBuilder::new(&app, "search", tauri::WebviewUrl::App("search".into()))
            .title("search")
            .center()
            .always_on_top(true)
            .transparent(true)
            .decorations(false)
            .build()?;

    let search_webview_window_rc = Arc::new(search_webview_window);
    let search_webview_window_rc_clone = Arc::clone(&search_webview_window_rc);

    search_webview_window_rc.on_window_event(move |we| match we {
        tauri::WindowEvent::Focused(focus) => {
            if !focus {
                if let Ok(_) = search_webview_window_rc_clone.close() {
                    info!("关闭成功");
                } else {
                    error!("关闭失败");
                }
            }
        }
        _ => info!("其他事件"),
    });

    Ok(())
}
