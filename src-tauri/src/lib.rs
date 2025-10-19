use std::fs;
use tauri::Manager;
use tauri::tray::{TrayIconBuilder, TrayIconEvent, MouseButton, MouseButtonState};
use tauri::menu::{Menu, MenuItem, Submenu, CheckMenuItemBuilder};
use serde::{Deserialize, Serialize};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};

// 窗口状态结构
#[derive(Serialize, Deserialize, Debug)]
struct WindowState {
    width: u32,
    height: u32,
    x: i32,
    y: i32,
}

// 保存待办事项
#[tauri::command]
fn save_todos(app: tauri::AppHandle, todos: String) -> Result<(), String> {
    let app_dir = app.path().app_data_dir()
        .map_err(|e| e.to_string())?;
    
    // 确保目录存在
    fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;
    
    let todos_path = app_dir.join("todos.json");
    fs::write(todos_path, todos).map_err(|e| e.to_string())?;
    
    Ok(())
}

// 加载待办事项
#[tauri::command]
fn load_todos(app: tauri::AppHandle) -> Result<String, String> {
    let app_dir = app.path().app_data_dir()
        .map_err(|e| e.to_string())?;
    
    let todos_path = app_dir.join("todos.json");
    
    if todos_path.exists() {
        fs::read_to_string(todos_path).map_err(|e| e.to_string())
    } else {
        Ok("[]".to_string())
    }
}

// 保存窗口状态
#[tauri::command]
fn save_window_state(app: tauri::AppHandle, width: u32, height: u32, x: i32, y: i32) -> Result<(), String> {
    let app_dir = app.path().app_data_dir()
        .map_err(|e| e.to_string())?;
    
    fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;
    
    let state = WindowState { width, height, x, y };
    let state_json = serde_json::to_string(&state).map_err(|e| e.to_string())?;
    
    let state_path = app_dir.join("window_state.json");
    fs::write(state_path, state_json).map_err(|e| e.to_string())?;
    
    Ok(())
}

// 加载窗口状态
fn load_window_state(app: &tauri::AppHandle) -> Option<WindowState> {
    let app_dir = app.path().app_data_dir().ok()?;
    let state_path = app_dir.join("window_state.json");
    
    if state_path.exists() {
        let state_json = fs::read_to_string(state_path).ok()?;
        serde_json::from_str(&state_json).ok()
    } else {
        None
    }
}

// 关闭窗口
#[tauri::command]
fn close_window(window: tauri::Window) {
    window.hide().unwrap();
}

// 最小化窗口
#[tauri::command]
fn minimize_window(window: tauri::Window) -> Result<(), String> {
    window.minimize().map_err(|e| e.to_string())?;
    Ok(())
}

// 切换窗口置顶
#[tauri::command]
fn toggle_always_on_top(window: tauri::Window, flag: bool) -> Result<(), String> {
    window.set_always_on_top(flag).map_err(|e| e.to_string())?;
    Ok(())
}

// 保存语言设置
#[tauri::command]
fn save_language(app: tauri::AppHandle, language: String) -> Result<(), String> {
    let app_dir = app.path().app_data_dir()
        .map_err(|e| e.to_string())?;
    
    fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;
    
    let lang_path = app_dir.join("language.txt");
    fs::write(lang_path, language).map_err(|e| e.to_string())?;
    
    Ok(())
}

// 加载语言设置
#[tauri::command]
fn load_language(app: tauri::AppHandle) -> Result<String, String> {
    let app_dir = app.path().app_data_dir()
        .map_err(|e| e.to_string())?;
    
    let lang_path = app_dir.join("language.txt");
    
    if lang_path.exists() {
        fs::read_to_string(lang_path).map_err(|e| e.to_string())
    } else {
        // 默认英文
        Ok("en-US".to_string())
    }
}

// 启用开机自启动
#[tauri::command]
fn enable_autostart(app: tauri::AppHandle) -> Result<(), String> {
    app.autolaunch().enable()
        .map_err(|e| e.to_string())
}

// 禁用开机自启动
#[tauri::command]
fn disable_autostart(app: tauri::AppHandle) -> Result<(), String> {
    app.autolaunch().disable()
        .map_err(|e| e.to_string())
}

// 检查开机自启动状态
#[tauri::command]
fn is_autostart_enabled(app: tauri::AppHandle) -> Result<bool, String> {
    app.autolaunch().is_enabled()
        .map_err(|e| e.to_string())
}

// 获取应用语言（从配置文件读取，默认英文）
fn get_app_language(app: &tauri::AppHandle) -> String {
    if let Ok(app_dir) = app.path().app_data_dir() {
        let lang_file = app_dir.join("language.txt");
        if lang_file.exists() {
            if let Ok(lang) = fs::read_to_string(lang_file) {
                let lang = lang.trim().to_string();
                if !lang.is_empty() {
                    return lang;
                }
            }
        }
    }
    // 默认英文
    String::from("en-US")
}

