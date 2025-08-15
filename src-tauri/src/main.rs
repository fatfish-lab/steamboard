// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
//#![allow(dead_code, unused_variables)]

mod app;
mod database;
mod steam;

use dotenv::dotenv;
use serde_json::Value;
use std::env;
use arboard::Clipboard;
use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager
};
use tauri_plugin_opener::OpenerExt;

use crate::app::SETTINGS;
use crate::app::{ErrorType};


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    // Create local data directory for steamboard
    let steamboard_dir = app::get_data_local_dir()
        .map_err(|e| {
            log::error!("Failed to generate local data path: {}", e);
            format!("Failed to generate local data path: {}", e)
        })?;

    let _ = std::fs::create_dir_all(&steamboard_dir)
        .map_err(|e| {
            log::error!("Failed to create steamboard local data directory: {}", e);
            format!("Failed to create steamboard local data directory: {}", e)
        })?;



    //tauri_app_lib::run();
    tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_log::Builder::new()
            .target(tauri_plugin_log::Target::new(
                tauri_plugin_log::TargetKind::Folder {
                    path: steamboard_dir.clone(),
                    file_name: Some("steamboard".to_string())
                }
            ))
            .max_file_size(50_000) /* bytes */
            .filter(|metadata| metadata.target().starts_with("Steamboard"))
            .build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .setup(|app| {
            #[cfg(desktop)]
            let _ = app.handle().plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
                let window = app.get_webview_window("main").expect("Main window not found");

                let _ = window.show();
                let _ = window.set_focus();
            }));

            // #[cfg(desktop)]
            // let _ = app.handle().plugin(tauri_plugin_updater::Builder::new().build());

            let platform = tauri_plugin_os::platform();
            let icon_as_template = platform == "macos";
            let icon_bytes: &[u8] = if icon_as_template {
                &include_bytes!("../icons/logo_template.png")[..]
            } else {
                &include_bytes!("../icons/logo.png")[..]
            };

            let app_handle = app.handle().clone();
            let show_item = MenuItem::with_id(app, "open", "Open", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let website_item = MenuItem::with_id(app, "website", "Official website", true, None::<&str>)?;
            let github_item = MenuItem::with_id(app, "github", "Github page", true, None::<&str>)?;
            let support_item = MenuItem::with_id(app, "support", "Support", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &website_item, &github_item, &support_item, &quit_item])?;
            let _ = TrayIconBuilder::with_id("main")
                .icon(Image::from_bytes(&icon_bytes).expect("Failed to load icon"))
                .icon_as_template(icon_as_template)
                .tooltip("Open Steamboard")
                .menu(&menu)
                .on_tray_icon_event(|tray, event| match event {
                    TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } => {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {}
                })
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "open" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "support" => {
                        let _ = app.opener().open_path("https://discord.gg/CatQEFsK", None::<&str>);
                    }
                    "website" => {
                        let _ = app.opener().open_path("https://steamboard.app/", None::<&str>);
                    }
                    "github" => {
                        let _ = app.opener().open_path("https://github.com/fatfish-lab/steamboard", None::<&str>);
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;
            on_setup(app_handle);
            Ok(())
        })
        .on_window_event(|window, event| {
            match event {
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    let _ = window.hide();
                    api.prevent_close();
                }
                _ => {}
            }
        })
        .invoke_handler(tauri::generate_handler![
            sync_command,
            export_csv_command,
            open_command,
            get_settings_command,
            set_settings_command,
            check_api_key_command,
            open_location_command,
            copy_to_clipboard_command,
            get_detailed_sales_command,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}


fn on_setup(app_handle: AppHandle) {
    tokio::spawn(async move {
        if let Err(e) = app::start(app_handle).await {
            log::error!("App start failed: {}", e);
        }
    });
}

// Helper wrapper for Tauri commands to handle errors and return JSON compatible errors
type ErrorJSON = Value;
async fn command_result<T, F>(fut: F) -> Result<T, ErrorJSON>
where
    F: std::future::Future<Output = Result<T, app::ErrorType>>,
{
    match fut.await {
        Ok(val) => Ok(val),
        Err(e) => Err(app::format_error_for_webview(&e)),
    }
}

#[tauri::command]
async fn sync_command(app_handle: AppHandle) -> Result<String, ErrorJSON> {
    command_result(async {
        let connection = database::open().await?;
        app::sync(&connection, &app_handle).await?;
        Ok(format!("Sync completed"))
    }).await
}


#[tauri::command]
async fn open_location_command(app_handle: AppHandle) -> Result<String, ErrorJSON> {
    command_result(async {
        let steamboard_dir = app::get_data_local_dir()
            .map_err(|e| app::ErrorType::Missing(format!("Failed to generate local data path: {}", e)))?;
        let _ = app_handle.opener().open_path(steamboard_dir.to_string_lossy(), None::<&str>);
        Ok(steamboard_dir.to_string_lossy().into_owned())
    }).await
}


#[tauri::command]
async fn open_command(path: String, app_handle: AppHandle) -> Result<String, ErrorJSON> {
    command_result(async {
        let _ = app_handle.opener().open_path(&path, None::<&str>);
        Ok(path)
    }).await
}


#[tauri::command]
async fn get_settings_command() -> Result<app::Settings, ErrorJSON> {
    command_result(async {
        let global_settings = SETTINGS.read().await;
        let settings = global_settings.clone();
        drop(global_settings);
        Ok(settings)
    }).await
}


#[tauri::command]
async fn set_settings_command(settings: app::Settings, app_handle: AppHandle) -> Result<String, ErrorJSON> {
    command_result(async {
        let mut global_settings = SETTINGS.write().await;
        global_settings.steam_api_key = settings.steam_api_key;
        global_settings.poll_interval = std::cmp::max(settings.poll_interval, Some(60));
        drop(global_settings);

        let connection = database::open().await?;
        if let Err(e) = database::save_settings(&connection).await {
            if let ErrorType::Missing(_) = e {
                app::create_password(&app_handle).await?;
                database::save_settings(&connection).await?;
            }
        }

        Ok("Settings saved".into())
    }).await
}


#[tauri::command]
async fn check_api_key_command(steam: String) -> Result<String, ErrorJSON> {
    command_result(async {
        let res = steam::check_api_key(Some(steam)).await?;
        Ok(res)
    }).await
}


#[tauri::command]
async fn get_detailed_sales_command(from_date: Option<String>, to_date: Option<String>) -> Result<Vec<steam::CPartnerFinancialsDetailedSalesResult>, ErrorJSON> {
    command_result(async {
        let connection = database::open().await?;
        let res = database::get_sale_details_by_date(&connection, from_date, to_date).await?;
        Ok(res)
    }).await
}


#[tauri::command]
async fn export_csv_command(path: String, from_date: Option<String>, to_date: Option<String>, delimiter: String) -> Result<String, ErrorJSON> {
    command_result(async {
        let connection = database::open().await?;
        let rows = database::get_sale_details_by_date(&connection, from_date, to_date).await?;
        let delimiter_byte = delimiter.bytes().next().unwrap_or(b',');
        let result = app::export_to_csv(rows, path, delimiter_byte).await?;
        Ok(result)
    }).await
}


#[tauri::command]
async fn copy_to_clipboard_command(from_date: Option<String>, to_date: Option<String>, delimiter: String) -> Result<String, ErrorJSON> {
    command_result(async {
        let connection = database::open().await?;
        let rows = database::get_sale_details_by_date(&connection, from_date, to_date).await?;
        let delimiter_byte = delimiter.bytes().next().unwrap_or(b',');
        let csv_string = app::write_csv(&rows, delimiter_byte)?;
        let mut clipboard = Clipboard::new().map_err(|e| app::ErrorType::BadFormatting(format!("Clipboard error: {}", e)))?;
        clipboard.set_text(csv_string).map_err(|e| app::ErrorType::BadFormatting(format!("Clipboard error: {}", e)))?;
        Ok("Copied to system clipboard!".into())
    }).await
}