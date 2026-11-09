use once_cell::sync::OnceCell;
use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub mod migrations;
pub mod models;

use models::*;

static DB: Mutex<Option<Arc<Mutex<Connection>>>> = Mutex::new(None);
static DB_PATH: Mutex<Option<PathBuf>> = Mutex::new(None);
static APP_DIR: OnceCell<PathBuf> = OnceCell::new();

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct AppConfig {
    #[serde(default)]
    db_path: Option<PathBuf>,
}

fn config_path() -> Option<PathBuf> {
    APP_DIR.get().map(|d| d.join("config.json"))
}

fn load_config() -> AppConfig {
    if let Some(path) = config_path() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(config) = serde_json::from_str::<AppConfig>(&content) {
                return config;
            }
        }
    }
    AppConfig::default()
}

fn save_config(config: &AppConfig) -> Result<(), String> {
    let path = config_path().ok_or("应用目录未初始化")?;
    let content = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(&path, content).map_err(|e| format!("写入配置文件失败：{}", e))?;
    Ok(())
}

fn run_migrations(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS migrations_applied (id TEXT PRIMARY KEY)",
        [],
    )?;

    for (id, sql) in migrations::MIGRATIONS {
        let applied: bool = conn
            .query_row(
                "SELECT 1 FROM migrations_applied WHERE id = ?1",
                [id],
                |_| Ok(true),
            )
            .unwrap_or(false);
        if !applied {
            for stmt in sql.split(';').map(|s| s.trim()).filter(|s| !s.is_empty()) {
                if let Err(e) = conn.execute(stmt, []) {
                    let msg = e.to_string();
                    if msg.contains("duplicate column name") {
                        eprintln!("Migration {}: column already exists, skipping: {}", id, msg);
                    } else {
                        return Err(e);
                    }
                }
            }
            conn.execute("INSERT INTO migrations_applied (id) VALUES (?1)", [id])?;
        }
    }

    conn.execute("PRAGMA foreign_keys = ON;", [])?;
    Ok(())
}

fn open_db_at(path: &Path) -> Result<Connection, String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建数据库目录失败：{}", e))?;
    }
    let conn = Connection::open(path).map_err(|e| format!("打开数据库失败：{}", e))?;
    run_migrations(&conn).map_err(|e| format!("执行数据库迁移失败：{}", e))?;
    Ok(conn)
}

fn set_active_db(db_path: PathBuf) -> Result<(), String> {
    let conn = open_db_at(&db_path)?;

    let mut db_guard = DB.lock().map_err(|e| e.to_string())?;
    *db_guard = Some(Arc::new(Mutex::new(conn)));

    let mut path_guard = DB_PATH.lock().map_err(|e| e.to_string())?;
    *path_guard = Some(db_path);

    Ok(())
}

pub fn set_app_dir(app_dir: PathBuf) -> Result<(), String> {
    APP_DIR.set(app_dir).map_err(|_| "应用目录已初始化".to_string())?;
    Ok(())
}

pub fn is_db_configured(app_dir: &Path) -> bool {
    let config_file = app_dir.join("config.json");
    if let Ok(content) = fs::read_to_string(&config_file) {
        if let Ok(config) = serde_json::from_str::<AppConfig>(&content) {
            if config.db_path.is_some() {
                return true;
            }
        }
    }
    app_dir.join("parkingsyslog.db").exists()
}

pub fn init_db(app_dir: PathBuf) -> Result<(), String> {
    if APP_DIR.get().is_none() {
        APP_DIR.set(app_dir.clone()).map_err(|_| "应用目录已初始化".to_string())?;
    }

    let config = load_config();
    let db_path = config.db_path.unwrap_or_else(|| app_dir.join("parkingsyslog.db"));

    set_active_db(db_path)
}

pub fn get_setting(key: &str) -> Result<Option<String>, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;
    let value: Result<String, _> = conn.query_row(
        "SELECT value FROM app_settings WHERE key = ?1",
        [key],
        |row| row.get(0),
    );
    match value {
        Ok(v) => Ok(Some(v)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

pub fn set_setting(key: &str, value: &str) -> Result<(), String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO app_settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        [key, value],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn db() -> Arc<Mutex<Connection>> {
    DB.lock()
        .expect("DB mutex poisoned")
        .as_ref()
        .expect("Database not initialized")
        .clone()
}

pub fn current_db_path() -> Option<PathBuf> {
    DB_PATH.lock().ok()?.clone()
}

pub fn default_db_path() -> Option<PathBuf> {
    APP_DIR.get().map(|d| d.join("parkingsyslog.db"))
}

pub fn set_db_path(new_path: PathBuf, migrate: bool) -> Result<(), String> {
    if migrate {
        if let Some(ref old) = current_db_path() {
            if old.exists() && old != &new_path {
                fs::copy(old, &new_path).map_err(|e| {
                    format!("迁移数据失败：无法从 {} 复制到 {}，错误：{}", old.display(), new_path.display(), e)
                })?;
            }
        }
    }

    set_active_db(new_path.clone())?;

    let mut config = load_config();
    config.db_path = Some(new_path);
    save_config(&config)?;

    Ok(())
}

fn map_record(row: &rusqlite::Row) -> Result<ParkingRecord> {
    Ok(ParkingRecord {
        id: row.get(0)?,
        plate: row.get(1)?,
        charge_type: row.get(2)?,
        vehicle_type: row.get(3)?,
        vehicle_model: row.get(4)?,
        vehicle_brand: row.get(5)?,
        vehicle_series: row.get(6)?,
        owner_phone: row.get(7)?,
        owner_name: row.get(8)?,
        entry_time: row.get(9)?,
        entry_lane: row.get(10)?,
        exit_time: row.get(11)?,
        exit_lane: row.get(12)?,
        duration: row.get(13)?,
        duration_minutes: row.get(14)?,
        due_amount: row.get(15)?,
        paid_amount: row.get(16)?,
        coupon_type: row.get(17)?,
        coupon_discount: row.get(18)?,
        charge_discount_type: row.get(19)?,
        charge_discount_amount: row.get(20)?,
        payment_method: row.get(21)?,
        release_method: row.get(22)?,
        operator: row.get(23)?,
        remark: row.get(24)?,
        source_file: row.get(25)?,
    })
}

pub fn save_records(req: SaveRecordsRequest) -> Result<usize, String> {
    let db = db();
    let mut conn = db.lock().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    if req.replace {
        tx.execute("DELETE FROM parking_records", [])
            .map_err(|e| e.to_string())?;
    }

    {
        let mut stmt = tx.prepare(
            "INSERT INTO parking_records (
                plate, charge_type, vehicle_type, vehicle_model, vehicle_brand, vehicle_series,
                owner_phone, owner_name, entry_time, entry_lane, exit_time, exit_lane,
                duration, duration_minutes, due_amount, paid_amount, coupon_type, coupon_discount,
                charge_discount_type, charge_discount_amount, payment_method, release_method,
                operator, remark, source_file
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16,
                ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25
            )"
        ).map_err(|e| e.to_string())?;

        for r in &req.records {
            stmt.execute(rusqlite::params![
                r.plate.trim(), r.charge_type, r.vehicle_type, r.vehicle_model, r.vehicle_brand, r.vehicle_series,
                r.owner_phone, r.owner_name, r.entry_time, r.entry_lane, r.exit_time, r.exit_lane,
                r.duration, r.duration_minutes, r.due_amount, r.paid_amount, r.coupon_type, r.coupon_discount,
                r.charge_discount_type, r.charge_discount_amount, r.payment_method, r.release_method,
                r.operator, r.remark, r.source_file
            ]).map_err(|e| e.to_string())?;
        }
    }

    tx.commit().map_err(|e| e.to_string())?;
    Ok(req.records.len())
}

fn build_where_clause(filter: &ListRecordsFilter) -> String {
    let mut conditions: Vec<String> = vec![];
    if let Some(s) = &filter.plate {
        conditions.push(format!("TRIM(plate) LIKE '%{}%'", s.replace("'", "''")));
    }
    if let Some(s) = &filter.owner_name {
        conditions.push(format!("owner_name LIKE '%{}%'", s.replace("'", "''")));
    }
    if let Some(s) = &filter.vehicle_type {
        conditions.push(format!("vehicle_type LIKE '%{}%'", s.replace("'", "''")));
    }
    if let Some(s) = &filter.charge_type {
        conditions.push(format!("charge_type LIKE '%{}%'", s.replace("'", "''")));
    }
    if let Some(s) = &filter.release_method {
        conditions.push(format!("release_method LIKE '%{}%'", s.replace("'", "''")));
    }
    if let Some(s) = &filter.payment_method {
        conditions.push(format!("payment_method LIKE '%{}%'", s.replace("'", "''")));
    }
    if let Some(s) = &filter.entry_time_start {
        conditions.push(format!("entry_time >= '{}'", s.replace("'", "''")));
    }
    if let Some(s) = &filter.entry_time_end {
        conditions.push(format!("entry_time <= '{}'", s.replace("'", "''")));
    }
    if let Some(s) = &filter.exit_time_start {
        conditions.push(format!("exit_time >= '{}'", s.replace("'", "''")));
    }
    if let Some(s) = &filter.exit_time_end {
        conditions.push(format!("exit_time <= '{}'", s.replace("'", "''")));
    }
    if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    }
}

