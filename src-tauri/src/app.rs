use crate::database;
use crate::steam;
use std::fmt;
use std::io::Write;
use aes_gcm::aead::rand_core::RngCore;
use tauri::{AppHandle, Emitter};
use futures::stream::{self, StreamExt};
use tauri_plugin_notification::NotificationExt;
use std::time::{ Instant, Duration };
use tokio_rusqlite::Connection;
use tokio::task;
use tokio::time::sleep;
use once_cell::sync::Lazy;
use tokio::sync::RwLock;
use std::sync::Arc;
use std::path::Path;
use serde::{Serialize, Deserialize};
use serde_json::json;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use aes_gcm::aead::{Aead, OsRng};
use argon2::{Argon2, PasswordHasher};
use argon2::password_hash::SaltString;
use base64::{engine::general_purpose, Engine as _};
use keyring;
use directories::{ProjectDirs};



#[derive(Serialize, Clone)]
pub enum ErrorType {
    Missing(String),
    BadHttpRequest(String),
    BadToken(String),
    BadRequest(String),
    BadFormatting(String),
    Forbidden(String),
}

impl fmt::Display for ErrorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorType::Missing(msg) => write!(f, "[Missing] {}", msg),
            ErrorType::BadHttpRequest(msg) => write!(f, "[Bad HTTP Request] {}", msg),
            ErrorType::BadToken(msg) => write!(f, "[Bad Token] {}", msg),
            ErrorType::BadRequest(msg) => write!(f, "[Bad Request] {}", msg),
            ErrorType::BadFormatting(msg) => write!(f, "[Bad Formatting] {}", msg),
            ErrorType::Forbidden(msg) => write!(f, "[Forbidden] {}", msg),
        }
    }
}