// 构建托盘菜单
fn build_tray_menu(app: &tauri::AppHandle, current_lang: &str) -> Result<Menu<tauri::Wry>, tauri::Error> {
    let is_chinese = current_lang.starts_with("zh");
    
    let (show_text, lang_text, autostart_text, quit_text) = if is_chinese {
        ("显示窗口", "语言", "开机自启", "退出")
    } else {
        ("Show Window", "Language", "Auto Launch", "Quit")
    };
    
    // 语言子菜单项
    let zh_cn = MenuItem::with_id(app, "lang_zh", "简体中文", true, None::<&str>)?;
    let en_us = MenuItem::with_id(app, "lang_en", "English", true, None::<&str>)?;
    
    // 创建语言子菜单
    let lang_submenu = Submenu::with_items(app, lang_text, true, &[&zh_cn, &en_us])?;
    
    // 检查当前开机自启动状态
    let is_autostart_enabled = app.autolaunch().is_enabled().unwrap_or(false);
    
    // 主菜单项
    let show_item = MenuItem::with_id(app, "show", show_text, true, None::<&str>)?;
    let autostart_item = CheckMenuItemBuilder::with_id("autostart", autostart_text)
        .checked(is_autostart_enabled)
        .build(app)?;
    let quit_item = MenuItem::with_id(app, "quit", quit_text, true, None::<&str>)?;
    
    // 组装菜单
    Menu::with_items(app, &[&show_item, &autostart_item, &lang_submenu, &quit_item])
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, Some(vec![])))
    .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
      // 当用户尝试打开第二个实例时，显示并聚焦第一个实例的窗口
      if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
        let _ = window.unminimize();
      }
    }))
    .setup(|app| {
      // 读取保存的语言，默认英文
      let language = get_app_language(&app.handle());
      
      // 构建托盘菜单
      let menu = build_tray_menu(&app.handle(), &language)?;
      
      // 创建系统托盘图标（使用 with_id 指定 ID）
      let _tray = TrayIconBuilder::with_id("main")
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("QuickTask")
        .menu(&menu)
        .on_menu_event(|app, event| {
          // 处理菜单点击事件
          match event.id().as_ref() {
            "show" => {
              if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
              }
            }
            "lang_zh" => {
              // 切换到中文
              let _ = save_language(app.clone(), "zh-CN".to_string());
              // 重建托盘菜单
              if let Ok(new_menu) = build_tray_menu(&app, "zh-CN") {
                if let Some(tray) = app.tray_by_id("main") {
                  let _ = tray.set_menu(Some(new_menu));
                }
              }
              // 通知前端刷新
              if let Some(window) = app.get_webview_window("main") {
                let _ = window.eval("window.location.reload()");
              }
            }
            "lang_en" => {
              // 切换到英文
              let _ = save_language(app.clone(), "en-US".to_string());
              // 重建托盘菜单
              if let Ok(new_menu) = build_tray_menu(&app, "en-US") {
                if let Some(tray) = app.tray_by_id("main") {
                  let _ = tray.set_menu(Some(new_menu));
                }
              }
              // 通知前端刷新
              if let Some(window) = app.get_webview_window("main") {
                let _ = window.eval("window.location.reload()");
              }
            }
            "autostart" => {
              // 切换开机自启动
              let is_enabled = app.autolaunch().is_enabled().unwrap_or(false);
              if is_enabled {
                let _ = app.autolaunch().disable();
              } else {
                let _ = app.autolaunch().enable();
              }
              // 重建托盘菜单以更新复选框状态
              let language = get_app_language(&app);
              if let Ok(new_menu) = build_tray_menu(&app, &language) {
                if let Some(tray) = app.tray_by_id("main") {
                  let _ = tray.set_menu(Some(new_menu));
                }
              }
            }
            "quit" => {
              app.exit(0);
            }
            _ => {}
          }
        })
        .on_tray_icon_event(|tray, event| {
          // 处理左键点击显示窗口
          if let TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
          } = event
          {
            let app = tray.app_handle();
            if let Some(window) = app.get_webview_window("main") {
              let _ = window.show();
              let _ = window.set_focus();
            }
          }
        })
        .build(app)?;
      
      // 恢复窗口状态
      if let Some(window) = app.get_webview_window("main") {
        if let Some(state) = load_window_state(&app.handle()) {
          // 应用保存的窗口大小和位置
          let _ = window.set_size(tauri::Size::Physical(tauri::PhysicalSize {
            width: state.width,
            height: state.height,
          }));
          let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
            x: state.x,
            y: state.y,
          }));
        }
        
        // 监听窗口移动和调整大小事件，自动保存
        let app_handle = app.handle().clone();
        window.on_window_event(move |event| {
          if let tauri::WindowEvent::Resized(_) | tauri::WindowEvent::Moved(_) = event {
            if let Some(window) = app_handle.get_webview_window("main") {
              if let (Ok(size), Ok(position)) = (window.outer_size(), window.outer_position()) {
                let _ = save_window_state(
                  app_handle.clone(),
                  size.width,
                  size.height,
                  position.x,
                  position.y,
                );
              }
            }
          }
        });
      }
      
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      save_todos,
      load_todos,
      close_window,
      minimize_window,
      toggle_always_on_top,
      save_window_state,
      save_language,
      load_language,
      enable_autostart,
      disable_autostart,
      is_autostart_enabled
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