pub fn list_records(filter: ListRecordsFilter) -> Result<Paginated<ParkingRecord>, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    let where_clause = build_where_clause(&filter);

    let total: i64 = conn.query_row(
        &format!("SELECT COUNT(*) FROM parking_records {}", where_clause),
        [],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;

    let offset = (filter.page - 1) * filter.per_page;
    let sql = format!(
        "SELECT * FROM parking_records {} ORDER BY id LIMIT {} OFFSET {}",
        where_clause, filter.per_page, offset
    );

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let items = stmt
        .query_map([], map_record)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(Paginated {
        items,
        total,
        page: filter.page,
        per_page: filter.per_page,
    })
}

pub fn clear_records() -> Result<usize, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;
    let count = conn.execute("DELETE FROM parking_records", [])
        .map_err(|e| e.to_string())?;
    Ok(count)
}

pub fn get_stats() -> Result<StatsResult, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    let summary = conn.query_row(
        "SELECT
            COUNT(*),
            COALESCE(SUM(due_amount), 0),
            COALESCE(SUM(paid_amount), 0),
            COALESCE(AVG(duration_minutes), 0)
         FROM parking_records",
        [],
        |row| {
            let total_due: i64 = row.get(1)?;
            let total_paid: i64 = row.get(2)?;
            Ok(StatsSummary {
                total_count: row.get(0)?,
                total_due,
                total_paid,
                total_unpaid: total_due - total_paid,
                avg_duration_minutes: row.get(3)?,
            })
        },
    ).map_err(|e| e.to_string())?;

    fn distribution(
        conn: &Connection,
        column: &str,
    ) -> Result<Vec<DistributionItem>, String> {
        let sql = format!(
            "SELECT COALESCE({column}, '') AS name, COUNT(*) AS cnt, COALESCE(SUM(due_amount), 0), COALESCE(SUM(paid_amount), 0)
             FROM parking_records
             GROUP BY COALESCE({column}, '')
             ORDER BY cnt DESC",
            column = column
        );
        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let items = stmt
            .query_map([], |row| {
                Ok(DistributionItem {
                    name: row.get(0)?,
                    count: row.get(1)?,
                    due: row.get(2)?,
                    paid: row.get(3)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        Ok(items)
    }

    fn trend(conn: &Connection, fmt: &str, limit_days: Option<i64>) -> Result<Vec<TimeSeriesItem>, String> {
        let limit_clause = limit_days
            .map(|n| format!(" ORDER BY date DESC LIMIT {}", n))
            .unwrap_or_else(|| " ORDER BY date".to_string());
        let sql = format!(
            "SELECT strftime('{fmt}', entry_time) AS date,
                    COUNT(*) AS cnt,
                    COALESCE(SUM(due_amount), 0),
                    COALESCE(SUM(paid_amount), 0)
             FROM parking_records
             WHERE entry_time != ''
             GROUP BY date{limit}",
            fmt = fmt,
            limit = limit_clause
        );
        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let mut items: Vec<TimeSeriesItem> = stmt
            .query_map([], |row| {
                Ok(TimeSeriesItem {
                    date: row.get(0)?,
                    count: row.get(1)?,
                    due: row.get(2)?,
                    paid: row.get(3)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        if limit_days.is_some() {
            items.reverse();
        }
        Ok(items)
    }

    Ok(StatsResult {
        vehicle_type_distribution: distribution(&conn, "vehicle_type")?,
        charge_type_distribution: distribution(&conn, "charge_type")?,
        release_method_distribution: distribution(&conn, "release_method")?,
        payment_method_distribution: distribution(&conn, "payment_method")?,
        daily_trend: trend(&conn, "%Y-%m-%d", Some(30))?,
        monthly_trend: trend(&conn, "%Y-%m", None)?,
        summary,
    })
}

pub fn get_top_unpaid_vehicles(filter: TopUnpaidVehiclesFilter) -> Result<Paginated<TopUnpaidVehicle>, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    let limit = std::cmp::max(filter.limit.unwrap_or(10), 1);
    let page = std::cmp::max(filter.page, 1);
    let per_page = std::cmp::max(filter.per_page, 1);
    let offset = (page - 1) * per_page;

    let total: i64 = conn.query_row(
        "WITH ranked AS (
            SELECT plate,
                   COALESCE(SUM(due_amount - paid_amount), 0) AS unpaid
            FROM parking_records
            GROUP BY plate
            HAVING unpaid > 0
            ORDER BY unpaid DESC
            LIMIT ?1
        ) SELECT COUNT(*) FROM ranked",
        [limit],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;

    let sql =
        "WITH ranked AS (
            SELECT plate,
                   COUNT(*) AS cnt,
                   COALESCE(SUM(due_amount), 0) AS due,
                   COALESCE(SUM(paid_amount), 0) AS paid,
                   COALESCE(SUM(due_amount - paid_amount), 0) AS unpaid
            FROM parking_records
            GROUP BY plate
            HAVING unpaid > 0
            ORDER BY unpaid DESC
            LIMIT ?1
        )
        SELECT plate, cnt, due, paid, unpaid
        FROM ranked
        ORDER BY unpaid DESC
        LIMIT ?2 OFFSET ?3";

    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
    let items = stmt
        .query_map([limit, per_page, offset], |row| {
            Ok(TopUnpaidVehicle {
                plate: row.get(0)?,
                count: row.get(1)?,
                due: row.get(2)?,
                paid: row.get(3)?,
                unpaid: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(Paginated {
        items,
        total,
        page,
        per_page,
    })
}

pub fn export_all_records() -> Result<Vec<ParkingRecord>, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT * FROM parking_records ORDER BY id")
        .map_err(|e| e.to_string())?;
    let items = stmt
        .query_map([], map_record)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(items)
}

pub fn get_filtered_stats(filter: ListRecordsFilter) -> Result<StatsSummary, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    let where_clause = build_where_clause(&filter);

    let sql = format!(
        "SELECT
            COUNT(*),
            COALESCE(SUM(due_amount), 0),
            COALESCE(SUM(paid_amount), 0),
            COALESCE(AVG(duration_minutes), 0)
         FROM parking_records {}",
        where_clause
    );

    conn.query_row(&sql, [], |row| {
        let total_due: i64 = row.get(1)?;
        let total_paid: i64 = row.get(2)?;
        Ok(StatsSummary {
            total_count: row.get(0)?,
            total_due,
            total_paid,
            total_unpaid: total_due - total_paid,
            avg_duration_minutes: row.get(3)?,
        })
    })
    .map_err(|e| e.to_string())
}

fn build_vehicle_where_clause(
    plate: &str,
    entry_time_start: &Option<String>,
    entry_time_end: &Option<String>,
    exit_time_start: &Option<String>,
    exit_time_end: &Option<String>,
    month: Option<&str>,
) -> String {
    let mut conditions: Vec<String> = vec![
        format!("TRIM(plate) LIKE '%{}%'", plate.replace("'", "''")),
        "exit_time IS NOT NULL AND exit_time != ''".to_string(),
    ];
    if let Some(s) = entry_time_start {
        if !s.is_empty() {
            conditions.push(format!("entry_time >= '{}'", s.replace("'", "''")));
        }
    }
    if let Some(s) = entry_time_end {
        if !s.is_empty() {
            conditions.push(format!("entry_time <= '{}'", s.replace("'", "''")));
        }
    }
    if let Some(s) = exit_time_start {
        if !s.is_empty() {
            conditions.push(format!("exit_time >= '{}'", s.replace("'", "''")));
        }
    }
    if let Some(s) = exit_time_end {
        if !s.is_empty() {
            conditions.push(format!("exit_time <= '{}'", s.replace("'", "''")));
        }
    }
    if let Some(m) = month {
        if !m.is_empty() {
            conditions.push(format!("strftime('%Y-%m', exit_time) = '{}'", m.replace("'", "''")));
        }
    }
    format!("WHERE {}", conditions.join(" AND "))
}

pub fn get_vehicle_monthly_stats(filter: VehicleMonthlyFilter) -> Result<Vec<VehicleMonthlyItem>, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    let where_clause = build_vehicle_where_clause(
        &filter.plate,
        &filter.entry_time_start,
        &filter.entry_time_end,
        &filter.exit_time_start,
        &filter.exit_time_end,
        None,
    );

    let sql = format!(
        "SELECT
            strftime('%Y-%m', exit_time) AS month,
            COUNT(*) AS cnt,
            COALESCE(SUM(due_amount), 0),
            COALESCE(SUM(paid_amount), 0)
         FROM parking_records
         {}
         GROUP BY month
         ORDER BY month",
        where_clause
    );

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let items = stmt
        .query_map([], |row| {
            let due: i64 = row.get(2)?;
            let paid: i64 = row.get(3)?;
            Ok(VehicleMonthlyItem {
                month: row.get(0)?,
                count: row.get(1)?,
                due,
                paid,
                unpaid: due - paid,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(items)
}

fn parse_building_unit_room(address: &str) -> (String, String, String) {
    let parts: Vec<&str> = address.split('-').collect();
    match parts.len() {
        3 => (parts[0].to_string(), parts[1].to_string(), parts[2].to_string()),
        2 => (parts[0].to_string(), parts[1].to_string(), String::new()),
        1 => (parts[0].to_string(), String::new(), String::new()),
        _ => (String::new(), String::new(), String::new()),
    }
}

fn map_monthly_rental(row: &rusqlite::Row) -> Result<MonthlyRentalVehicle> {
    Ok(MonthlyRentalVehicle {
        id: row.get(0)?,
        plate: row.get(1)?,
        parking_space: row.get(2)?,
        vehicle_type: row.get(3)?,
        owner_phone: row.get(4)?,
        owner_name: row.get(5)?,
        owner_address: row.get(6)?,
        start_date: row.get(7)?,
        end_date: row.get(8)?,
        parking_group: row.get(9)?,
        parking_space_count: row.get(10)?,
        source_file: row.get(11)?,
        building: row.get(12)?,
        unit: row.get(13)?,
        room: row.get(14)?,
    })
}

pub fn save_monthly_rentals(req: SaveMonthlyRentalsRequest) -> Result<usize, String> {
    let db = db();
    let mut conn = db.lock().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    if req.replace {
        tx.execute("DELETE FROM monthly_rental_vehicles", [])
            .map_err(|e| e.to_string())?;
    }

    {
        let mut stmt = tx.prepare(
            "INSERT INTO monthly_rental_vehicles (
                plate, parking_space, vehicle_type, owner_phone, owner_name, owner_address,
                start_date, end_date, parking_group, parking_space_count, source_file,
                building, unit, room
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14
            )"
        ).map_err(|e| e.to_string())?;

        for r in &req.records {
            let (building, unit, room) = parse_building_unit_room(&r.owner_address);
            stmt.execute(rusqlite::params![
                r.plate.trim(), r.parking_space, r.vehicle_type, r.owner_phone, r.owner_name, r.owner_address,
                r.start_date, r.end_date, r.parking_group, r.parking_space_count, r.source_file,
                building, unit, room
            ]).map_err(|e| e.to_string())?;
        }
    }

    tx.commit().map_err(|e| e.to_string())?;
    Ok(req.records.len())
}

fn build_monthly_where_clause(filter: &ListMonthlyRentalsFilter) -> String {
    let mut conditions: Vec<String> = vec![];
    if let Some(s) = &filter.plate {
        conditions.push(format!("TRIM(plate) LIKE '%{}%'", s.replace("'", "''")));
    }
    if let Some(s) = &filter.owner_name {
        conditions.push(format!("owner_name LIKE '%{}%'", s.replace("'", "''")));
    }
    if let Some(s) = &filter.vehicle_type {
        conditions.push(format!("vehicle_type LIKE '%{}%'", s.replace("'", "''")));
    }
    if let Some(s) = &filter.start_date_start {
        conditions.push(format!("start_date >= '{}'", s.replace("'", "''")));
    }
    if let Some(s) = &filter.start_date_end {
        conditions.push(format!("start_date <= '{}'", s.replace("'", "''")));
    }
    if let Some(s) = &filter.end_date_start {
        conditions.push(format!("end_date >= '{}'", s.replace("'", "''")));
    }
    if let Some(s) = &filter.end_date_end {
        conditions.push(format!("end_date <= '{}'", s.replace("'", "''")));
    }
    if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    }
}

pub fn list_monthly_rentals(filter: ListMonthlyRentalsFilter) -> Result<Paginated<MonthlyRentalVehicle>, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    let where_clause = build_monthly_where_clause(&filter);

    let total: i64 = conn.query_row(
        &format!("SELECT COUNT(*) FROM monthly_rental_vehicles {}", where_clause),
        [],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;

    let offset = (filter.page - 1) * filter.per_page;
    let sql = format!(
        "SELECT * FROM monthly_rental_vehicles {} ORDER BY id LIMIT {} OFFSET {}",
        where_clause, filter.per_page, offset
    );

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let items = stmt
        .query_map([], map_monthly_rental)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(Paginated {
        items,
        total,
        page: filter.page,
        per_page: filter.per_page,
    })
}

pub fn clear_monthly_rentals() -> Result<usize, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;
    let count = conn.execute("DELETE FROM monthly_rental_vehicles", [])
        .map_err(|e| e.to_string())?;
    Ok(count)
}

fn map_free_car(row: &rusqlite::Row) -> Result<FreeCar> {
    Ok(FreeCar {
        id: row.get(0)?,
        plate: row.get(1)?,
        parking_space: row.get(2)?,
        vehicle_type: row.get(3)?,
        owner_phone: row.get(4)?,
        owner_name: row.get(5)?,
        owner_address: row.get(6)?,
        start_date: row.get(7)?,
        end_date: row.get(8)?,
        parking_group: row.get(9)?,
        parking_space_count: row.get(10)?,
        source_file: row.get(11)?,
        building: row.get(12)?,
        unit: row.get(13)?,
        room: row.get(14)?,
    })
}

pub fn save_free_cars(req: SaveFreeCarsRequest) -> Result<usize, String> {
    let db = db();
    let mut conn = db.lock().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    if req.replace {
        tx.execute("DELETE FROM free_cars", [])
            .map_err(|e| e.to_string())?;
    }

    {
        let mut stmt = tx.prepare(
            "INSERT INTO free_cars (
                plate, parking_space, vehicle_type, owner_phone, owner_name, owner_address,
                start_date, end_date, parking_group, parking_space_count, source_file,
                building, unit, room
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14
            )"
        ).map_err(|e| e.to_string())?;

        for r in &req.records {
            let (building, unit, room) = parse_building_unit_room(&r.owner_address);
            stmt.execute(rusqlite::params![
                r.plate.trim(), r.parking_space, r.vehicle_type, r.owner_phone, r.owner_name, r.owner_address,
                r.start_date, r.end_date, r.parking_group, r.parking_space_count, r.source_file,
                building, unit, room
            ]).map_err(|e| e.to_string())?;
        }
    }

    tx.commit().map_err(|e| e.to_string())?;
    Ok(req.records.len())
}

pub fn list_free_cars(filter: ListFreeCarsFilter) -> Result<Paginated<FreeCar>, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    let where_clause = build_monthly_where_clause(&ListMonthlyRentalsFilter {
        plate: filter.plate,
        owner_name: filter.owner_name,
        vehicle_type: filter.vehicle_type,
        start_date_start: filter.start_date_start,
        start_date_end: filter.start_date_end,
        end_date_start: filter.end_date_start,
        end_date_end: filter.end_date_end,
        page: filter.page,
        per_page: filter.per_page,
    });

    let total: i64 = conn.query_row(
        &format!("SELECT COUNT(*) FROM free_cars {}", where_clause),
        [],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;

    let offset = (filter.page - 1) * filter.per_page;
    let sql = format!(
        "SELECT * FROM free_cars {} ORDER BY id LIMIT {} OFFSET {}",
        where_clause, filter.per_page, offset
    );

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let items = stmt
        .query_map([], map_free_car)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(Paginated {
        items,
        total,
        page: filter.page,
        per_page: filter.per_page,
    })
}

pub fn clear_free_cars() -> Result<usize, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;
    let count = conn.execute("DELETE FROM free_cars", [])
        .map_err(|e| e.to_string())?;
    Ok(count)
}

pub fn update_monthly_rental(record: MonthlyRentalVehicle) -> Result<usize, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;
    let (building, unit, room) = parse_building_unit_room(&record.owner_address);
    let count = conn.execute(
        "UPDATE monthly_rental_vehicles SET
            plate = ?1, parking_space = ?2, vehicle_type = ?3, owner_phone = ?4,
            owner_name = ?5, owner_address = ?6, start_date = ?7, end_date = ?8,
            parking_group = ?9, parking_space_count = ?10, source_file = ?11,
            building = ?12, unit = ?13, room = ?14
        WHERE id = ?15",
        rusqlite::params![
            record.plate.trim(), record.parking_space, record.vehicle_type,
            record.owner_phone, record.owner_name, record.owner_address,
            record.start_date, record.end_date, record.parking_group,
            record.parking_space_count, record.source_file,
            building, unit, room, record.id
        ]
    ).map_err(|e| e.to_string())?;
    Ok(count)
}

pub fn update_free_car(record: FreeCar) -> Result<usize, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;
    let (building, unit, room) = parse_building_unit_room(&record.owner_address);
    let count = conn.execute(
        "UPDATE free_cars SET
            plate = ?1, parking_space = ?2, vehicle_type = ?3, owner_phone = ?4,
            owner_name = ?5, owner_address = ?6, start_date = ?7, end_date = ?8,
            parking_group = ?9, parking_space_count = ?10, source_file = ?11,
            building = ?12, unit = ?13, room = ?14
        WHERE id = ?15",
        rusqlite::params![
            record.plate.trim(), record.parking_space, record.vehicle_type,
            record.owner_phone, record.owner_name, record.owner_address,
            record.start_date, record.end_date, record.parking_group,
            record.parking_space_count, record.source_file,
            building, unit, room, record.id
        ]
    ).map_err(|e| e.to_string())?;
    Ok(count)
}

fn map_monthly_card_ledger(row: &rusqlite::Row) -> Result<MonthlyCardLedger> {
    Ok(MonthlyCardLedger {
        id: row.get(0)?,
        seq: row.get(1)?,
        process_date: row.get(2)?,
        room_number: row.get(3)?,
        owner_name: row.get(4)?,
        owner_phone: row.get(5)?,
        plate: row.get(6)?,
        payment_period: row.get(7)?,
        payment_months: row.get(8)?,
        rent: row.get(9)?,
        management_fee: row.get(10)?,
        actual_start_date: row.get(11)?,
        actual_end_date: row.get(12)?,
        actual_months: row.get(13)?,
        rent_gifted: row.get::<_, i64>(14)? != 0,
        property_fee_owed: row.get::<_, i64>(15)? != 0,
        operator: row.get(16)?,
        remark: row.get(17)?,
        source_file: row.get(18)?,
    })
}

pub fn save_monthly_card_ledgers(req: SaveMonthlyCardLedgersRequest) -> Result<usize, String> {
    let db = db();
    let mut conn = db.lock().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    if req.replace {
        tx.execute("DELETE FROM monthly_card_ledger", [])
            .map_err(|e| e.to_string())?;
    }

    {
        let mut stmt = tx.prepare(
            "INSERT INTO monthly_card_ledger (
                seq, process_date, room_number, owner_name, owner_phone,
                plate, payment_period, payment_months, rent, management_fee,
                actual_start_date, actual_end_date, actual_months,
                rent_gifted, property_fee_owed, operator, remark, source_file
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10,
                ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18
            )"
        ).map_err(|e| e.to_string())?;

        for r in &req.records {
            stmt.execute(rusqlite::params![
                r.seq, r.process_date, r.room_number, r.owner_name, r.owner_phone,
                r.plate.trim(), r.payment_period, r.payment_months, r.rent, r.management_fee,
                r.actual_start_date, r.actual_end_date, r.actual_months,
                if r.rent_gifted { 1 } else { 0 },
                if r.property_fee_owed { 1 } else { 0 },
                r.operator, r.remark, r.source_file
            ]).map_err(|e| e.to_string())?;
        }
    }

    tx.commit().map_err(|e| e.to_string())?;
    Ok(req.records.len())
}

fn build_monthly_card_ledger_where_clause(filter: &ListMonthlyCardLedgerFilter) -> String {
    let mut conditions: Vec<String> = vec![];
    if let Some(s) = &filter.plate {
        conditions.push(format!("TRIM(plate) LIKE '%{}%'", s.replace("'", "''")));
    }
    if let Some(s) = &filter.owner_name {
        conditions.push(format!("owner_name LIKE '%{}%'", s.replace("'", "''")));
    }
    if let Some(s) = &filter.room_number {
        conditions.push(format!("room_number LIKE '%{}%'", s.replace("'", "''")));
    }
    if let Some(s) = &filter.process_date_start {
        conditions.push(format!("process_date >= '{}'", s.replace("'", "''")));
    }
    if let Some(s) = &filter.process_date_end {
        conditions.push(format!("process_date <= '{}'", s.replace("'", "''")));
    }
    if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    }
}

pub fn list_monthly_card_ledgers(filter: ListMonthlyCardLedgerFilter) -> Result<Paginated<MonthlyCardLedger>, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    let where_clause = build_monthly_card_ledger_where_clause(&filter);

    let total: i64 = conn.query_row(
        &format!("SELECT COUNT(*) FROM monthly_card_ledger {}", where_clause),
        [],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;

    let offset = (filter.page - 1) * filter.per_page;
    let sql = format!(
        "SELECT * FROM monthly_card_ledger {} ORDER BY id LIMIT {} OFFSET {}",
        where_clause, filter.per_page, offset
    );

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let items = stmt
        .query_map([], map_monthly_card_ledger)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(Paginated {
        items,
        total,
        page: filter.page,
        per_page: filter.per_page,
    })
}

pub fn clear_monthly_card_ledgers() -> Result<usize, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;
    let count = conn.execute("DELETE FROM monthly_card_ledger", [])
        .map_err(|e| e.to_string())?;
    Ok(count)
}

