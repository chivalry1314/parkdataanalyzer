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

export interface StatsResult {
  summary: StatsSummary;
  vehicleTypeDistribution: DistributionItem[];
  chargeTypeDistribution: DistributionItem[];
  releaseMethodDistribution: DistributionItem[];
  paymentMethodDistribution: DistributionItem[];
  dailyTrend: TimeSeriesItem[];
  monthlyTrend: TimeSeriesItem[];
}
