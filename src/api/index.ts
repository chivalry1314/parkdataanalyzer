import { invoke } from "@tauri-apps/api/core";
import type {
  BuildingChargeItem,
  BuildingChargePlateDetail,
  BuildingChargesFilters,
  FreeCar,
  MonthlyRentalVehicle,
  Paginated,
  ParkingRecord,
  StatsResult,
  StatsSummary,
  TopUnpaidVehicle,
  TopUnpaidVehiclesFilter,
  VehicleMonthlyItem,
} from "../types";

export interface ListRecordsFilter {
  plate?: string;
  ownerName?: string;
  vehicleType?: string;
  chargeType?: string;
  releaseMethod?: string;
  paymentMethod?: string;
  entryTimeStart?: string;
  entryTimeEnd?: string;
  exitTimeStart?: string;
  exitTimeEnd?: string;
  page: number;
  perPage: number;
}

export interface SaveRecordsRequest {
  records: ParkingRecord[];
  replace: boolean;
}

export interface RawPaginated<T> {
  items: T[];
  total: number;
  page: number;
  per_page: number;
}

export const saveRecords = (req: SaveRecordsRequest) =>
  invoke<number>("save_records_command", { req });

export const listRecords = async (filter: ListRecordsFilter): Promise<Paginated<ParkingRecord>> => {
  const raw = await invoke<RawPaginated<ParkingRecord>>("list_records_command", {
    filter: {
      plate: filter.plate,
      owner_name: filter.ownerName,
      vehicle_type: filter.vehicleType,
      charge_type: filter.chargeType,
      release_method: filter.releaseMethod,
      payment_method: filter.paymentMethod,
      entry_time_start: filter.entryTimeStart,
      entry_time_end: filter.entryTimeEnd,
      exit_time_start: filter.exitTimeStart,
      exit_time_end: filter.exitTimeEnd,
      page: filter.page,
      per_page: filter.perPage,
    },
  });
  return {
    items: raw.items,
    total: raw.total,
    page: raw.page,
    perPage: raw.per_page,
  };
};

export const clearRecords = () => invoke<number>("clear_records_command");

export const getStats = () => invoke<StatsResult>("get_stats_command");

export const getTopUnpaidVehicles = async (
  filter: TopUnpaidVehiclesFilter
): Promise<Paginated<TopUnpaidVehicle>> => {
  const raw = await invoke<RawPaginated<TopUnpaidVehicle>>("get_top_unpaid_vehicles_command", {
    filter: {
      limit: filter.limit,
      page: filter.page,
      per_page: filter.perPage,
    },
  });
  return {
    items: raw.items,
    total: raw.total,
    page: raw.page,
    perPage: raw.per_page,
  };
};

export const exportAllRecords = () => invoke<ParkingRecord[]>("export_all_records_command");

export const getFilteredStats = (filter: Omit<ListRecordsFilter, "page" | "perPage">) =>
  invoke<StatsSummary>("get_filtered_stats_command", {
    filter: {
      plate: filter.plate,
      owner_name: filter.ownerName,
      vehicle_type: filter.vehicleType,
      charge_type: filter.chargeType,
      release_method: filter.releaseMethod,
      payment_method: filter.paymentMethod,
      entry_time_start: filter.entryTimeStart,
      entry_time_end: filter.entryTimeEnd,
      exit_time_start: filter.exitTimeStart,
      exit_time_end: filter.exitTimeEnd,
      page: 1,
      per_page: 1,
    },
  });

export interface VehicleTimeFilters {
  entryTimeStart?: string;
  entryTimeEnd?: string;
  exitTimeStart?: string;
  exitTimeEnd?: string;
}

export const getVehicleMonthlyStats = (plate: string, timeFilters: VehicleTimeFilters = {}) =>
  invoke<VehicleMonthlyItem[]>("get_vehicle_monthly_stats_command", {
    filter: {
      plate,
      entry_time_start: timeFilters.entryTimeStart,
      entry_time_end: timeFilters.entryTimeEnd,
      exit_time_start: timeFilters.exitTimeStart,
      exit_time_end: timeFilters.exitTimeEnd,
    },
  });

export const listVehicleRecords = (
  plate: string,
  month: string,
  page: number,
  perPage: number,
  timeFilters: VehicleTimeFilters = {}
) =>
  invoke<RawPaginated<ParkingRecord>>("list_vehicle_records_command", {
    filter: {
      plate,
      month,
      entry_time_start: timeFilters.entryTimeStart,
      entry_time_end: timeFilters.entryTimeEnd,
      exit_time_start: timeFilters.exitTimeStart,
      exit_time_end: timeFilters.exitTimeEnd,
      page,
      per_page: perPage,
    },
  }).then((raw) => ({
    items: raw.items,
    total: raw.total,
    page: raw.page,
    perPage: raw.per_page,
  }));

export interface SaveMonthlyRentalsRequest {
  records: MonthlyRentalVehicle[];
  replace: boolean;
}

export interface ListMonthlyRentalsFilter {
  plate?: string;
  ownerName?: string;
  vehicleType?: string;
  startDateStart?: string;
  startDateEnd?: string;
  endDateStart?: string;
  endDateEnd?: string;
  page: number;
  perPage: number;
}