fn clean_owner_addresses(table: &str) -> Result<usize, String> {
    use once_cell::sync::Lazy;
    use regex::Regex;

    static ADDRESS_START: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(\d+-\d+(?:-\d+)?)\s*(.*?)\s*(租户)?$").unwrap());
    static ADDRESS_END: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(.*?)\s*(\d+-\d+(?:-\d+)?)\s*(租户)?$").unwrap());
    static DONG_UNIT: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(\d+)栋(\d+)单元(\d+)$").unwrap());
    static ADDRESS_SUFFIX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(\d+-\d+(?:-\d+)?)(租户)$").unwrap());

    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    let select_sql = format!("SELECT id, owner_name, owner_address FROM {}", table);
    let mut stmt = conn.prepare(&select_sql)
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    drop(stmt);

    let update_sql = format!("UPDATE {} SET owner_name = ?1, owner_address = ?2, building = ?3, unit = ?4, room = ?5 WHERE id = ?6", table);
    let mut updated = 0;
    let mut update_stmt = conn.prepare(&update_sql).map_err(|e| e.to_string())?;

    for (id, mut name, mut address) in rows {
        let original_name = name.clone();
        let original_address = address.clone();

        // 1. 地址为空时，从车主姓名中提取地址
        if address.is_empty() {
            if let Some(caps) = ADDRESS_START.captures(&name) {
                let extracted_addr = caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
                let remaining = caps.get(2).map(|m| m.as_str().trim().to_string()).unwrap_or_default();
                let suffix = caps.get(3).map(|m| m.as_str().to_string()).unwrap_or_default();
                address = extracted_addr;
                name = remaining;
                if suffix == "租户" && !name.ends_with("租户") {
                    name = format!("{}租户", name);
                }
            } else if let Some(caps) = ADDRESS_END.captures(&name) {
                let extracted_addr = caps.get(2).map(|m| m.as_str().to_string()).unwrap_or_default();
                let remaining = caps.get(1).map(|m| m.as_str().trim().to_string()).unwrap_or_default();
                let suffix = caps.get(3).map(|m| m.as_str().to_string()).unwrap_or_default();
                address = extracted_addr;
                name = remaining;
                if suffix == "租户" && !name.ends_with("租户") {
                    name = format!("{}租户", name);
                }
            }
        }

        // 2. 将 "11栋1单元1001" 转换为 "11-1-1001"
        if let Some(caps) = DONG_UNIT.captures(&address) {
            address = format!("{}-{}-{}", &caps[1], &caps[2], &caps[3]);
        }

        // 3. 将 "7-1-703租户" 拆分，地址保留 "7-1-703"，"租户" 追加到车主姓名
        if let Some(caps) = ADDRESS_SUFFIX.captures(&address) {
            address = caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
            if !name.ends_with("租户") {
                name = format!("{}租户", name);
            }
        }

        let (building, unit, room) = parse_building_unit_room(&address);
        if name != original_name || address != original_address || building != "" || unit != "" || room != "" {
            update_stmt.execute(rusqlite::params![name, address, building, unit, room, id])
                .map_err(|e| e.to_string())?;
            updated += 1;
        }
    }

    Ok(updated)
}

