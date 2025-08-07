use crate::app;
use tauri::{
    AppHandle,
    Manager,
    path::BaseDirectory
};
use tokio_rusqlite::{params, Connection};
use crate::app::{ErrorType, Settings};
use crate::steam::DetailedSales;
use crate::steam::CPartnerFinancialsDetailedSalesResult;


pub async fn open() -> Result<Connection, ErrorType> {
    let steamboard_dir = app::get_data_local_dir()
        .map_err(|e| ErrorType::Missing(format!("Failed to generate local data path: {}", e)))?;
    let db_path = steamboard_dir.join("steamboard.db");

    let connection = Connection::open(db_path)
        .await
        .map_err(|e| ErrorType::BadRequest(e.to_string()))?;

    Ok(connection)
}


pub async fn create(app_handle: AppHandle) -> Result<Connection, ErrorType> {
    let sql_path = app_handle.path().resolve("steamboard.sql", BaseDirectory::Resource)
        .map_err(|e| ErrorType::Missing(format!("Failed to resolve path to steamboard.sql: {}", e)))?;

    log::info!("Using SQL file at: {}", sql_path.display());

    let steamboard_sql = std::fs::read_to_string(sql_path)
        .map_err(|e| ErrorType::Missing(format!("Failed to read steamboard.sql: {}", e)))?;

    let connection = open().await?;

    connection.call(move |conn| {
        conn.execute_batch(&steamboard_sql)?;
        conn.execute("INSERT INTO settings (id) VALUES (0) ON CONFLICT(id) DO NOTHING", params![])?;
        conn.execute("INSERT INTO steam_key_request_info (key_request_id) VALUES (0) ON CONFLICT(key_request_id) DO NOTHING", params![])?;
        Ok(())
    })
    .await
    .map_err(|e| ErrorType::BadRequest(format!("Failed to create the database: {}", e)))?;

    Ok(connection)
}


pub async fn has_settings(connection: &Connection) -> Result<bool, ErrorType> {
    let count = connection.call(|conn| {
        let mut stmt = conn.prepare("SELECT COUNT(*) FROM settings WHERE id = 0 AND steam_api_key IS NOT NULL")?;
        let count: i64 = stmt.query_row([], |row| row.get(0))?;
        Ok(count)
    })
    .await
    .map_err(|e| ErrorType::BadRequest(format!("Failed to check settings existence: {}", e)))?;

    Ok(count > 0)
}


pub async fn get_settings(connection: &Connection) -> Result<Settings, ErrorType> {
    let password = app::get_password().await?;
    let mut settings = connection.call(|conn| {
        let settings = conn.query_row("SELECT * FROM settings WHERE id = 0",
            params![],
            |row| {
                Ok(Settings {
                    id: row.get(0)?,
                    steam_api_key: row.get(1)?,
                    poll_interval: row.get(2)?,
                    highwatermark: row.get(3)?,
                })
            })?;
        Ok(settings)
    })
    .await
    .map_err(|e| ErrorType::BadRequest(format!("Failed to query settings: {}", e)))?;

    // Decrypt the API key if it exists
    if let Some(encrypted_api_key) = &settings.steam_api_key {
        let decrypted_key = app::decrypt(&password, encrypted_api_key)?;
        settings.steam_api_key = Some(decrypted_key);
    } else {
        settings.steam_api_key = Some(String::new());
    }

    Ok(settings)
}

pub async fn save_settings(connection: &Connection) -> Result<(), ErrorType> {
    let password = app::get_password().await?;

    if password.is_empty() {
        return Err(ErrorType::Missing("No password set for encryption".to_string()));
    }

    let settings = app::SETTINGS.read().await;
    let mut encrypted_key = settings.steam_api_key.clone();

    // Encrypt the API key before saving
    if let Some(api_key) = &settings.steam_api_key {
        encrypted_key = Some(app::encrypt(&password, api_key)?);
    }

    connection.call(move |conn| {
        conn.execute("UPDATE settings SET steam_api_key = ?2, poll_interval = ?3, highwatermark = ?4 WHERE id = ?1",
        params![
            settings.id.clone(),
            encrypted_key,
            settings.poll_interval.clone(),
            settings.highwatermark.clone()
        ])?;
        Ok(())
    })
    .await
    .map_err(|e| ErrorType::BadRequest(format!("Failed to update highwatermark: {}", e)))?;

    Ok(())
}