// Helper to format error in compatible JS Error format
pub fn format_error_for_webview(error: &ErrorType) -> serde_json::Value {
    match error {
        ErrorType::Missing(msg) => json!({ "type": "Missing", "name": "Something is missing" ,"message": msg }),
        ErrorType::BadHttpRequest(msg) => json!({ "type": "BadHttpRequest", "name": "A HTTP error occurred" ,"message": msg }),
        ErrorType::BadToken(msg) => json!({ "type": "BadToken", "name": "Your API key is not valid" ,"message": msg }),
        ErrorType::BadRequest(msg) => json!({ "type": "BadRequest", "name": "An error occurred with the database" ,"message": msg }),
        ErrorType::BadFormatting(msg) => json!({ "type": "BadFormatting", "name": "The data format is incorrect" ,"message": msg }),
        ErrorType::Forbidden(msg) => json!({ "type": "Forbidden", "name": "Something can't be processed" ,"message": msg }),
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Settings {
    pub id: i32,
    pub steam_api_key: Option<String>,
    pub poll_interval: Option<i32>,
    pub highwatermark: Option<String>,
}

// Wrap the global in `Arc` so it's clonable and usable in tasks.
pub static SETTINGS: Lazy<Arc<RwLock<Settings>>> = Lazy::new(|| {
    Arc::new(RwLock::new(Settings {
        id: 0,
        steam_api_key: None,
        poll_interval: Some(600),
        highwatermark: Some(String::from("0")),
    }))
});

pub async fn set_settings(connection: &Connection, app_handle: &AppHandle) -> Result<(), ErrorType> {
    let sets = database::get_settings(&connection).await;

    match sets {
        Ok(sets) => {
            let mut settings = SETTINGS.write().await;
            settings.id = sets.id;
            settings.steam_api_key = sets.steam_api_key;
            settings.poll_interval = sets.poll_interval;
            settings.highwatermark = sets.highwatermark;
            // steam::check_api_key(settings.steam_api_key.clone()).await?;
            let _ = app_handle.emit("settings-updated", settings.clone());
        },
        Err(e) => {
            if let ErrorType::BadToken(_) = e {
                app_handle.emit("decryption-failed", format_error_for_webview(&e)).unwrap();
            }
            return Err(e);
        }
    }
    // {
    //     let mut settings = SETTINGS.write().await;
    //     settings.id = sets.id;
    //     settings.steam_api_key = sets.steam_api_key;
    //     settings.poll_interval = sets.poll_interval;
    //     settings.highwatermark = sets.highwatermark;
    //     //steam::check_api_key(settings.steam_api_key.clone()).await?;
    //     let _ = app_handle.emit("settings-updated", settings.clone());
    // }

    Ok(())
}

pub async fn create_password(app_handle: &AppHandle) -> Result<(), ErrorType> {
    let _password = match get_password().await {
        Ok(p) => p,
        Err(e) => {
            // Check if the error is ErrorType::Forbidden (indicating user denied access)
            if let ErrorType::Forbidden(_) = e {
                app_handle.emit("denied-keyring-access", format_error_for_webview(&e)).unwrap();
                return Err(e);
            } else {
                // Create a default password if not found
                log::error!("Failed to get password: {}", e);
                let mut default_password = [0u8; 16];
                OsRng.fill_bytes(&mut default_password);
                let encoded_password = general_purpose::STANDARD.encode(&default_password);
                set_password(&encoded_password).await?;
                log::info!("Default password set in keyring");
                encoded_password
            }
        },
    };

    Ok(())
}


pub async fn start(app_handle: AppHandle) -> Result<String, ErrorType> {
    log::info!("Starting Steamboard - v{}", app_handle.package_info().version);

    let connection = database::create(app_handle.clone()).await?;

    let has_settings = database::has_settings(&connection).await?;
    if has_settings {
        // Set steam_api_key to string to indicate to the frontend that it is set
        let mut settings = SETTINGS.write().await;
        if settings.steam_api_key.is_none() {
            settings.steam_api_key = Some(String::new());
        }
        drop(settings);

        create_password(&app_handle).await?;
        set_settings(&connection, &app_handle).await?;
        tokio::spawn(async move {
            if let Err(e) = periodic_sync(&connection, &app_handle).await {
                log::error!("Periodic sync failed: {}", e);
            }
        });
    }

    Ok("Loaded".into())
}


pub async fn periodic_sync(connection: &Connection, app_handle: &AppHandle)-> Result<(), ErrorType> {
    loop {
        let settings = SETTINGS.read().await;
        let poll_interval = settings.poll_interval.unwrap_or(600);
        log::info!("Starting periodic sync with interval: {} seconds", poll_interval);
        drop(settings);
        sync(&connection, &app_handle).await?;
        sleep(Duration::from_secs(poll_interval as u64)).await;
    }
}


pub async fn sync(connection: &Connection, app_handle: &AppHandle) -> Result<(), ErrorType> {
    log::info!("Starting sync...");
    app_handle.emit("sync-progress", 0).unwrap();

    let started_at = Instant::now();

    let settings = SETTINGS.read().await;
    let mut steam_api_key = settings.steam_api_key.clone();
    let mut highwatermark = settings.highwatermark.clone();
    drop(settings);

    // If no API key is set, try to reload settings with initialization
    if steam_api_key.is_none() || steam_api_key.as_ref().unwrap().is_empty() {
        app_handle.emit("sync-progress", 1).unwrap();

        let _ = create_password(&app_handle).await;
        let settings = SETTINGS.read().await;
        steam_api_key = settings.steam_api_key.clone();
        highwatermark = settings.highwatermark.clone();
        drop(settings);

        if steam_api_key.is_none() || steam_api_key.as_ref().unwrap().is_empty() {
            return Err(ErrorType::BadToken("API key is still missing after start".into()));
        }
    }

    let first_sync = highwatermark.is_none() || highwatermark.as_ref().unwrap() == "0";

    let changed_dates = steam::get_changed_dates_for_partner(steam_api_key.clone(), highwatermark.clone()).await?;
    let mut dates = changed_dates.dates.unwrap_or(vec![]);
    dates.reverse();

    let max_concurrent_dates = 10;
    let dates_for_stream = dates.clone();
    let dates_len = dates_for_stream.len();

    let sales_details: Vec<Vec<steam::CPartnerFinancialsDetailedSalesResult>> = stream::iter(dates_for_stream.into_iter().enumerate())
        .map(|(i, date)| {
            let steam_api_key = steam_api_key.clone();
            let conn = connection.clone();
            app_handle.emit("sync-progress", (i as f32) / (dates_len as f32)).unwrap();
            async move {
                sync_one_date(conn, steam_api_key, date).await
            }
        })
        .buffer_unordered(max_concurrent_dates)
        .filter_map(|res| async {
            match res {
                Ok(r) => {
                    let _ = app_handle.emit("sync-data", &r);
                    Some(r)
                }
                Err(e) => {
                    log::error!("Error syncing date: {}", e);
                    None
                }
            }
        })
        .collect()
        .await;

    // Flatten the Vec<Vec<_>> into Vec<_>
    let all_sales_details: Vec<steam::CPartnerFinancialsDetailedSalesResult> = sales_details.into_iter().flatten().collect();

    let mut settings = SETTINGS.write().await;
    settings.highwatermark = Some(changed_dates.result_highwatermark);
    drop(settings);

    let _ = database::save_settings(&connection).await?;

    app_handle.emit("sync-progress", 1).unwrap();
    log::info!("Sync done in {:?}, {} sales details added", started_at.elapsed(), all_sales_details.len());

    if first_sync {
        log::info!("Sending notification: Initial sync completed successfully!");
        app_handle.notification()
            .builder()
            .title("Sync completed")
            .body("Initial sync completed successfully!")
            .show()
            .unwrap();
    } else {
        // Notify user about new sales
        let sales_amount = all_sales_details.len();
        let summary = if sales_amount == 1 {
            format!("{} new sale", sales_amount)
        } else {
            format!("{} new sales", sales_amount)
        };

        let body = if sales_amount < 10 {
            format!("Way to go!")
        } else if sales_amount < 50 {
            format!("Well done!")
        } else if sales_amount < 50 {
            format!("Wow, that's a lot!")
        } else if sales_amount < 100 {
            format!("You are a sale machine!")
        } else if sales_amount < 200 {
            format!("You're on fire!")
        } else if sales_amount < 500 {
            format!("Ka-ching! You are a sale monster!")
        } else {
            format!("Jackpot!")
        };

        if sales_amount > 0 {
            log::info!("Sending notification: {} - {}", summary, body);
            app_handle.notification()
                .builder()
                .title(&summary)
                .body(&body)
                .show()
                .unwrap();
        }
    }

    Ok(())
}


async fn sync_one_date(conn: Connection, api_key: Option<String>, date: String) -> Result<Vec<steam::CPartnerFinancialsDetailedSalesResult>, ErrorType> {
    let mut highwatermark = database::get_highwatermark(&conn, date.to_string()).await?;
    let mut incoming_sales_details = Vec::new();
    log::info!("Syncing {} (watermark: {})...", date, highwatermark);
    if highwatermark == 0 {
        log::info!("Deleting if existing sale details for {}", date);
        let _ = database::delete_sale_detail(&conn, date.to_string()).await?;
    }
    loop {
        let date_detailed_sales = steam::get_detailed_sales(api_key.clone(), &date, highwatermark).await?;
        let max_id = date_detailed_sales.max_id.parse::<i64>().map_err(|e| ErrorType::BadFormatting(format!("Could not parse max id: {}", e)))?;
        if max_id == highwatermark {
            break;
        }
        highwatermark = max_id;
        let mut inserted_sales_details = database::insert_sale_detail(&conn, date_detailed_sales).await?;
        log::info!("Inserted {} sales details for {}", inserted_sales_details.len(), date);
        incoming_sales_details.append(&mut inserted_sales_details);
    }
    database::insert_sale_date(&conn, date.to_string(), highwatermark.to_string()).await?;

    Ok(incoming_sales_details)
}


pub fn write_csv<T>(data: &[T], delimiter: u8) -> Result<String, ErrorType> where T: serde::Serialize,{
    let mut buffer = Vec::new();
    {
        let mut wtr = csv::WriterBuilder::new()
            .delimiter(delimiter)
            .from_writer(&mut buffer);

        for record in data.iter() {
            wtr.serialize(record)
                .map_err(|e| ErrorType::BadFormatting(format!("CSV serialize error: {}", e)))?;
        }

        wtr.flush()
            .map_err(|e| ErrorType::BadFormatting(format!("CSV flush error: {}", e)))?;
    }
    String::from_utf8(buffer)
        .map_err(|e| ErrorType::BadFormatting(format!("UTF-8 conversion error: {}", e)))
}


pub async fn export_to_csv(data: Vec<steam::CPartnerFinancialsDetailedSalesResult>, path: String, delimiter: u8) -> Result<String, ErrorType> {
    let path_clone = path.clone();
    let _ = task::spawn_blocking(move || -> Result<(), ErrorType> {
        let mut file = std::fs::File::create(Path::new(&path_clone)).map_err(|e| ErrorType::BadFormatting(format!("File error: {}", e)))?;
        let csv_string = write_csv(&data, delimiter)?;
        file.write_all(csv_string.as_bytes()).map_err(|e| ErrorType::BadFormatting(format!("Error while writing to file: {}", e)))?;
        Ok(())
    })
    .await
    .map_err(|e| ErrorType::BadFormatting(format!("Error while creating the CSV file: {}", e)))??;

    Ok(format!("Csv file exported to {}", &path))
}


/// Derive a 256-bit AES key from password and salt
fn derive_key_from_password(password: &str, salt: &[u8]) -> Result<[u8; 32], ErrorType> {
    let salt_string = SaltString::encode_b64(salt)
        .map_err(|e| ErrorType::Forbidden(format!("Invalid base64 salt: {}", e)))?;
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt_string)
        .map_err(|e| ErrorType::Forbidden(format!("Password hashing failed: {}", e)))?;

    let raw_hash = password_hash.hash.ok_or("Missing hash")
        .map_err(|e| ErrorType::Forbidden(format!("Encryption error: {}", e)))?;
    let mut key = [0u8; 32];
    key.copy_from_slice(raw_hash.as_bytes());
    Ok(key)
}


/// Encrypt a plaintext string using password-derived AES-GCM key
pub fn encrypt(password: &str, plaintext: &str) -> Result<String, ErrorType> {
    let mut salt = [0u8; 16];
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut salt);
    OsRng.fill_bytes(&mut nonce_bytes);

    let key = derive_key_from_password(password, &salt)?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| ErrorType::Forbidden(format!("AES init failed: {}", e)))?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher.encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| ErrorType::Forbidden(format!("encryption failed: {}", e)))?;

    let ciphertext = general_purpose::STANDARD.encode(&ciphertext);
    let salt = general_purpose::STANDARD.encode(&salt);
    let nonce_bytes = general_purpose::STANDARD.encode(&nonce_bytes);
    let encrypted = vec![ciphertext, salt, nonce_bytes].join(",");
    Ok(encrypted)
}


