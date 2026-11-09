export interface ParkingRecord {
  id: number;
  plate: string;
  chargeType: string;
  vehicleType: string;
  vehicleModel: string;
  vehicleBrand: string;
  vehicleSeries: string;
  ownerPhone: string;
  ownerName: string;
  entryTime: string;
  entryLane: string;
  exitTime: string;
  exitLane: string;
  duration: string;
  durationMinutes: number;
  dueAmount: number;
  paidAmount: number;
  couponType: string;
  couponDiscount: number;
  chargeDiscountType: string;
  chargeDiscountAmount: number;
  paymentMethod: string;
  releaseMethod: string;
  operator: string;
  remark: string;
  sourceFile: string;
}

export interface ParseResult {
  records: ParkingRecord[];
  errors: string[];
}

export interface RecordFilters {
  plate: string;
  ownerName: string;
  vehicleType: string;
  chargeType: string;
  releaseMethod: string;
  paymentMethod: string;
  entryTimeStart: string;
  entryTimeEnd: string;
  exitTimeStart: string;
  exitTimeEnd: string;
}

export interface VehicleMonthlyItem {
  month: string;
  count: number;
  due: number;
  paid: number;
  unpaid: number;
}

export interface MonthlyRentalVehicle {
  id: number;
  plate: string;
  parkingSpace: string;
  vehicleType: string;
  ownerPhone: string;
  ownerName: string;
  ownerAddress: string;
  startDate: string;
  endDate: string;
  parkingGroup: string;
  parkingSpaceCount: number;
  sourceFile: string;
  building: string;
  unit: string;
  room: string;
}

export interface MonthlyRentalParseResult {
  records: MonthlyRentalVehicle[];
  errors: string[];
}

export interface MonthlyRentalFilters {
  plate: string;
  ownerName: string;
  vehicleType: string;
  startDateStart: string;
  startDateEnd: string;
  endDateStart: string;
  endDateEnd: string;
}

export interface FreeCar {
  id: number;
  plate: string;
  parkingSpace: string;
  vehicleType: string;
  ownerPhone: string;
  ownerName: string;
  ownerAddress: string;
  startDate: string;
  endDate: string;
  parkingGroup: string;
  parkingSpaceCount: number;
  sourceFile: string;
  building: string;
  unit: string;
  room: string;
}

export interface FreeCarParseResult {
  records: FreeCar[];
  errors: string[];
}

export interface FreeCarFilters {
  plate: string;
  ownerName: string;
  vehicleType: string;
  startDateStart: string;
  startDateEnd: string;
  endDateStart: string;
  endDateEnd: string;
}

export interface MonthlyCardLedger {
  id: number;
  seq: number;
  processDate: string;
  roomNumber: string;
  ownerName: string;
  ownerPhone: string;
  plate: string;
  paymentPeriod: string;
  paymentMonths: number;
  rent: number;
  managementFee: number;
  actualStartDate: string;
  actualEndDate: string;
  actualMonths: number;
  rentGifted: boolean;
  propertyFeeOwed: boolean;
  operator: string;
  remark: string;
  sourceFile: string;
}

export interface MonthlyCardLedgerParseResult {
  records: MonthlyCardLedger[];
  errors: string[];
}

export interface MonthlyCardLedgerFilters {
  plate: string;
  ownerName: string;
  roomNumber: string;
  processDateStart: string;
  processDateEnd: string;
}

export interface BuildingChargesFilters {
  building: string;
  unit: string;
  room: string;
  entryTimeStart: string;
  entryTimeEnd: string;
  exitTimeStart: string;
  exitTimeEnd: string;
  page?: number;
  perPage?: number;
}

export interface BuildingChargeItem {
  building: string;
  unit: string;
  room: string;
  plateCount: number;
  totalCount: number;
  totalDue: number;
  totalPaid: number;
  totalUnpaid: number;
}

export interface BuildingChargePlateDetail {
  plate: string;
  items: VehicleMonthlyItem[];
}

export interface Paginated<T> {
  items: T[];
  total: number;
  page: number;
  perPage: number;
}

export interface StatsSummary {
  totalCount: number;
  totalDue: number;
  totalPaid: number;
  totalUnpaid: number;
  avgDurationMinutes: number;
  avgDue: number;
  avgPaid: number;
}

export interface DistributionItem {
  name: string;
  count: number;
  due: number;
  paid: number;
}

export interface TimeSeriesItem {
  date: string;
  count: number;
  due: number;
  paid: number;
}

export interface TopUnpaidVehicle {
  plate: string;
  count: number;
  due: number;
  paid: number;
  unpaid: number;
}

export interface TopUnpaidVehiclesFilter {
  limit?: number;
  page: number;
  perPage: number;
}

export interface UnpaidIntervalItem {
  range: string;
  min: number;
  max?: number;
  vehicleCount: number;
  totalUnpaid: number;
}

export interface UnpaidIntervalStats {
  intervals: UnpaidIntervalItem[];
  totalVehicles: number;
  totalUnpaid: number;
  mean: number;
  median: number;
  stdDev: number;
  min: number;
  max: number;
}

export interface UnpaidIntervalFilter {
  entryTimeStart?: string;
  entryTimeEnd?: string;
  exitTimeStart?: string;
  exitTimeEnd?: string;
  bins?: number;
  mode?: "equal-freq" | "equal-width";
}

export interface StatsResult {
  summary: StatsSummary;
  vehicleTypeDistribution: DistributionItem[];
  chargeTypeDistribution: DistributionItem[];
  releaseMethodDistribution: DistributionItem[];
  paymentMethodDistribution: DistributionItem[];
  dailyTrend: TimeSeriesItem[];
  monthlyTrend: TimeSeriesItem[];
}
