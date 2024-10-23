use std::sync::Arc;

use error::{AppError, AppResult};
use log::{error, info};
use tauri::{
    generate_handler,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, AppHandle, WebviewWindowBuilder,
};

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
            ///// 下面是托盘菜单
            tray_setting(app)?;
            /////////////////////////////////
            Ok(())
        })
        .invoke_handler(generate_handler![test])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// 托盘设置
fn tray_setting(app: &mut App) -> AppResult<()> {
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&quit])?;

    let icon = app.default_window_icon().ok_or(AppError::TauriError("获取窗口默认icon失败".to_string()))?.clone();

    TrayIconBuilder::new()
        .menu(&menu)
        .menu_on_left_click(false)
        .icon(icon)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "quit" => {
                info!("托盘退出按钮被点击了。");
                app.exit(0);
            }
            _ => {
                println!("托盘按钮{:?}没有句柄", event.id);
            }
        })
        .on_tray_icon_event(|tary, event| match event {
            TrayIconEvent::Click { id, position, rect, button, button_state } => {
                if button == MouseButton::Left && button_state == MouseButtonState::Up {
                    info!("鼠标左键点击了托盘icon");
                }
            },
            // tauri::tray::TrayIconEvent::DoubleClick { id, position, rect, button } => todo!(),
            // tauri::tray::TrayIconEvent::Enter { id, position, rect } => todo!(),
            // tauri::tray::TrayIconEvent::Move { id, position, rect } => todo!(),
            // tauri::tray::TrayIconEvent::Leave { id, position, rect } => todo!(),
            _ => info!("未处理的其他托盘icon事件"),
        })
        .build(app)?;
    
    Ok(())
}

#[tauri::command]
async fn test(app: AppHandle) -> AppResult<()> {
    let search_webview_window =
        WebviewWindowBuilder::new(&app, "search", tauri::WebviewUrl::App("search".into()))
            .title("search")
            .center()
            .always_on_top(true)
            .focused(true)
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