/// Decrypt AES-GCM ciphertext using password and stored salt + nonce
pub fn decrypt(password: &str, encrypted: &str) -> Result<String, ErrorType> {
    let parts = encrypted.split(',').collect::<Vec<&str>>();
    if parts.len() != 3 {
        return Err(ErrorType::Forbidden("Invalid encrypted format".into()));
    }
    let ciphertext_b64 = parts[0];
    let salt_b64 = parts[1];
    let nonce_b64 = parts[2];
    let salt = general_purpose::STANDARD.decode(salt_b64)
        .map_err(|e| ErrorType::Forbidden(format!("Invalid base64 salt: {}", e)))?;
    let nonce_bytes = general_purpose::STANDARD.decode(nonce_b64)
        .map_err(|e| ErrorType::Forbidden(format!("Invalid base64 nonce: {}", e)))?;
    let ciphertext = general_purpose::STANDARD.decode(ciphertext_b64)
        .map_err(|e| ErrorType::Forbidden(format!("Invalid base64 ciphertext: {}", e)))?;
    let key = derive_key_from_password(password, &salt)?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| ErrorType::Forbidden(format!("AES init failed: {}", e)))?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    let decrypted = cipher.decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| ErrorType::BadToken(format!("Decryption failed: {}", e)))?;
    let plaintext = String::from_utf8(decrypted)
        .map_err(|e| ErrorType::Forbidden(format!("UTF-8 conversion error: {}", e)))?;
    Ok(plaintext)
}


