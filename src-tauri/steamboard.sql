CREATE TABLE IF NOT EXISTS "steam_discount_info" (
	"discountid" INTEGER NOT NULL UNIQUE,
	"discount_description" TEXT,
	"discount_group" TEXT,
	"discount_percentage" INTEGER DEFAULT 0,
	PRIMARY KEY("discountid")
);

CREATE TABLE IF NOT EXISTS "steam_game_item_info" (
	"appid" INTEGER NOT NULL UNIQUE,
	"game_item_id" INTEGER UNIQUE,
	"game_item_description" TEXT,
	"game_item_category" TEXT,
	PRIMARY KEY("appid")
);

CREATE TABLE IF NOT EXISTS "steam_combined_discount_info" (
	"combined_discount_id" INTEGER NOT NULL UNIQUE,
	"combined_discount_name" TEXT,
	"total_discount_percentage" INTEGER DEFAULT 0,
	"discount_ids" TEXT,
	PRIMARY KEY("combined_discount_id")
);

CREATE TABLE IF NOT EXISTS "steam_bundle_info" (
	"bundleid" INTEGER NOT NULL UNIQUE,
	"bundle_name" TEXT,
	PRIMARY KEY("bundleid")
);

CREATE TABLE IF NOT EXISTS "steam_country_info" (
	"country_code" TEXT NOT NULL UNIQUE,
	"country_name" TEXT,
	"region" TEXT,
	PRIMARY KEY("country_code")
);

CREATE TABLE IF NOT EXISTS "steam_partner_info" (
	"partnerid" INTEGER NOT NULL UNIQUE,
	"partner_name" TEXT,
	PRIMARY KEY("partnerid")
);

CREATE TABLE IF NOT EXISTS "steam_app_info" (
	"appid" INTEGER NOT NULL UNIQUE,
	"app_name" TEXT,
	PRIMARY KEY("appid")
);

CREATE TABLE IF NOT EXISTS "steam_package_info" (
	"packageid" INTEGER NOT NULL UNIQUE,
	"package_name" TEXT,
	PRIMARY KEY("packageid")
);

CREATE TABLE IF NOT EXISTS "steam_key_request_info" (
	"key_request_id" INTEGER NOT NULL UNIQUE,
	"key_request_notes" TEXT,
	"game_code_id" INTEGER,
	"game_code_description" TEXT,
	"territory_code_id" INTEGER,
	"territory_code_description" TEXT,
	PRIMARY KEY("key_request_id")
);

CREATE TABLE IF NOT EXISTS "steam_results" (
	"id" INTEGER,
	"partnerid" INTEGER NOT NULL,
	"date" TEXT NOT NULL,
	"line_item_type" TEXT,
	"packageid" INTEGER,
	"bundleid" INTEGER,
	"appid" INTEGER,
	"game_item_id" INTEGER,
	"package_sale_type" TEXT,
	"key_request_id" INTEGER,
	"platform" TEXT,
	"country_code" TEXT,
	"base_price" INTEGER,
	"sale_price" INTEGER,
	"currency" TEXT,
	"gross_units_sold" INTEGER DEFAULT 0,
	"gross_units_returned" INTEGER DEFAULT 0,
	"gross_sales_usd" REAL DEFAULT 0,
	"gross_returns_usd" REAL DEFAULT 0,
	"net_tax_usd" REAL DEFAULT 0,
	"gross_units_activated" INTEGER DEFAULT 0,
	"view_grant_partnerid" INTEGER,
	"net_units_sold" INTEGER DEFAULT 0,
	"net_sales_usd" REAL DEFAULT 0,
	"avg_sale_price_usd" REAL DEFAULT 0,
	"combined_discount_id" INTEGER,
	"primary_appid" INTEGER,
	"additional_revenue_share_tier" INTEGER,
	PRIMARY KEY("id"),
	FOREIGN KEY ("packageid") REFERENCES "steam_package_info"("packageid")
	ON UPDATE NO ACTION ON DELETE NO ACTION,
	FOREIGN KEY ("appid") REFERENCES "steam_app_info"("appid")
	ON UPDATE NO ACTION ON DELETE NO ACTION,
	FOREIGN KEY ("partnerid") REFERENCES "steam_partner_info"("partnerid")
	ON UPDATE NO ACTION ON DELETE NO ACTION,
	FOREIGN KEY ("country_code") REFERENCES "steam_country_info"("country_code")
	ON UPDATE NO ACTION ON DELETE NO ACTION,
	FOREIGN KEY ("bundleid") REFERENCES "steam_bundle_info"("bundleid")
	ON UPDATE NO ACTION ON DELETE NO ACTION,
	FOREIGN KEY ("combined_discount_id") REFERENCES "steam_combined_discount_info"("combined_discount_id")
	ON UPDATE NO ACTION ON DELETE NO ACTION,
	FOREIGN KEY ("game_item_id") REFERENCES "steam_game_item_info"("game_item_id")
	ON UPDATE NO ACTION ON DELETE NO ACTION,
	FOREIGN KEY ("key_request_id") REFERENCES "steam_key_request_info"("key_request_id")
	ON UPDATE NO ACTION ON DELETE NO ACTION
);

CREATE TABLE IF NOT EXISTS "settings" (
	"id" INTEGER NOT NULL UNIQUE,
	"steam_api_key" TEXT UNIQUE,
	"poll_interval" INTEGER NOT NULL DEFAULT 30,
	"highwatermark" TEXT NOT NULL DEFAULT '0',
	PRIMARY KEY("id")
);

CREATE TABLE IF NOT EXISTS "steam_dates" (
	"date" TEXT NOT NULL UNIQUE,
	"highwatermark_id" INTEGER NOT NULL,
	PRIMARY KEY("date")
);