pub fn clean_free_car_addresses() -> Result<usize, String> {
    clean_owner_addresses("free_cars")
}

pub fn clean_monthly_rental_addresses() -> Result<usize, String> {
    clean_owner_addresses("monthly_rental_vehicles")
}

fn build_building_charges_where(filter: &BuildingChargesFilter) -> String {
    let mut conditions: Vec<String> = vec![];

    if let Some(s) = &filter.building {
        if !s.is_empty() {
            conditions.push(format!("building = '{}'", s.replace("'", "''")));
        }
    }
    if let Some(s) = &filter.unit {
        if !s.is_empty() {
            conditions.push(format!("unit = '{}'", s.replace("'", "''")));
        }
    }
    if let Some(s) = &filter.room {
        if !s.is_empty() {
            conditions.push(format!("room = '{}'", s.replace("'", "''")));
        }
    }

    if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    }
}

fn build_parking_time_conditions(
    entry_time_start: &Option<String>,
    entry_time_end: &Option<String>,
    exit_time_start: &Option<String>,
    exit_time_end: &Option<String>,
    alias: &str,
) -> String {
    let mut conditions: Vec<String> = vec![];
    if let Some(s) = entry_time_start {
        if !s.is_empty() {
            conditions.push(format!("{}entry_time >= '{}'", alias, s.replace("'", "''")));
        }
    }
    if let Some(s) = entry_time_end {
        if !s.is_empty() {
            conditions.push(format!("{}entry_time <= '{}'", alias, s.replace("'", "''")));
        }
    }
    if let Some(s) = exit_time_start {
        if !s.is_empty() {
            conditions.push(format!("{}exit_time >= '{}'", alias, s.replace("'", "''")));
        }
    }
    if let Some(s) = exit_time_end {
        if !s.is_empty() {
            conditions.push(format!("{}exit_time <= '{}'", alias, s.replace("'", "''")));
        }
    }
    if conditions.is_empty() {
        String::new()
    } else {
        format!("AND {}", conditions.join(" AND "))
    }
}

