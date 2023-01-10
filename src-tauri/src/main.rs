#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use chrono::prelude::*;
use tauri::{
    CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem,
    WindowEvent,
};
use tauri_plugin_autostart::MacosLauncher;
use user_idle::UserIdle;
use window_shadows::set_shadow;
use window_vibrancy::apply_mica;

#[derive(Clone, serde::Serialize)]
struct EmitPayload {
    data: String,
}

#[derive(Clone, serde::Serialize)]
struct PortfolioPayload {
    timestamp: String,
}

#[tauri::command]
async fn online_check(url: &str, token: &str, app: tauri::AppHandle) -> Result<(), ()> {
    let idle = UserIdle::get_time().unwrap();
    // send post request to server
    if idle.as_minutes() < 2 {
        let client = reqwest::Client::new();
        // get current time as &str
        let now = Local::now();
        let payload = PortfolioPayload {
            timestamp: now.to_string(),
        };
        let res = client
            .post(url)
            .header("Authorization", token)
            .json(&payload)
            .send()
            .await;
        if res.is_err() || res.unwrap().status() != 200 {
            let emit_payload = EmitPayload {
                data: format!(
                    "<span class='time-error'>{}</span> <span>Error sending data to server</span>",
                    now.to_rfc3339()
                ),
            };
            let _ = app.emit_all("online-check", emit_payload).unwrap();
            return Ok(());
        }
        let emit_payload = EmitPayload {
            data: format!(
                "<span class='time-success'>{}</span> <span>Data sent to server</span>",
                now.to_rfc3339()
            ),
        };
        let _ = app.emit_all("online-check", emit_payload).unwrap();
        return Ok(());
    }
    Ok(())
}

fn main() {
    let quit = CustomMenuItem::new("quit", "Quit");
    let hide = CustomMenuItem::new("hide", "hide");
    let tray_menu = SystemTrayMenu::new()
        .add_item(quit)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(hide);
    let tray = SystemTray::new().with_menu(tray_menu);
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .invoke_handler(tauri::generate_handler![online_check])
        .system_tray(tray)
        .setup(|app| {
            let id = app.listen_global("event-name", |event| {
                println!("got event-name with payload {:?}", event.payload());
            });
            app.unlisten(id);
            let window = app.get_window("main").unwrap();
            #[cfg(any(windows, target_os = "windows"))]
            set_shadow(&window, true).unwrap();

            #[cfg(target_os = "windows")]
            apply_mica(&window)
                .expect("Unsupported platform! 'apply_blur' is only supported on Windows");
            Ok(())
        })
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::DoubleClick { .. } => {
                let window = app.get_window("main").unwrap();
                window.show().unwrap();
            }
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "quit" => app.app_handle().exit(0),
                "hide" => {
                    let window = app.get_window("main").unwrap();
                    window.hide().unwrap();
                }
                _ => {}
            },
            _ => {}
        })
        .on_window_event(|event| match event.event() {
            WindowEvent::CloseRequested { api, .. } => {
                let window = event.window();
                window.hide().unwrap();
                api.prevent_close();
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
