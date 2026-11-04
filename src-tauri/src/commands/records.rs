use crate::db;
use crate::db::models::*;

#[tauri::command]
pub fn save_records_command(req: SaveRecordsRequest) -> Result<usize, String> {
    db::save_records(req)
}

#[tauri::command]
pub fn list_records_command(filter: ListRecordsFilter) -> Result<Paginated<ParkingRecord>, String> {
    db::list_records(filter)
}

#[tauri::command]
pub fn clear_records_command() -> Result<usize, String> {
    db::clear_records()
}

#[tauri::command]
pub fn get_stats_command() -> Result<StatsResult, String> {
    db::get_stats()
}

#[tauri::command]
pub fn get_top_unpaid_vehicles_command(filter: TopUnpaidVehiclesFilter) -> Result<Paginated<TopUnpaidVehicle>, String> {
    db::get_top_unpaid_vehicles(filter)
}

#[tauri::command]
pub fn export_all_records_command() -> Result<Vec<ParkingRecord>, String> {
    db::export_all_records()
}

#[tauri::command]
pub fn get_filtered_stats_command(filter: ListRecordsFilter) -> Result<StatsSummary, String> {
    db::get_filtered_stats(filter)
}

#[tauri::command]
pub fn get_vehicle_monthly_stats_command(filter: VehicleMonthlyFilter) -> Result<Vec<VehicleMonthlyItem>, String> {
    db::get_vehicle_monthly_stats(filter)
}

#[tauri::command]
pub fn list_vehicle_records_command(filter: VehicleRecordsFilter) -> Result<Paginated<ParkingRecord>, String> {
    db::list_vehicle_records(filter)
}

#[tauri::command]
pub fn save_monthly_rentals_command(req: SaveMonthlyRentalsRequest) -> Result<usize, String> {
    db::save_monthly_rentals(req)
}

#[tauri::command]
pub fn list_monthly_rentals_command(filter: ListMonthlyRentalsFilter) -> Result<Paginated<MonthlyRentalVehicle>, String> {
    db::list_monthly_rentals(filter)
}

#[tauri::command]
pub fn clear_monthly_rentals_command() -> Result<usize, String> {
    db::clear_monthly_rentals()
}

#[tauri::command]
pub fn save_free_cars_command(req: SaveFreeCarsRequest) -> Result<usize, String> {
    db::save_free_cars(req)
}

#[tauri::command]
pub fn list_free_cars_command(filter: ListFreeCarsFilter) -> Result<Paginated<FreeCar>, String> {
    db::list_free_cars(filter)
}

#[tauri::command]
pub fn clear_free_cars_command() -> Result<usize, String> {
    db::clear_free_cars()
}

#[tauri::command]
pub fn clean_free_car_addresses_command() -> Result<usize, String> {
    db::clean_free_car_addresses()
}

#[tauri::command]
pub fn clean_monthly_rental_addresses_command() -> Result<usize, String> {
    db::clean_monthly_rental_addresses()
}

#[tauri::command]
pub fn get_building_charges_command(filter: BuildingChargesFilter) -> Result<Paginated<BuildingChargeItem>, String> {
    db::get_building_charges(filter)
}

#[tauri::command]
pub fn get_building_charge_detail_command(filter: BuildingChargesFilter) -> Result<Vec<BuildingChargePlateDetail>, String> {
    db::get_building_charge_detail(filter)
}

#[tauri::command]
pub fn get_db_path_command() -> Result<String, String> {
    db::current_db_path()
        .map(|p| p.to_string_lossy().to_string())
        .ok_or_else(|| "数据库路径未初始化".to_string())
}

#[tauri::command]
pub fn get_default_db_path_command() -> Result<String, String> {
    db::default_db_path()
        .map(|p| p.to_string_lossy().to_string())
        .ok_or_else(|| "应用目录未初始化".to_string())
}

#[tauri::command]
pub fn set_db_path_command(path: String, migrate: bool) -> Result<String, String> {
    let new_path = std::path::PathBuf::from(path);
    db::set_db_path(new_path, migrate)?;
    get_db_path_command()
}