export const saveMonthlyRentals = (req: SaveMonthlyRentalsRequest) =>
  invoke<number>("save_monthly_rentals_command", { req });

export const listMonthlyRentals = async (
  filter: ListMonthlyRentalsFilter
): Promise<Paginated<MonthlyRentalVehicle>> => {
  const raw = await invoke<RawPaginated<MonthlyRentalVehicle>>("list_monthly_rentals_command", {
    filter: {
      plate: filter.plate,
      owner_name: filter.ownerName,
      vehicle_type: filter.vehicleType,
      start_date_start: filter.startDateStart,
      start_date_end: filter.startDateEnd,
      end_date_start: filter.endDateStart,
      end_date_end: filter.endDateEnd,
      page: filter.page,
      per_page: filter.perPage,
    },
  });
  return {
    items: raw.items,
    total: raw.total,
    page: raw.page,
    perPage: raw.per_page,
  };
};

export const clearMonthlyRentals = () => invoke<number>("clear_monthly_rentals_command");

export interface SaveFreeCarsRequest {
  records: FreeCar[];
  replace: boolean;
}

export interface ListFreeCarsFilter {
  plate?: string;
  ownerName?: string;
  vehicleType?: string;
  startDateStart?: string;
  startDateEnd?: string;
  endDateStart?: string;
  endDateEnd?: string;
  page: number;
  perPage: number;
}

export const saveFreeCars = (req: SaveFreeCarsRequest) =>
  invoke<number>("save_free_cars_command", { req });

export const listFreeCars = async (
  filter: ListFreeCarsFilter
): Promise<Paginated<FreeCar>> => {
  const raw = await invoke<RawPaginated<FreeCar>>("list_free_cars_command", {
    filter: {
      plate: filter.plate,
      owner_name: filter.ownerName,
      vehicle_type: filter.vehicleType,
      start_date_start: filter.startDateStart,
      start_date_end: filter.startDateEnd,
      end_date_start: filter.endDateStart,
      end_date_end: filter.endDateEnd,
      page: filter.page,
      per_page: filter.perPage,
    },
  });
  return {
    items: raw.items,
    total: raw.total,
    page: raw.page,
    perPage: raw.per_page,
  };
};

export const clearFreeCars = () => invoke<number>("clear_free_cars_command");

export const cleanFreeCarAddresses = () => invoke<number>("clean_free_car_addresses_command");

export const cleanMonthlyRentalAddresses = () =>
  invoke<number>("clean_monthly_rental_addresses_command");

export const updateFreeCar = (record: FreeCar) =>
  invoke<number>("update_free_car_command", { record });

export const updateMonthlyRental = (record: MonthlyRentalVehicle) =>
  invoke<number>("update_monthly_rental_command", { record });

export const getBuildingCharges = async (filter: BuildingChargesFilters): Promise<Paginated<BuildingChargeItem>> => {
  const raw = await invoke<RawPaginated<BuildingChargeItem>>("get_building_charges_command", {
    filter: {
      building: filter.building,
      unit: filter.unit,
      room: filter.room,
      entry_time_start: filter.entryTimeStart,
      entry_time_end: filter.entryTimeEnd,
      exit_time_start: filter.exitTimeStart,
      exit_time_end: filter.exitTimeEnd,
      page: filter.page ?? 1,
      per_page: filter.perPage ?? 20,
    },
  });
  return {
    items: raw.items,
    total: raw.total,
    page: raw.page,
    perPage: raw.per_page,
  };
};

export const getBuildingChargeDetail = (filter: BuildingChargesFilters) =>
  invoke<BuildingChargePlateDetail[]>("get_building_charge_detail_command", {
    filter: {
      building: filter.building,
      unit: filter.unit,
      room: filter.room,
      entry_time_start: filter.entryTimeStart,
      entry_time_end: filter.entryTimeEnd,
      exit_time_start: filter.exitTimeStart,
      exit_time_end: filter.exitTimeEnd,
    },
  });

export const exportBuildingCharges = (filePath: string, filter: BuildingChargesFilters) =>
  invoke<void>("export_building_charges_command", {
    filePath,
    filter: {
      building: filter.building,
      unit: filter.unit,
      room: filter.room,
      entry_time_start: filter.entryTimeStart,
      entry_time_end: filter.entryTimeEnd,
      exit_time_start: filter.exitTimeStart,
      exit_time_end: filter.exitTimeEnd,
    },
  });

export const getBackendInfo = () => invoke<string>("get_backend_info_command");
export const getDbPath = () => invoke<string>("get_db_path_command");
export const getDefaultDbPath = () => invoke<string>("get_default_db_path_command");
export const setDbPath = (path: string, migrate: boolean) =>
  invoke<string>("set_db_path_command", { path, migrate });

export interface VehicleMonthlyFilter {
  plate: string;
  entryTimeStart?: string;
  entryTimeEnd?: string;
  exitTimeStart?: string;
  exitTimeEnd?: string;
}

export const exportVehicleCharges = (filePath: string, filter: VehicleMonthlyFilter) =>
  invoke<void>("export_vehicle_charges_command", { filePath, filter: {
    plate: filter.plate,
    entry_time_start: filter.entryTimeStart,
    entry_time_end: filter.entryTimeEnd,
    exit_time_start: filter.exitTimeStart,
    exit_time_end: filter.exitTimeEnd,
  } });