pub async fn get_password() -> Result<String, ErrorType> {
    let keyring = keyring::Entry::new("Steamboard", "User")
        .map_err(|e| ErrorType::Forbidden(format!("Failed to create keyring entry: {}", e)))?;
    // Use spawn_blocking since keyring operations may block
    tokio::task::spawn_blocking(move || {
        keyring.get_password()
            .map_err(|e| {
                if e.to_string().contains("No matching entry") {
                    ErrorType::Missing(format!("No password found in keyring: {}", e))
                } else {
                    ErrorType::Forbidden(format!("Failed to get password from keyring: {}", e))
                }
            })
    })
    .await
    .map_err(|e| ErrorType::Forbidden(format!("Failed to get password from keyring: {}", e)))?
}


pub async fn set_password(password: &str) -> Result<(), ErrorType> {
    let keyring = keyring::Entry::new("Steamboard", "User")
        .map_err(|e| ErrorType::Forbidden(format!("Failed to create keyring entry: {}", e)))?;
    // Clone password to String so it can be moved into the closure
    let password_owned = password.to_owned();
    tokio::task::spawn_blocking(move || {
        keyring.set_password(&password_owned)
            .map_err(|e| ErrorType::Forbidden(format!("Failed to set password in keyring: {}", e)))
    })
    .await
    .map_err(|e| ErrorType::Forbidden(format!("Failed to set password in keyring: {}", e)))?
}

pub fn get_data_local_dir() -> Result<std::path::PathBuf, ErrorType> {
    let Some(steamboard_dirs) = ProjectDirs::from("com", "fatfishlab", "steamboard") else {
        return Err(ErrorType::Missing("Could not find local data directory".to_string()));
    };

    Ok(steamboard_dirs.data_local_dir().to_path_buf())
}