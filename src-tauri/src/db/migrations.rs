pub const MIGRATIONS: &[(&str, &str)] = &[
    (
        "001_initial",
        r#"
        CREATE TABLE IF NOT EXISTS parking_records (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            plate TEXT NOT NULL,
            charge_type TEXT NOT NULL DEFAULT '',
            vehicle_type TEXT NOT NULL DEFAULT '',
            vehicle_model TEXT NOT NULL DEFAULT '',
            vehicle_brand TEXT NOT NULL DEFAULT '',
            vehicle_series TEXT NOT NULL DEFAULT '',
            owner_phone TEXT NOT NULL DEFAULT '',
            owner_name TEXT NOT NULL DEFAULT '',
            entry_time TEXT NOT NULL DEFAULT '',
            entry_lane TEXT NOT NULL DEFAULT '',
            exit_time TEXT NOT NULL DEFAULT '',
            exit_lane TEXT NOT NULL DEFAULT '',
            duration TEXT NOT NULL DEFAULT '',
            duration_minutes REAL NOT NULL DEFAULT 0,
            due_amount INTEGER NOT NULL DEFAULT 0,
            paid_amount INTEGER NOT NULL DEFAULT 0,
            coupon_type TEXT NOT NULL DEFAULT '',
            coupon_discount INTEGER NOT NULL DEFAULT 0,
            charge_discount_type TEXT NOT NULL DEFAULT '',
            charge_discount_amount INTEGER NOT NULL DEFAULT 0,
            payment_method TEXT NOT NULL DEFAULT '',
            release_method TEXT NOT NULL DEFAULT '',
            operator TEXT NOT NULL DEFAULT '',
            remark TEXT NOT NULL DEFAULT '',
            source_file TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
        );

        CREATE INDEX IF NOT EXISTS idx_parking_records_plate ON parking_records(plate);
        CREATE INDEX IF NOT EXISTS idx_parking_records_vehicle_type ON parking_records(vehicle_type);
        CREATE INDEX IF NOT EXISTS idx_parking_records_release_method ON parking_records(release_method);
        CREATE INDEX IF NOT EXISTS idx_parking_records_entry_time ON parking_records(entry_time);
        "#,
    ),
    (
        "002_add_stats_indexes",
        r#"
        CREATE INDEX IF NOT EXISTS idx_parking_records_charge_type ON parking_records(charge_type);
        CREATE INDEX IF NOT EXISTS idx_parking_records_payment_method ON parking_records(payment_method);
        CREATE INDEX IF NOT EXISTS idx_parking_records_exit_time ON parking_records(exit_time);
        CREATE INDEX IF NOT EXISTS idx_parking_records_exit_time_month ON parking_records(strftime('%Y-%m', exit_time));
        "#,
    ),
    (
        "003_add_monthly_rental_vehicles",
        r#"
        CREATE TABLE IF NOT EXISTS monthly_rental_vehicles (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            plate TEXT NOT NULL,
            parking_space TEXT NOT NULL DEFAULT '',
            vehicle_type TEXT NOT NULL DEFAULT '',
            owner_phone TEXT NOT NULL DEFAULT '',
            owner_name TEXT NOT NULL DEFAULT '',
            owner_address TEXT NOT NULL DEFAULT '',
            start_date TEXT NOT NULL DEFAULT '',
            end_date TEXT NOT NULL DEFAULT '',
            parking_group TEXT NOT NULL DEFAULT '',
            parking_space_count INTEGER NOT NULL DEFAULT 0,
            source_file TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
        );

        CREATE INDEX IF NOT EXISTS idx_monthly_rental_plate ON monthly_rental_vehicles(plate);
        CREATE INDEX IF NOT EXISTS idx_monthly_rental_vehicle_type ON monthly_rental_vehicles(vehicle_type);
        CREATE INDEX IF NOT EXISTS idx_monthly_rental_start_date ON monthly_rental_vehicles(start_date);
        CREATE INDEX IF NOT EXISTS idx_monthly_rental_end_date ON monthly_rental_vehicles(end_date);
        "#,
    ),
    (
        "004_add_free_cars",
        r#"
        CREATE TABLE IF NOT EXISTS free_cars (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            plate TEXT NOT NULL,
            parking_space TEXT NOT NULL DEFAULT '',
            vehicle_type TEXT NOT NULL DEFAULT '',
            owner_phone TEXT NOT NULL DEFAULT '',
            owner_name TEXT NOT NULL DEFAULT '',
            owner_address TEXT NOT NULL DEFAULT '',
            start_date TEXT NOT NULL DEFAULT '',
            end_date TEXT NOT NULL DEFAULT '',
            parking_group TEXT NOT NULL DEFAULT '',
            parking_space_count INTEGER NOT NULL DEFAULT 0,
            source_file TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
        );

        CREATE INDEX IF NOT EXISTS idx_free_cars_plate ON free_cars(plate);
        CREATE INDEX IF NOT EXISTS idx_free_cars_vehicle_type ON free_cars(vehicle_type);
        CREATE INDEX IF NOT EXISTS idx_free_cars_start_date ON free_cars(start_date);
        CREATE INDEX IF NOT EXISTS idx_free_cars_end_date ON free_cars(end_date);
        "#,
    ),
    (
        "005_add_building_unit_room",
        r#"
        ALTER TABLE monthly_rental_vehicles ADD COLUMN building TEXT NOT NULL DEFAULT '';
        ALTER TABLE monthly_rental_vehicles ADD COLUMN unit TEXT NOT NULL DEFAULT '';
        ALTER TABLE monthly_rental_vehicles ADD COLUMN room TEXT NOT NULL DEFAULT '';

        ALTER TABLE free_cars ADD COLUMN building TEXT NOT NULL DEFAULT '';
        ALTER TABLE free_cars ADD COLUMN unit TEXT NOT NULL DEFAULT '';
        ALTER TABLE free_cars ADD COLUMN room TEXT NOT NULL DEFAULT '';

        CREATE INDEX IF NOT EXISTS idx_monthly_rental_building ON monthly_rental_vehicles(building);
        CREATE INDEX IF NOT EXISTS idx_monthly_rental_unit ON monthly_rental_vehicles(unit);
        CREATE INDEX IF NOT EXISTS idx_monthly_rental_room ON monthly_rental_vehicles(room);
        CREATE INDEX IF NOT EXISTS idx_free_cars_building ON free_cars(building);
        CREATE INDEX IF NOT EXISTS idx_free_cars_unit ON free_cars(unit);
        CREATE INDEX IF NOT EXISTS idx_free_cars_room ON free_cars(room);
        "#,
    ),
    (
        "006_add_app_settings",
        r#"
        CREATE TABLE IF NOT EXISTS app_settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL DEFAULT ''
        );
        "#,
    ),
    (
        "007_add_monthly_card_ledger",
        r#"
        CREATE TABLE IF NOT EXISTS monthly_card_ledger (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            seq INTEGER NOT NULL DEFAULT 0,
            process_date TEXT NOT NULL DEFAULT '',
            room_number TEXT NOT NULL DEFAULT '',
            owner_name TEXT NOT NULL DEFAULT '',
            owner_phone TEXT NOT NULL DEFAULT '',
            plate TEXT NOT NULL DEFAULT '',
            payment_period TEXT NOT NULL DEFAULT '',
            payment_months INTEGER NOT NULL DEFAULT 0,
            rent INTEGER NOT NULL DEFAULT 0,
            management_fee INTEGER NOT NULL DEFAULT 0,
            actual_start_date TEXT NOT NULL DEFAULT '',
            actual_end_date TEXT NOT NULL DEFAULT '',
            actual_months INTEGER NOT NULL DEFAULT 0,
            rent_gifted INTEGER NOT NULL DEFAULT 0,
            property_fee_owed INTEGER NOT NULL DEFAULT 0,
            operator TEXT NOT NULL DEFAULT '',
            remark TEXT NOT NULL DEFAULT '',
            source_file TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
        );

        CREATE INDEX IF NOT EXISTS idx_monthly_card_ledger_plate ON monthly_card_ledger(plate);
        CREATE INDEX IF NOT EXISTS idx_monthly_card_ledger_room_number ON monthly_card_ledger(room_number);
        CREATE INDEX IF NOT EXISTS idx_monthly_card_ledger_owner_name ON monthly_card_ledger(owner_name);
        CREATE INDEX IF NOT EXISTS idx_monthly_card_ledger_process_date ON monthly_card_ledger(process_date);
        "#,
    ),
];
