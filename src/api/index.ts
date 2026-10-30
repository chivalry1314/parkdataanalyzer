import { invoke } from "@tauri-apps/api/core";
import type {
  Paginated,
  ParkingRecord,
  StatsResult,
  StatsSummary,
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