pub fn get_building_charges(filter: BuildingChargesFilter) -> Result<Paginated<BuildingChargeItem>, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    let where_clause = build_building_charges_where(&filter);
    let time_conditions = build_parking_time_conditions(
        &filter.entry_time_start,
        &filter.entry_time_end,
        &filter.exit_time_start,
        &filter.exit_time_end,
        "pr.",
    );

    let page = std::cmp::max(filter.page.unwrap_or(1), 1);
    let per_page = std::cmp::max(filter.per_page.unwrap_or(20), 1);
    let offset = (page - 1) * per_page;

    let count_sql = format!(
        r#"
        WITH matched_vehicles AS (
            SELECT building, unit, room, TRIM(plate) AS plate FROM monthly_rental_vehicles {}
            UNION
            SELECT building, unit, room, TRIM(plate) AS plate FROM free_cars {}
        )
        SELECT COUNT(*) FROM (
            SELECT mv.building, mv.unit, mv.room
            FROM matched_vehicles mv
            LEFT JOIN parking_records pr ON TRIM(pr.plate) = mv.plate {}
            GROUP BY mv.building, mv.unit, mv.room
        )
        "#,
        where_clause, where_clause, time_conditions
    );
    let total: i64 = conn.query_row(&count_sql, [], |row| row.get(0)).map_err(|e| e.to_string())?;

    let sql = format!(
        r#"
        WITH matched_vehicles AS (
            SELECT building, unit, room, TRIM(plate) AS plate FROM monthly_rental_vehicles {}
            UNION
            SELECT building, unit, room, TRIM(plate) AS plate FROM free_cars {}
        )
        SELECT
            mv.building,
            mv.unit,
            mv.room,
            COUNT(DISTINCT mv.plate) AS plate_count,
            COUNT(pr.id) AS total_count,
            COALESCE(SUM(pr.due_amount), 0) AS total_due,
            COALESCE(SUM(pr.paid_amount), 0) AS total_paid,
            COALESCE(SUM(pr.due_amount - pr.paid_amount), 0) AS total_unpaid
        FROM matched_vehicles mv
        LEFT JOIN parking_records pr ON TRIM(pr.plate) = mv.plate {}
        GROUP BY mv.building, mv.unit, mv.room
        ORDER BY mv.building, mv.unit, mv.room
        LIMIT {} OFFSET {}
        "#,
        where_clause, where_clause, time_conditions, per_page, offset
    );

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let items = stmt
        .query_map([], |row| {
            Ok(BuildingChargeItem {
                building: row.get(0)?,
                unit: row.get(1)?,
                room: row.get(2)?,
                plate_count: row.get(3)?,
                total_count: row.get(4)?,
                total_due: row.get(5)?,
                total_paid: row.get(6)?,
                total_unpaid: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(Paginated {
        items,
        total,
        page,
        per_page,
    })
}

pub fn get_building_charge_detail(filter: BuildingChargesFilter) -> Result<Vec<BuildingChargePlateDetail>, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    let where_clause = build_building_charges_where(&filter);

    let sql = format!(
        r#"
        SELECT DISTINCT TRIM(plate) AS plate FROM (
            SELECT plate FROM monthly_rental_vehicles {}
            UNION
            SELECT plate FROM free_cars {}
        )
        ORDER BY plate
        "#,
        where_clause, where_clause
    );

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let plates: Vec<String> = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    drop(stmt);
    drop(conn);

    let mut details = Vec::new();
    for plate in plates {
        let plate = plate.trim().to_string();
        match get_vehicle_monthly_stats(VehicleMonthlyFilter {
            plate: plate.clone(),
            entry_time_start: filter.entry_time_start.clone(),
            entry_time_end: filter.entry_time_end.clone(),
            exit_time_start: filter.exit_time_start.clone(),
            exit_time_end: filter.exit_time_end.clone(),
        }) {
            Ok(items) => {
                if !items.is_empty() {
                    details.push(BuildingChargePlateDetail { plate, items });
                }
            }
            Err(e) => {
                eprintln!("get_building_charge_detail: 统计车牌 {} 失败: {}", plate, e);
            }
        }
    }

    Ok(details)
}

pub fn list_vehicle_records(filter: VehicleRecordsFilter) -> Result<Paginated<ParkingRecord>, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;

    let where_clause = build_vehicle_where_clause(
        &filter.plate,
        &filter.entry_time_start,
        &filter.entry_time_end,
        &filter.exit_time_start,
        &filter.exit_time_end,
        Some(&filter.month),
    );

    let total: i64 = conn.query_row(
        &format!("SELECT COUNT(*) FROM parking_records {}", where_clause),
        [],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;

    let offset = (filter.page - 1) * filter.per_page;
    let sql = format!(
        "SELECT * FROM parking_records {} ORDER BY exit_time LIMIT {} OFFSET {}",
        where_clause, filter.per_page, offset
    );

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let items = stmt
        .query_map([], map_record)
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(Paginated {
        items,
        total,
        page: filter.page,
        per_page: filter.per_page,
    })
}

fn build_unpaid_interval_where(filter: &UnpaidIntervalFilter) -> String {
    let mut conditions: Vec<String> = vec![];
    if let Some(s) = &filter.entry_time_start {
        if !s.is_empty() {
            conditions.push(format!("entry_time >= '{}'", s.replace("'", "''")));
        }
    }
    if let Some(s) = &filter.entry_time_end {
        if !s.is_empty() {
            conditions.push(format!("entry_time <= '{}'", s.replace("'", "''")));
        }
    }
    if let Some(s) = &filter.exit_time_start {
        if !s.is_empty() {
            conditions.push(format!("exit_time >= '{}'", s.replace("'", "''")));
        }
    }
    if let Some(s) = &filter.exit_time_end {
        if !s.is_empty() {
            conditions.push(format!("exit_time <= '{}'", s.replace("'", "''")));
        }
    }
    if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    }
}

fn format_interval_range(n: usize, min: i64, max: i64) -> String {
    if n == 1 {
        if min == max {
            format!("{} 元", min)
        } else {
            format!("{} ~ {} 元", min, max)
        }
    } else if min == max {
        format!("{} 元", min)
    } else {
        format!("{} ~ {} 元", min, max)
    }
}

fn quantile_bin_details(
    mut vehicles: Vec<UnpaidIntervalVehicle>,
    n: usize,
) -> Vec<UnpaidIntervalDetail> {
    if vehicles.is_empty() || n == 0 {
        return vec![];
    }

    vehicles.sort_by_key(|v| v.unpaid);
    let total = vehicles.len();
    let base = total / n;
    let mut extra = total % n;

    let mut bins = Vec::with_capacity(n);
    let mut start = 0;

    for _ in 0..n {
        let size = base + if extra > 0 { 1 } else { 0 };
        if size == 0 {
            continue;
        }
        let end = (start + size).min(total);
        let min = vehicles[start].unpaid;
        let max = vehicles[end - 1].unpaid;

        bins.push(UnpaidIntervalDetail {
            range: format_interval_range(n, min, max),
            min,
            max: Some(max),
            vehicles: vehicles[start..end].to_vec(),
        });

        start = end;
        if extra > 0 {
            extra -= 1;
        }
    }

    bins
}

fn get_vehicle_unpaids(
    conn: &rusqlite::Connection,
    filter: &UnpaidIntervalFilter,
) -> Result<Vec<UnpaidIntervalVehicle>, String> {
    let where_clause = build_unpaid_interval_where(filter);
    let sql = format!(
        "SELECT plate, COALESCE(SUM(due_amount - paid_amount), 0) AS unpaid
         FROM parking_records {}
         GROUP BY plate
         ORDER BY unpaid",
        where_clause
    );

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let vehicles = stmt
        .query_map([], |row| {
            Ok(UnpaidIntervalVehicle {
                plate: row.get(0)?,
                unpaid: row.get(1)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(vehicles)
}

fn equal_width_bin_details(
    vehicles: Vec<UnpaidIntervalVehicle>,
    n: usize,
) -> Vec<UnpaidIntervalDetail> {
    if vehicles.is_empty() || n == 0 {
        return vec![];
    }

    let min = vehicles.iter().map(|v| v.unpaid).min().unwrap_or(0);
    let max = vehicles.iter().map(|v| v.unpaid).max().unwrap_or(0);

    if min == max {
        return vec![UnpaidIntervalDetail {
            range: format!("{} 元", min),
            min,
            max: Some(max),
            vehicles,
        }];
    }

    let width = ((max - min + n as i64 - 1) / n as i64).max(1);
    let mut bins: Vec<UnpaidIntervalDetail> = (0..n)
        .map(|i| {
            let bin_min = min + i as i64 * width;
            let bin_max = if i == n - 1 { max } else { bin_min + width - 1 };
            UnpaidIntervalDetail {
                range: format!("{} ~ {} 元", bin_min, bin_max),
                min: bin_min,
                max: Some(bin_max),
                vehicles: vec![],
            }
        })
        .collect();

    for vehicle in vehicles {
        let mut idx = ((vehicle.unpaid - min) / width) as usize;
        if idx >= n {
            idx = n - 1;
        }
        bins[idx].vehicles.push(vehicle);
    }

    bins.into_iter().filter(|b| !b.vehicles.is_empty()).collect()
}

pub fn get_unpaid_interval_details(
    filter: UnpaidIntervalFilter,
) -> Result<Vec<UnpaidIntervalDetail>, String> {
    let db = db();
    let conn = db.lock().map_err(|e| e.to_string())?;
    let vehicles = get_vehicle_unpaids(&conn, &filter)?;
    drop(conn);

    let positive_vehicles: Vec<UnpaidIntervalVehicle> =
        vehicles.into_iter().filter(|v| v.unpaid > 0).collect();
    let total_vehicles = positive_vehicles.len();

    let bins = std::cmp::max(filter.bins.unwrap_or(5), 1) as usize;
    let n = bins.min(total_vehicles.max(1));

    if filter.mode.as_deref() == Some("equal-width") {
        Ok(equal_width_bin_details(positive_vehicles, n))
    } else {
        Ok(quantile_bin_details(positive_vehicles, n))
    }
}

pub fn get_unpaid_interval_stats(filter: UnpaidIntervalFilter) -> Result<UnpaidIntervalStats, String> {
    let details = get_unpaid_interval_details(filter)?;

    let total_vehicles: i64 = details.iter().map(|d| d.vehicles.len() as i64).sum();
    let total_unpaid: i64 = details
        .iter()
        .flat_map(|d| d.vehicles.iter().map(|v| v.unpaid))
        .sum();

    let all_unpaids: Vec<i64> = details
        .iter()
        .flat_map(|d| d.vehicles.iter().map(|v| v.unpaid))
        .collect();

    let mean = if total_vehicles > 0 {
        total_unpaid as f64 / total_vehicles as f64
    } else {
        0.0
    };

    let median = if all_unpaids.is_empty() {
        0.0
    } else {
        let mid = all_unpaids.len() / 2;
        if all_unpaids.len() % 2 == 0 {
            (all_unpaids[mid - 1] as f64 + all_unpaids[mid] as f64) / 2.0
        } else {
            all_unpaids[mid] as f64
        }
    };

    let variance = if total_vehicles > 1 {
        all_unpaids
            .iter()
            .map(|v| {
                let diff = *v as f64 - mean;
                diff * diff
            })
            .sum::<f64>()
            / (total_vehicles as f64 - 1.0)
    } else {
        0.0
    };
    let std_dev = variance.sqrt();

    let min = all_unpaids.first().copied().unwrap_or(0);
    let max = all_unpaids.last().copied().unwrap_or(0);

    let intervals = details
        .into_iter()
        .map(|d| UnpaidIntervalItem {
            range: d.range,
            min: d.min,
            max: d.max,
            vehicle_count: d.vehicles.len() as i64,
            total_unpaid: d.vehicles.iter().map(|v| v.unpaid).sum(),
        })
        .collect();

    Ok(UnpaidIntervalStats {
        intervals,
        total_vehicles,
        total_unpaid,
        mean,
        median,
        std_dev,
        min,
        max,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static DB_TEST_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn test_db_flow() {
        let _lock = DB_TEST_LOCK.lock().unwrap();
        let temp = tempfile::tempdir().unwrap();
        init_db(temp.path().to_path_buf()).unwrap();

        let records = vec![
            ParkingRecord {
                id: 0,
                plate: "川A12345".to_string(),
                charge_type: "小型车".to_string(),
                vehicle_type: "临时车".to_string(),
                vehicle_model: "轿车".to_string(),
                vehicle_brand: "大众".to_string(),
                vehicle_series: "速腾".to_string(),
                owner_phone: "13800000000".to_string(),
                owner_name: "张三".to_string(),
                entry_time: "2026-10-28 08:00:00".to_string(),
                entry_lane: "1号门入口".to_string(),
                exit_time: "2026-10-28 10:00:00".to_string(),
                exit_lane: "1号门出口".to_string(),
                duration: "02:00:00".to_string(),
                duration_minutes: 120.0,
                due_amount: 10,
                paid_amount: 10,
                coupon_type: "--".to_string(),
                coupon_discount: 0,
                charge_discount_type: "--".to_string(),
                charge_discount_amount: 0,
                payment_method: "--".to_string(),
                release_method: "自动放行".to_string(),
                operator: "system".to_string(),
                remark: "".to_string(),
                source_file: "test.xlsx".to_string(),
            },
        ];

        let saved = save_records(SaveRecordsRequest { records, replace: true }).unwrap();
        assert_eq!(saved, 1);

        let result = list_records(ListRecordsFilter {
            plate: None,
            owner_name: None,
            vehicle_type: None,
            charge_type: None,
            release_method: None,
            payment_method: None,
            entry_time_start: None,
            entry_time_end: None,
            exit_time_start: None,
            exit_time_end: None,
            page: 1,
            per_page: 10,
        })
        .unwrap();
        assert_eq!(result.total, 1);
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.items[0].plate, "川A12345");

        let stats = get_stats().unwrap();
        assert_eq!(stats.summary.total_count, 1);
        assert_eq!(stats.summary.total_due, 10);
        assert_eq!(stats.summary.total_paid, 10);
    }

    #[test]
    fn test_clean_free_car_addresses() {
        let _lock = DB_TEST_LOCK.lock().unwrap();
        let temp = tempfile::tempdir().unwrap();
        init_db(temp.path().to_path_buf()).unwrap();

        // 清空已有免费车记录，确保测试不受其他测试数据影响
        clear_free_cars().unwrap();

        let records = vec![
            FreeCar {
                id: 0,
                plate: "川A00001".to_string(),
                parking_space: "".to_string(),
                vehicle_type: "免费车".to_string(),
                owner_phone: "13800000000".to_string(),
                owner_name: "10-2-1103况志航".to_string(),
                owner_address: "".to_string(),
                start_date: "2026-01-01".to_string(),
                end_date: "2026-12-31".to_string(),
                parking_group: "".to_string(),
                parking_space_count: 0,
                source_file: "test.xlsx".to_string(),
                building: "".to_string(),
                unit: "".to_string(),
                room: "".to_string(),
            },
            FreeCar {
                id: 0,
                plate: "川A00002".to_string(),
                parking_space: "".to_string(),
                vehicle_type: "免费车".to_string(),
                owner_phone: "13800000000".to_string(),
                owner_name: "曾兵7-1-202".to_string(),
                owner_address: "".to_string(),
                start_date: "2026-01-01".to_string(),
                end_date: "2026-12-31".to_string(),
                parking_group: "".to_string(),
                parking_space_count: 0,
                source_file: "test.xlsx".to_string(),
                building: "".to_string(),
                unit: "".to_string(),
                room: "".to_string(),
            },
            FreeCar {
                id: 0,
                plate: "川A00003".to_string(),
                parking_space: "".to_string(),
                vehicle_type: "免费车".to_string(),
                owner_phone: "13800000000".to_string(),
                owner_name: "张三".to_string(),
                owner_address: "11栋1单元1001".to_string(),
                start_date: "2026-01-01".to_string(),
                end_date: "2026-12-31".to_string(),
                parking_group: "".to_string(),
                parking_space_count: 0,
                source_file: "test.xlsx".to_string(),
                building: "".to_string(),
                unit: "".to_string(),
                room: "".to_string(),
            },
            FreeCar {
                id: 0,
                plate: "川A00004".to_string(),
                parking_space: "".to_string(),
                vehicle_type: "免费车".to_string(),
                owner_phone: "13800000000".to_string(),
                owner_name: "李四".to_string(),
                owner_address: "7-1-703租户".to_string(),
                start_date: "2026-01-01".to_string(),
                end_date: "2026-12-31".to_string(),
                parking_group: "".to_string(),
                parking_space_count: 0,
                source_file: "test.xlsx".to_string(),
                building: "".to_string(),
                unit: "".to_string(),
                room: "".to_string(),
            },
            FreeCar {
                id: 0,
                plate: "川A00005".to_string(),
                parking_space: "".to_string(),
                vehicle_type: "免费车".to_string(),
                owner_phone: "13800000000".to_string(),
                owner_name: "朱致恒1-1-1801租户".to_string(),
                owner_address: "".to_string(),
                start_date: "2026-01-01".to_string(),
                end_date: "2026-12-31".to_string(),
                parking_group: "".to_string(),
                parking_space_count: 0,
                source_file: "test.xlsx".to_string(),
                building: "".to_string(),
                unit: "".to_string(),
                room: "".to_string(),
            },
        ];

        save_free_cars(SaveFreeCarsRequest { records, replace: true }).unwrap();
        let updated = clean_free_car_addresses().unwrap();
        assert_eq!(updated, 5);

        let result = list_free_cars(ListFreeCarsFilter {
            plate: None,
            owner_name: None,
            vehicle_type: None,
            start_date_start: None,
            start_date_end: None,
            end_date_start: None,
            end_date_end: None,
            page: 1,
            per_page: 100,
        }).unwrap();

        let by_plate: std::collections::HashMap<String, FreeCar> = result
            .items
            .into_iter()
            .map(|r| (r.plate.clone(), r))
            .collect();

        let r1 = by_plate.get("川A00001").unwrap();
        assert_eq!(r1.owner_address, "10-2-1103");
        assert_eq!(r1.owner_name, "况志航");

        let r2 = by_plate.get("川A00002").unwrap();
        assert_eq!(r2.owner_address, "7-1-202");
        assert_eq!(r2.owner_name, "曾兵");

        let r3 = by_plate.get("川A00003").unwrap();
        assert_eq!(r3.owner_address, "11-1-1001");
        assert_eq!(r3.owner_name, "张三");

        let r4 = by_plate.get("川A00004").unwrap();
        assert_eq!(r4.owner_address, "7-1-703");
        assert_eq!(r4.owner_name, "李四租户");

        let r5 = by_plate.get("川A00005").unwrap();
        assert_eq!(r5.owner_address, "1-1-1801");
        assert_eq!(r5.owner_name, "朱致恒租户");
    }

    #[test]
    fn test_clean_monthly_rental_addresses() {
        let _lock = DB_TEST_LOCK.lock().unwrap();
        let temp = tempfile::tempdir().unwrap();
        init_db(temp.path().to_path_buf()).unwrap();

        clear_monthly_rentals().unwrap();

        let records = vec![
            MonthlyRentalVehicle {
                id: 0,
                plate: "川B00001".to_string(),
                parking_space: "".to_string(),
                vehicle_type: "月租车".to_string(),
                owner_phone: "13800000000".to_string(),
                owner_name: "10-2-1103况志航".to_string(),
                owner_address: "".to_string(),
                start_date: "2026-01-01".to_string(),
                end_date: "2026-12-31".to_string(),
                parking_group: "".to_string(),
                parking_space_count: 0,
                source_file: "test.xlsx".to_string(),
                building: "".to_string(),
                unit: "".to_string(),
                room: "".to_string(),
            },
            MonthlyRentalVehicle {
                id: 0,
                plate: "川B00002".to_string(),
                parking_space: "".to_string(),
                vehicle_type: "月租车".to_string(),
                owner_phone: "13800000000".to_string(),
                owner_name: "张三".to_string(),
                owner_address: "11栋1单元1001".to_string(),
                start_date: "2026-01-01".to_string(),
                end_date: "2026-12-31".to_string(),
                parking_group: "".to_string(),
                parking_space_count: 0,
                source_file: "test.xlsx".to_string(),
                building: "".to_string(),
                unit: "".to_string(),
                room: "".to_string(),
            },
        ];

        save_monthly_rentals(SaveMonthlyRentalsRequest { records, replace: true }).unwrap();
        let updated = clean_monthly_rental_addresses().unwrap();
        assert_eq!(updated, 2);

        let result = list_monthly_rentals(ListMonthlyRentalsFilter {
            plate: None,
            owner_name: None,
            vehicle_type: None,
            start_date_start: None,
            start_date_end: None,
            end_date_start: None,
            end_date_end: None,
            page: 1,
            per_page: 100,
        }).unwrap();

        let by_plate: std::collections::HashMap<String, MonthlyRentalVehicle> = result
            .items
            .into_iter()
            .map(|r| (r.plate.clone(), r))
            .collect();

        let r1 = by_plate.get("川B00001").unwrap();
        assert_eq!(r1.owner_address, "10-2-1103");
        assert_eq!(r1.owner_name, "况志航");

        let r2 = by_plate.get("川B00002").unwrap();
        assert_eq!(r2.owner_address, "11-1-1001");
        assert_eq!(r2.owner_name, "张三");
    }

    #[test]
    fn test_building_charges() {
        let _lock = DB_TEST_LOCK.lock().unwrap();
        let temp = tempfile::tempdir().unwrap();
        init_db(temp.path().to_path_buf()).unwrap();

        clear_free_cars().unwrap();
        clear_monthly_rentals().unwrap();

        save_free_cars(SaveFreeCarsRequest {
            records: vec![FreeCar {
                id: 0,
                plate: "川C00001".to_string(),
                parking_space: "".to_string(),
                vehicle_type: "免费车".to_string(),
                owner_phone: "13800000000".to_string(),
                owner_name: "业主".to_string(),
                owner_address: "10-2-1103".to_string(),
                start_date: "2026-01-01".to_string(),
                end_date: "2026-12-31".to_string(),
                parking_group: "".to_string(),
                parking_space_count: 0,
                source_file: "test.xlsx".to_string(),
                building: "".to_string(),
                unit: "".to_string(),
                room: "".to_string(),
            }],
            replace: true,
        }).unwrap();

        save_records(SaveRecordsRequest {
            records: vec![
                ParkingRecord {
                    id: 0,
                    plate: "川C00001".to_string(),
                    charge_type: "小型车".to_string(),
                    vehicle_type: "临时车".to_string(),
                    vehicle_model: "".to_string(),
                    vehicle_brand: "".to_string(),
                    vehicle_series: "".to_string(),
                    owner_phone: "".to_string(),
                    owner_name: "".to_string(),
                    entry_time: "2026-09-01 08:00:00".to_string(),
                    entry_lane: "".to_string(),
                    exit_time: "2026-09-01 10:00:00".to_string(),
                    exit_lane: "".to_string(),
                    duration: "02:00:00".to_string(),
                    duration_minutes: 120.0,
                    due_amount: 100,
                    paid_amount: 80,
                    coupon_type: "".to_string(),
                    coupon_discount: 0,
                    charge_discount_type: "".to_string(),
                    charge_discount_amount: 0,
                    payment_method: "".to_string(),
                    release_method: "".to_string(),
                    operator: "".to_string(),
                    remark: "".to_string(),
                    source_file: "test.xlsx".to_string(),
                },
                ParkingRecord {
                    id: 0,
                    plate: "川C00001".to_string(),
                    charge_type: "小型车".to_string(),
                    vehicle_type: "临时车".to_string(),
                    vehicle_model: "".to_string(),
                    vehicle_brand: "".to_string(),
                    vehicle_series: "".to_string(),
                    owner_phone: "".to_string(),
                    owner_name: "".to_string(),
                    entry_time: "2026-09-02 08:00:00".to_string(),
                    entry_lane: "".to_string(),
                    exit_time: "".to_string(),
                    exit_lane: "".to_string(),
                    duration: "".to_string(),
                    duration_minutes: 0.0,
                    due_amount: 50,
                    paid_amount: 0,
                    coupon_type: "".to_string(),
                    coupon_discount: 0,
                    charge_discount_type: "".to_string(),
                    charge_discount_amount: 0,
                    payment_method: "".to_string(),
                    release_method: "".to_string(),
                    operator: "".to_string(),
                    remark: "".to_string(),
                    source_file: "test.xlsx".to_string(),
                },
            ],
            replace: true,
        }).unwrap();

        clean_free_car_addresses().unwrap();

        let result = get_building_charges(BuildingChargesFilter {
            building: Some("10".to_string()),
            unit: Some("2".to_string()),
            room: None,
            entry_time_start: None,
            entry_time_end: None,
            exit_time_start: None,
            exit_time_end: None,
            page: None,
            per_page: None,
        }).unwrap();

        assert_eq!(result.items.len(), 1);
        assert_eq!(result.total, 1);
        assert_eq!(result.page, 1);
        assert_eq!(result.items[0].building, "10");
        assert_eq!(result.items[0].unit, "2");
        assert_eq!(result.items[0].room, "1103");
        assert_eq!(result.items[0].plate_count, 1);
        assert_eq!(result.items[0].total_count, 2);
        assert_eq!(result.items[0].total_due, 150);
        assert_eq!(result.items[0].total_paid, 80);
        assert_eq!(result.items[0].total_unpaid, 70);

        let detail = get_building_charge_detail(BuildingChargesFilter {
            building: Some("10".to_string()),
            unit: Some("2".to_string()),
            room: Some("1103".to_string()),
            entry_time_start: None,
            entry_time_end: None,
            exit_time_start: None,
            exit_time_end: None,
            page: None,
            per_page: None,
        }).unwrap();

        assert_eq!(detail.len(), 1);
        assert_eq!(detail[0].plate, "川C00001");
        assert_eq!(detail[0].items.len(), 1);
        assert_eq!(detail[0].items[0].month, "2026-09");
        assert_eq!(detail[0].items[0].count, 1);
        assert_eq!(detail[0].items[0].due, 100);
        assert_eq!(detail[0].items[0].paid, 80);
        assert_eq!(detail[0].items[0].unpaid, 20);
    }
}
