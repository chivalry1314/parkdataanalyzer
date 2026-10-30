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
];
