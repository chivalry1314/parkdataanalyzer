use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParkingRecord {
    pub id: i64,
    pub plate: String,
    #[serde(rename = "chargeType")]
    pub charge_type: String,
    #[serde(rename = "vehicleType")]
    pub vehicle_type: String,
    #[serde(rename = "vehicleModel")]
    pub vehicle_model: String,
    #[serde(rename = "vehicleBrand")]
    pub vehicle_brand: String,
    #[serde(rename = "vehicleSeries")]
    pub vehicle_series: String,
    #[serde(rename = "ownerPhone")]
    pub owner_phone: String,
    #[serde(rename = "ownerName")]
    pub owner_name: String,
    #[serde(rename = "entryTime")]
    pub entry_time: String,
    #[serde(rename = "entryLane")]
    pub entry_lane: String,
    #[serde(rename = "exitTime")]
    pub exit_time: String,
    #[serde(rename = "exitLane")]
    pub exit_lane: String,
    pub duration: String,
    #[serde(rename = "durationMinutes")]
    pub duration_minutes: f64,
    #[serde(rename = "dueAmount")]
    pub due_amount: i64,
    #[serde(rename = "paidAmount")]
    pub paid_amount: i64,
    #[serde(rename = "couponType")]
    pub coupon_type: String,
    #[serde(rename = "couponDiscount")]
    pub coupon_discount: i64,
    #[serde(rename = "chargeDiscountType")]
    pub charge_discount_type: String,
    #[serde(rename = "chargeDiscountAmount")]
    pub charge_discount_amount: i64,
    #[serde(rename = "paymentMethod")]
    pub payment_method: String,
    #[serde(rename = "releaseMethod")]
    pub release_method: String,
    pub operator: String,
    pub remark: String,
    #[serde(rename = "sourceFile")]
    pub source_file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveRecordsRequest {
    pub records: Vec<ParkingRecord>,
    pub replace: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListRecordsFilter {
    pub plate: Option<String>,
    pub owner_name: Option<String>,
    pub vehicle_type: Option<String>,
    pub charge_type: Option<String>,
    pub release_method: Option<String>,
    pub payment_method: Option<String>,
    pub entry_time_start: Option<String>,
    pub entry_time_end: Option<String>,
    pub exit_time_start: Option<String>,
    pub exit_time_end: Option<String>,
    pub page: i64,
    pub per_page: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paginated<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsSummary {
    #[serde(rename = "totalCount")]
    pub total_count: i64,
    #[serde(rename = "totalDue")]
    pub total_due: i64,
    #[serde(rename = "totalPaid")]
    pub total_paid: i64,
    #[serde(rename = "totalUnpaid")]
    pub total_unpaid: i64,
    #[serde(rename = "avgDurationMinutes")]
    pub avg_duration_minutes: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionItem {
    pub name: String,
    pub count: i64,
    pub due: i64,
    pub paid: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesItem {
    pub date: String,
    pub count: i64,
    pub due: i64,
    pub paid: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleMonthlyItem {
    pub month: String,
    pub count: i64,
    pub due: i64,
    pub paid: i64,
    pub unpaid: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleMonthlyFilter {
    pub plate: String,
    pub entry_time_start: Option<String>,
    pub entry_time_end: Option<String>,
    pub exit_time_start: Option<String>,
    pub exit_time_end: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleRecordsFilter {
    pub plate: String,
    pub month: String,
    pub entry_time_start: Option<String>,
    pub entry_time_end: Option<String>,
    pub exit_time_start: Option<String>,
    pub exit_time_end: Option<String>,
    pub page: i64,
    pub per_page: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsResult {
    pub summary: StatsSummary,
    #[serde(rename = "vehicleTypeDistribution")]
    pub vehicle_type_distribution: Vec<DistributionItem>,
    #[serde(rename = "chargeTypeDistribution")]
    pub charge_type_distribution: Vec<DistributionItem>,
    #[serde(rename = "releaseMethodDistribution")]
    pub release_method_distribution: Vec<DistributionItem>,
    #[serde(rename = "paymentMethodDistribution")]
    pub payment_method_distribution: Vec<DistributionItem>,
    #[serde(rename = "dailyTrend")]
    pub daily_trend: Vec<TimeSeriesItem>,
    #[serde(rename = "monthlyTrend")]
    pub monthly_trend: Vec<TimeSeriesItem>,
}