pub async fn get_highwatermark(connection: &Connection, date: String) -> Result<i64, ErrorType> {
    let sale_dates = connection.call(|conn| {
        let mut stmt = conn.prepare("SELECT highwatermark_id FROM steam_dates WHERE date = ?1")?;
        let sale_dates_iter = stmt.query_map([date], |row| row.get(0))?;
        let mut sale_dates = Vec::new();

        for date in sale_dates_iter {
            sale_dates.push(date?);
        }

        Ok(sale_dates)
    })
    .await
    .map_err(|e| ErrorType::BadRequest(format!("failed to get highwatermark: {}", e)))?;

    if sale_dates.len() == 0 {
        Ok(0)
    }
    else {
        Ok(sale_dates[0])
    }
}


pub async fn delete_sale_detail(connection: &Connection, date: String) -> Result<(), ErrorType> {
    connection.call(move |conn| {
        conn.execute(
            "DELETE FROM steam_results WHERE date = ?1",
            params![date])?;
        Ok(())
    })
    .await
    .map_err(|e| ErrorType::BadRequest(format!("failed to delete sale date: {}", e)))?;

    Ok(())
}


pub async fn insert_sale_date(connection: &Connection, date: String, highwatermark: String) -> Result<(), ErrorType> {
    connection.call(move |conn| {
        conn.execute(
            "INSERT INTO steam_dates (date, highwatermark_id) VALUES (?1, ?2) ON CONFLICT (date) DO UPDATE SET highwatermark_id = ?2",
            params![date, highwatermark])?;
        Ok(())
    })
    .await
    .map_err(|e| ErrorType::BadRequest(format!("failed to insert sale date: {}", e)))?;

    Ok(())
}


