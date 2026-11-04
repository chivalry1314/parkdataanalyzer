// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod db;

use commands::export::{
    export_building_charges_command,
    export_vehicle_charges_command,
};
use commands::info::get_backend_info_command;
use commands::records::*;
use commands::records::{
    clean_free_car_addresses_command,
    clean_monthly_rental_addresses_command,
    clear_free_cars_command,
    clear_monthly_rentals_command,
    get_building_charge_detail_command,
    get_building_charges_command,
    get_top_unpaid_vehicles_command,
    list_free_cars_command,
    list_monthly_rentals_command,
    save_free_cars_command,
    save_monthly_rentals_command,
};
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            let app_dir = app.path().app_data_dir().expect("Failed to get app data dir");
            std::fs::create_dir_all(&app_dir).ok();

            if db::is_db_configured(&app_dir) {
                db::init_db(app_dir).expect("Failed to initialize database");
            } else {
                db::set_app_dir(app_dir).expect("Failed to set app dir");
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            save_records_command,
            list_records_command,
            clear_records_command,
            get_stats_command,
            export_all_records_command,
            get_filtered_stats_command,
            get_vehicle_monthly_stats_command,
            list_vehicle_records_command,
            get_top_unpaid_vehicles_command,
            get_db_path_command,
            get_default_db_path_command,
            set_db_path_command,
            export_vehicle_charges_command,
            export_building_charges_command,
            save_monthly_rentals_command,
            list_monthly_rentals_command,
            clear_monthly_rentals_command,
            save_free_cars_command,
            list_free_cars_command,
            clear_free_cars_command,
            clean_free_car_addresses_command,
            clean_monthly_rental_addresses_command,
            get_building_charges_command,
            get_building_charge_detail_command,
            get_backend_info_command,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
