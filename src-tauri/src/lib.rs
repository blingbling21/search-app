use std::{
    fs,
    os::windows::process::CommandExt,
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
};

use error::{AppError, AppResult};
use tauri::{
    generate_handler,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, Position, State,
    WebviewWindow, WebviewWindowBuilder,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};
use tracing::{error, info};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::fmt::writer::MakeWriterExt;

mod error;

struct AppState {
    start_path: PathBuf,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // let Ok(settings) = Config::builder()
    //     .add_source(config::File::with_name("config"))
    //     .build()
    // else {
    //     return error!("读取配置文件失败。");
    // };
    // let settings = settings
    //     .try_deserialize::<HashMap<String, String>>()
    //     .expect("反序列化失败");
    // info!("settings: {:?}", settings);

    let Ok(mut current_exe) = std::env::current_exe() else {
        return error!("获取应用程序当前路径失败。");
    };
    info!("current_exe: {:?}", current_exe);
    current_exe.pop();
    let mut log_path = current_exe.clone();
    log_path.push("log");
    let file_appender = RollingFileAppender::new(Rotation::DAILY, log_path, "log.log");
    let console_writer = std::io::stdout;
    tracing_subscriber::fmt()
        .with_writer(console_writer.and(file_appender))
        .init();

    let mut current_dir = current_exe.clone();
    current_dir.push("startapp");
    info!("current_dir: {:?}", current_dir);
    if !current_dir.is_dir() {
        info!("文件夹{:?}不存在，进行创建。", current_dir);
        if !fs::create_dir(&current_dir).is_ok() {
            return error!("文件夹{:?}创建失败。", current_dir);
        }
        info!("文件夹{:?}创建成功。", current_dir);
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            // if cfg!(debug_assertions) {
            // app.handle().plugin(
            //     tauri_plugin_log::Builder::default()
            //         .level(log::LevelFilter::Info)
            //         .build(),
            // )?;
            // }
            app.manage(AppState {
                start_path: current_dir,
            });
            ///// 创建并设置search window
            create_search_window(app)?;
            ///// 设置托盘菜单
            tray_setting(app)?;
            ///// 设置全局快捷键
            shortcut_setting(app)?;
            /////////////////////////////////
            Ok(())
        })
        .invoke_handler(generate_handler![
            window_resize,
            get_search_result,
            execute_app // choose_dir,
                        // get_app_path
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// 托盘设置，右键显示托盘菜单，左键显示search window
///
/// 托盘菜单有：退出
fn tray_setting(app: &mut App) -> AppResult<()> {
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    // let setting = MenuItem::with_id(app, "setting", "设置", true, None::<&str>)?;
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
            // "setting" => {
            //     info!("托盘设置按钮被点击了。");
            //     match create_setting_window(app) {
            //         Ok(_) => info!("设置窗口创建成功"),
            //         Err(err) => error!("设置窗口创建失败: {:?}", err),
            //     }
            // }
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
            _ => {},
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
                    if let Err(err) = search_webview_window_rc_clone.emit("hidewindow", ()) {
                        error!("发送hidewindow事件失败: {:?}", err);
                    }
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

// /// 创建设置页面的窗口
// fn create_setting_window(app: &AppHandle) -> AppResult<()> {
//     WebviewWindowBuilder::new(app, "setting", tauri::WebviewUrl::App("setting".into()))
//         .title("设置")
//         .center()
//         .build()?;

//     Ok(())
// }

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
fn window_resize(app: AppHandle, width: u32, height: u32) -> AppResult<()> {
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

/// 根据搜索值返回对应的软件名称列表
#[tauri::command]
fn get_search_result(state: State<'_, AppState>, search_value: String) -> AppResult<Vec<String>> {
    if search_value == "" {
        return Ok(vec![]);
    }
    info!("输入的搜索值：{}", search_value);
    let path = Path::new(&state.start_path);
    if !path.is_dir() {
        return Err(AppError::OtherError("配置目录不存在".to_string()));
    }
    let mut list = vec![];
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let Ok(file_name) = entry.file_name().into_string() else {
            return Err(AppError::OtherError("字符串转换时出错".to_string()));
        };
        let file_type = entry.file_type()?;
        let file_meta = entry.metadata()?;
        info!("file_name: {:?}", file_name);
        info!("file_type: {:?}", file_type);
        info!("file_meta: {:?}", file_meta);
        info!(
            "file_name.contains(&search_value): {}",
            file_name.contains(&search_value)
        );
        if file_name.contains(&search_value) {
            list.push(file_name);
        }
    }
    info!("list: {:?}", list);
    Ok(list)
}

/// 根据前端返回的app_name，执行程序
#[tauri::command]
fn execute_app(app: AppHandle, state: State<'_, AppState>, app_name: String) -> AppResult<()> {
    let Some(search_window) = app.get_webview_window("search") else {
        return Err(AppError::TauriError("获取search window失败".to_string()));
    };
    if let Ok(_) = search_window.hide() {
        if let Err(err) = search_window.emit("hidewindow", ()) {
            error!("发送hidewindow事件失败: {:?}", err);
        }
        info!("隐藏search窗口");
    } else {
        error!("隐藏search窗口失败");
    }

    let Some(app_path) = state.start_path.as_path().to_str() else {
        error!("state.start_path转换为&str时出错。");
        return Err(AppError::OtherError(
            "state.start_path转换为&str时出错。".to_string(),
        ));
    };
    let app_path = format!("{}\\\\{}", app_path, app_name);
    info!("app_path: {}", app_path);
    let output = Command::new("cmd")
        .args(["/C", "start", "", &app_path])
        .creation_flags(0x08000000)
        .output()?;
    info!("output: {:?}", output);
    Ok(())
}

// /// 打开原本的app_path文件夹，然后选择一个文件夹，并返回新选择的文件夹的路径
// #[tauri::command]
// fn choose_dir(state: State<'_, AppState>) -> AppResult<String> {
//     let path = Path::new(&state.start_path);
//     info!("文件夹原本路径: {:?}", path);
//     let folder = FileDialog::new().set_directory(path).pick_folder();
//     let Some(folder) = folder else {
//         return Err(AppError::OtherError("获取文件夹路径失败".to_string()));
//     };
//     info!("获取文件夹路径成功: {:?}", folder);
//     Ok(format!("{:?}", folder))
// }

// /// 返回配置文件中app_path给前端
// #[tauri::command]
// fn get_app_path(state: State<'_, AppState>) -> AppResult<String> {
//     let Some(app_path) = state.start_path.as_path().to_str() else {
//         error!("state.start_path转换为&str时出错。");
//         return Err(AppError::OtherError(
//             "state.start_path转换为&str时出错。".to_string(),
//         ));
//     };
//     info!("获取AppState中config的app_path成功: {}", app_path);
//     Ok(app_path.into())
// }

// 更新配置文件中的app_path
// #[tauri::command]
// fn save_app_path(state: State<'_, Mutex<AppState>>, new_app_path: String) -> AppResult<()> {
//     let Ok(state) = state.lock() else {
//         return Err(AppError::OtherError(
//             "get_search_result：state解锁时发生错误".to_string(),
//         ));
//     };

//     Ok(())
// }