pub async fn insert_sale_detail(connection: &Connection, detailed_sales: DetailedSales) -> Result<Vec<CPartnerFinancialsDetailedSalesResult>, ErrorType> {
    let detail = detailed_sales;
    let aggregated_sales_details = connection.call(move |conn| {
        let mut aggregated_sales_details = Vec::new();
        for key_request_info in detail.key_request_info.iter().flatten() {
            conn.execute(
                "
                    INSERT INTO steam_key_request_info (
                        key_request_id,
                        key_request_notes,
                        game_code_id,
                        game_code_description,
                        territory_code_id,
                        territory_code_description
                    )
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                    ON CONFLICT (key_request_id) DO
                    UPDATE SET
                        key_request_notes = ?2,
                        game_code_id = ?3,
                        game_code_description = ?4,
                        territory_code_id = ?5,
                        territory_code_description = ?6
                ",
                params![
                    key_request_info.key_request_id,
                    key_request_info.key_request_notes,
                    key_request_info.game_code_id,
                    key_request_info.game_code_description,
                    key_request_info.territory_code_id,
                    key_request_info.territory_code_description,
                ]
            )?;
        }

        for package_info in detail.package_info.iter().flatten() {
            conn.execute(
                "
                    INSERT INTO steam_package_info (
                        packageid,
                        package_name
                    )
                    VALUES (?1, ?2)
                    ON CONFLICT (packageid) DO
                    UPDATE SET
                        package_name = ?2
                ",
                params![
                    package_info.packageid,
                    package_info.package_name,
                ]
            )?;
        }

        for app_info in detail.app_info.iter().flatten() {
            conn.execute(
                "
                    INSERT INTO steam_steam_app_info (
                        appid,
                        app_name
                    )
                    VALUES (?1, ?2)
                    ON CONFLICT (appid) DO
                    UPDATE SET
                        app_name = ?2
                ",
                params![
                    app_info.appid,
                    app_info.app_name,
                ]
            )?;
        }

        for bundle_info in detail.bundle_info.iter().flatten() {
            conn.execute(
                "
                    INSERT INTO steam_bundle_info (
                        bundleid,
                        bundle_name
                    )
                    VALUES (?1, ?2)
                    ON CONFLICT (bundleid) DO
                    UPDATE SET
                        bundle_name = ?2
                ",
                params![
                    bundle_info.bundleid,
                    bundle_info.bundle_name,
                ]
            )?;
        }

        for discount_info in detail.discount_info.iter().flatten() {
            conn.execute(
                "
                    INSERT INTO steam_discount_info (
                        discountid,
                        discount_description,
                        discount_group,
                        discount_percentage
                    )
                    VALUES (?1, ?2, ?3, ?4)
                    ON CONFLICT (discountid) DO
                    UPDATE SET
                        discount_description = ?2,
                        discount_group = ?3,
                        discount_percentage = ?4
                ",
                params![
                    discount_info.discountid,
                    discount_info.discount_description,
                    discount_info.discount_group,
                    discount_info.discount_percentage,
                ]
            )?;
        }

        for combined_discount_info in detail.combined_discount_info.iter().flatten() {
            conn.execute(
                "
                    INSERT INTO steam_combined_discount_info (
                        combined_discount_id,
                        combined_discount_name,
                        total_discount_percentage,
                        discount_ids
                    )
                    VALUES (?1, ?2, ?3, ?4)
                    ON CONFLICT (combined_discount_id) DO
                    UPDATE SET
                        combined_discount_name = ?2,
                        total_discount_percentage = ?3,
                        discount_ids = ?4
                ",
                params![
                    combined_discount_info.combined_discount_id,
                    combined_discount_info.combined_discount_name,
                    combined_discount_info.total_discount_percentage,
                    serde_json::to_string(&combined_discount_info.discount_ids).unwrap(),
                ],
            )?;
        }

        for game_item_info in detail.game_item_info.iter().flatten() {
            conn.execute(
                "
                    INSERT INTO steam_game_item_info (
                        appid,
                        game_item_id,
                        game_item_description,
                        game_item_category
                    )
                    VALUES (?1, ?2, ?3, ?4)
                    ON CONFLICT (appid) DO
                    UPDATE SET
                        game_item_description = ?3,
                        game_item_category = ?4
                ",
                params![
                    game_item_info.appid,
                    game_item_info.game_item_id,
                    game_item_info.game_item_description,
                    game_item_info.game_item_category,
                ],
            )?;
        }

        for country_info in detail.country_info.iter().flatten() {
            conn.execute(
                "
                    INSERT INTO steam_country_info (
                        country_code,
                        country_name,
                        region
                    )
                    VALUES (?1, ?2, ?3)
                    ON CONFLICT (country_code) DO
                    UPDATE SET
                        country_name = ?2,
                        region = ?3
                ",
                params![
                    country_info.country_code,
                    country_info.country_name,
                    country_info.region,
                ]
            )?;
        }

        for partner_info in detail.partner_info.iter().flatten() {
            conn.execute(
                "
                    INSERT INTO steam_partner_info (
                        partnerid,
                        partner_name
                    )
                    VALUES (?1, ?2)
                    ON CONFLICT (partnerid) DO
                    UPDATE SET
                        partner_name = ?2
                ",
                params![
                    partner_info.partnerid,
                    partner_info.partner_name,
                ]
            )?;
        }

        if let Some(results) = &detail.results {
            for result in results.iter() {
                conn.execute(
                    "
                        INSERT INTO steam_results (
                            partnerid,
                            date,
                            line_item_type,
                            packageid,
                            bundleid,
                            appid,
                            game_item_id,
                            package_sale_type,
                            key_request_id,
                            platform,
                            country_code,
                            base_price,
                            sale_price,
                            currency,
                            gross_units_sold,
                            gross_units_returned,
                            gross_sales_usd,
                            gross_returns_usd,
                            net_tax_usd,
                            gross_units_activated,
                            view_grant_partnerid,
                            net_units_sold,
                            net_sales_usd,
                            avg_sale_price_usd,
                            combined_discount_id,
                            primary_appid,
                            additional_revenue_share_tier
                        )
                        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27)
                    ",
                    params![
                        result.partnerid,
                        result.date,
                        result.line_item_type,
                        result.packageid,
                        result.bundleid,
                        result.appid,
                        result.game_item_id,
                        result.package_sale_type,
                        result.key_request_id,
                        result.platform,
                        result.country_code,
                        result.base_price,
                        result.sale_price,
                        result.currency,
                        result.gross_units_sold,
                        result.gross_units_returned,
                        result.gross_sales_usd,
                        result.gross_returns_usd,
                        result.net_tax_usd,
                        result.gross_units_activated,
                        result.view_grant_partnerid,
                        result.net_units_sold,
                        result.net_sales_usd,
                        result.avg_sale_price_usd,
                        result.combined_discount_id,
                        result.primary_appid,
                        result.additional_revenue_share_tier,
                    ],
                )?;

                let mut details = CPartnerFinancialsDetailedSalesResult::default();

                details.partnerid = result.partnerid.clone();
                details.date = result.date.clone();
                details.line_item_type = result.line_item_type.clone();
                details.packageid = result.packageid.clone();
                details.bundleid = result.bundleid.clone();
                details.appid = result.appid.clone();
                details.game_item_id = result.game_item_id.clone();
                details.package_sale_type = result.package_sale_type.clone();
                details.key_request_id = result.key_request_id.clone();
                details.platform = result.platform.clone();
                details.country_code = result.country_code.clone();
                details.base_price = result.base_price.clone();
                details.sale_price = result.sale_price.clone();
                details.currency = result.currency.clone();
                details.gross_units_sold = result.gross_units_sold.clone();
                details.gross_units_returned = result.gross_units_returned.clone();
                details.gross_sales_usd = result.gross_sales_usd.clone();
                details.gross_returns_usd = result.gross_returns_usd.clone();
                details.net_tax_usd = result.net_tax_usd.clone();
                details.gross_units_activated = result.gross_units_activated.clone();
                details.view_grant_partnerid = result.view_grant_partnerid.clone();
                details.net_units_sold = result.net_units_sold.clone();
                details.net_sales_usd = result.net_sales_usd.clone();
                details.avg_sale_price_usd = result.avg_sale_price_usd.clone();
                details.combined_discount_id = result.combined_discount_id.clone();
                details.primary_appid = result.primary_appid.clone();
                details.additional_revenue_share_tier = result.additional_revenue_share_tier.clone();

                if let Some(p) = detail.partner_info.as_ref().unwrap_or(&Vec::new()).iter().find(|x| x.partnerid == result.partnerid) {
                    details.partner_name = p.partner_name.clone();
                }

                if let Some(p) = detail.package_info.as_ref().unwrap_or(&Vec::new()).iter().find(|x| x.packageid == result.packageid.unwrap_or(0)) {
                    details.package_name = p.package_name.clone();
                }

                if let Some(p) = detail.bundle_info.as_ref().unwrap_or(&Vec::new()).iter().find(|x| x.bundleid == result.bundleid.unwrap_or(0)) {
                    details.bundle_name = p.bundle_name.clone();
                }

                if let Some(p) = detail.app_info.as_ref().unwrap_or(&Vec::new()).iter().find(|x| x.appid == result.appid.unwrap_or(0)) {
                    details.app_name = p.app_name.clone();
                }

                if let Some(p) = detail.game_item_info.as_ref().unwrap_or(&Vec::new()).iter().find(|x| x.game_item_id == result.game_item_id) {
                    details.game_item_description = p.game_item_description.clone();
                    details.game_item_category = p.game_item_category.clone();
                }

                if let Some(p) = detail.key_request_info.as_ref().unwrap_or(&Vec::new()).iter().find(|x| x.key_request_id == result.key_request_id.unwrap_or(0)) {
                    details.key_request_notes = p.key_request_notes.clone();
                    details.game_code_id = p.game_code_id.clone();
                    details.game_code_description = p.game_code_description.clone();
                    details.territory_code_id = p.territory_code_id.clone();
                    details.territory_code_description = p.territory_code_description.clone();
                }

                if let Some(p) = detail.country_info.as_ref().unwrap_or(&Vec::new()).iter().find(|x| x.country_code == result.country_code) {
                    details.country_name = p.country_name.clone();
                    details.region = p.region.clone();
                }

                if let Some(p) = detail.combined_discount_info.as_ref().unwrap_or(&Vec::new()).iter().find(|x| x.combined_discount_id == result.combined_discount_id.unwrap_or(0)) {
                    details.combined_discount_name = p.combined_discount_name.clone();
                    details.total_discount_percentage = p.total_discount_percentage.clone();
                }

                aggregated_sales_details.push(details);
            }
        }

        Ok(aggregated_sales_details)
    })
    .await
    .map_err(|e| ErrorType::BadRequest(format!("sales detail insertion failed: {}", e)))?;
    Ok(aggregated_sales_details)
}


