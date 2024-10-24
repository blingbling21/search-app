use std::sync::Arc;

use error::{AppError, AppResult};
use log::{error, info};
use tauri::{
    generate_handler, menu::{Menu, MenuItem}, tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent}, App, AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, Position, WebviewWindow, WebviewWindowBuilder
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

mod error;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            ///// 创建并设置search window
            create_search_window(app)?;
            ///// 设置托盘菜单
            tray_setting(app)?;
            ///// 设置全局快捷键
            shortcut_setting(app)?;
            /////////////////////////////////
            Ok(())
        })
        .invoke_handler(generate_handler![test])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// 托盘设置，右键显示托盘菜单，左键显示search window
///
/// 托盘菜单有：退出
fn tray_setting(app: &mut App) -> AppResult<()> {
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&quit])?;

    let icon = app
        .default_window_icon()
        .ok_or(AppError::TauriError("获取窗口默认icon失败".to_string()))?
        .clone();

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
            TrayIconEvent::Click {
                id: _,
                position: _,
                rect: _,
                button,
                button_state,
            } => {
                if button == MouseButton::Left && button_state == MouseButtonState::Up {
                    info!("鼠标左键点击了托盘icon");
                    let search_window = tary.app_handle().get_webview_window("search").unwrap();
                    search_window.show().unwrap();
                    search_window.set_focus().unwrap();
                    emit_search_focus(&search_window);
                }
            }
            // tauri::tray::TrayIconEvent::DoubleClick { id, position, rect, button } => todo!(),
            // tauri::tray::TrayIconEvent::Enter { id, position, rect } => todo!(),
            // tauri::tray::TrayIconEvent::Move { id, position, rect } => todo!(),
            // tauri::tray::TrayIconEvent::Leave { id, position, rect } => todo!(),
            _ => info!("未处理的其他托盘icon事件"),
        })
        .build(app)?;

    Ok(())
}

/// 新建search窗口，设置 初始化/失去焦点 后隐藏窗口
fn create_search_window(app: &mut App) -> AppResult<()> {
    let search_webview_window =
        WebviewWindowBuilder::new(app, "search", tauri::WebviewUrl::App("search".into()))
            .title("search")
            .always_on_top(true)
            .visible(false)
            .transparent(true)
            .decorations(false)
            .skip_taskbar(true)
            .resizable(false)
            .shadow(false)
            .build()?;

    if let Some(monitor) = search_webview_window.current_monitor()? {
        let size = monitor.size();
        let window_size = search_webview_window.inner_size()?;
        let x = ((size.width - window_size.width) / 2) as i32;
        let y = (size.height * 2 / 5) as i32;
        search_webview_window.set_position(Position::Physical(PhysicalPosition::new(x, y)))?;
    }

    let search_webview_window_rc = Arc::new(search_webview_window);
    let search_webview_window_rc_clone = Arc::clone(&search_webview_window_rc);

    search_webview_window_rc.on_window_event(move |we| match we {
        tauri::WindowEvent::Focused(focus) => {
            if !focus {
                if let Ok(_) = search_webview_window_rc_clone.hide() {
                    info!("隐藏search窗口");
                } else {
                    error!("隐藏search窗口失败");
                }
            }
        }
        _ => info!("其他事件"),
    });

    Ok(())
}

/// 设置快捷键
fn shortcut_setting(app: &mut App) -> AppResult<()> {
    let ctrl_r_shortcut = Shortcut::new(Some(Modifiers::CONTROL), Code::KeyR);
    let is_register = app.global_shortcut().is_registered(ctrl_r_shortcut);
    info!(
        "全局快捷键{}是否已被注册：{}。",
        ctrl_r_shortcut, is_register
    );
    app.global_shortcut()
        .on_shortcut(ctrl_r_shortcut, move |app_handle, shortcut, _event| {
            if shortcut == &ctrl_r_shortcut {
                info!("按下了全局快捷键ctrl+r");
                let search_window = app_handle.get_webview_window("search").unwrap();
                search_window.show().unwrap();
                search_window.set_focus().unwrap();
                emit_search_focus(&search_window);
            }
        })?;
    Ok(())
}

/// 前端页面size变化时，设置窗口的size
#[tauri::command]
fn test(app: AppHandle, width: u32, height: u32) -> AppResult<()> {
    info!("width: {}, height: {}", width, height);
    let Some(search_window) = app.get_webview_window("search") else {
        return Err(AppError::TauriError("获取search window失败".to_string()));
    };
    search_window.set_size(PhysicalSize::new(width, height))?;
    Ok(())
}

/// 触发前端input框focus
fn emit_search_focus(search_window: &WebviewWindow) {
    if let Err(err) = search_window.emit("search-focus", ()) {
        error!("发送search-focus事件失败: {:?}", err);
    }
}
