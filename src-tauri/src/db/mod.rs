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
            conn.execute_batch(sql)?;
            conn.execute("INSERT INTO migrations_applied (id) VALUES (?1)", [id])?;
        }
    }

    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
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
                r.plate, r.charge_type, r.vehicle_type, r.vehicle_model, r.vehicle_brand, r.vehicle_series,
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
        conditions.push(format!("plate LIKE '%{}%'", s.replace("'", "''")));
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
    let mut conditions: Vec<String> = vec![format!(
        "plate LIKE '%{}%'",
        plate.replace("'", "''")
    )];
    if let Some(s) = entry_time_start {
        conditions.push(format!("entry_time >= '{}'", s.replace("'", "''")));
    }
    if let Some(s) = entry_time_end {
        conditions.push(format!("entry_time <= '{}'", s.replace("'", "''")));
    }
    if let Some(s) = exit_time_start {
        conditions.push(format!("exit_time >= '{}'", s.replace("'", "''")));
    }
    if let Some(s) = exit_time_end {
        conditions.push(format!("exit_time <= '{}'", s.replace("'", "''")));
    }
    if let Some(m) = month {
        conditions.push(format!("strftime('%Y-%m', exit_time) = '{}'", m.replace("'", "''")));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_flow() {
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
            search: None,
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
}