pub async fn get_sale_details_by_date(connection: &Connection, from_date: Option<String>, to_date: Option<String>) -> Result<Vec<CPartnerFinancialsDetailedSalesResult>, ErrorType> {
    let from_date = from_date.unwrap_or("1970-01-01".to_string());
    let to_date = to_date.unwrap_or("9999-12-31".to_string());
    let sale_details = connection.call(move |conn| {
        let mut stmt = conn.prepare(
            "
                SELECT
                    result.partnerid,
                    result.date,
                    result.line_item_type,
                    result.packageid,
                    result.bundleid,
                    result.appid,
                    result.game_item_id,
                    result.package_sale_type,
                    result.key_request_id,
                    result.platform,
                    result.country_code,
                    result.base_price,
                    result.sale_price,
                    result.currency,
                    result.gross_units_sold,
                    result.gross_units_returned,
                    result.gross_sales_usd,
                    result.gross_returns_usd,
                    result.net_tax_usd,
                    result.gross_units_activated,
                    result.view_grant_partnerid,
                    result.net_units_sold,
                    result.net_sales_usd,
                    result.avg_sale_price_usd,
                    result.combined_discount_id,
                    result.primary_appid,
                    result.additional_revenue_share_tier,
                    partner.partner_name,
                    package.package_name,
                    bundle.bundle_name,
                    app.app_name,
                    game_item.game_item_description,
                    game_item.game_item_category,
                    key_request.key_request_notes,
                    key_request.game_code_id,
                    key_request.game_code_description,
                    key_request.territory_code_id,
                    key_request.territory_code_description,
                    country.country_name,
                    country.region,
                    combined_discount.combined_discount_name,
                    combined_discount.total_discount_percentage,
                    combined_discount.discount_ids
                FROM steam_results result
                    LEFT JOIN steam_partner_info partner on result.partnerid = partner.partnerid
                    LEFT JOIN steam_package_info package on result.packageid = package.packageid
                    LEFT JOIN steam_bundle_info bundle on result.bundleid = bundle.bundleid
                    LEFT JOIN steam_app_info app on result.appid = app.appid
                    LEFT JOIN steam_game_item_info game_item on result.game_item_id = game_item.game_item_id
                    LEFT JOIN steam_key_request_info key_request on result.key_request_id = key_request.key_request_id
                    LEFT JOIN steam_country_info country on result.country_code = country.country_code
                    LEFT JOIN steam_combined_discount_info combined_discount on result.combined_discount_id = combined_discount.combined_discount_id
                WHERE date >= ?1 AND date <= ?2
            "
        )?;

        let sale_details_iter = stmt.query_map([from_date, to_date], |row| {
            Ok(CPartnerFinancialsDetailedSalesResult {
                partnerid: row.get("partnerid")?,
                date: row.get("date")?,
                line_item_type: row.get("line_item_type")?,
                packageid: row.get("packageid")?,
                bundleid: row.get("bundleid")?,
                appid: row.get("appid")?,
                game_item_id: row.get("game_item_id")?,
                package_sale_type: row.get("package_sale_type")?,
                key_request_id: row.get("key_request_id")?,
                platform: row.get("platform")?,
                country_code: row.get("country_code")?,
                base_price: row.get("base_price")?,
                sale_price: row.get("sale_price")?,
                currency: row.get("currency")?,
                gross_units_sold: row.get("gross_units_sold")?,
                gross_units_returned: row.get("gross_units_returned")?,
                gross_sales_usd: row.get("gross_sales_usd")?,
                gross_returns_usd: row.get("gross_returns_usd")?,
                net_tax_usd: row.get("net_tax_usd")?,
                gross_units_activated: row.get("gross_units_activated")?,
                view_grant_partnerid: row.get("view_grant_partnerid")?,
                net_units_sold: row.get("net_units_sold")?,
                net_sales_usd: row.get("net_sales_usd")?,
                avg_sale_price_usd: row.get("avg_sale_price_usd")?,
                combined_discount_id: row.get("combined_discount_id")?,
                primary_appid: row.get("primary_appid")?,
                additional_revenue_share_tier: row.get("additional_revenue_share_tier")?,
                partner_name: row.get("partner_name")?,
                package_name: row.get("package_name")?,
                bundle_name: row.get("bundle_name")?,
                app_name: row.get("app_name")?,
                game_item_description: row.get("game_item_description")?,
                game_item_category: row.get("game_item_category")?,
                key_request_notes: row.get("key_request_notes")?,
                game_code_id: row.get("game_code_id")?,
                game_code_description: row.get("game_code_description")?,
                territory_code_id: row.get("territory_code_id")?,
                territory_code_description: row.get("territory_code_description")?,
                country_name: row.get("country_name")?,
                region: row.get("region")?,
                combined_discount_name: row.get("combined_discount_name")?,
                total_discount_percentage: row.get("total_discount_percentage")?,
                //discount_ids: row.get("discount_ids")?
            })
        })?;

        let mut sale_details = Vec::new();

        for detail in sale_details_iter {
            let det = detail?;
            sale_details.push(det);
        }
        Ok(sale_details)
    })
    .await
    .map_err(|e| ErrorType::BadRequest(format!("getting sales details failed: {}", e)))?;

    Ok(sale_details)
}
