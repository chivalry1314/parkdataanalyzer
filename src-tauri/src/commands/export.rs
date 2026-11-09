use crate::db;
use crate::db::models::{
    BuildingChargeItem, BuildingChargePlateDetail, BuildingChargesFilter, ParkingRecord,
    UnpaidIntervalDetail, UnpaidIntervalFilter, UnpaidIntervalItem, VehicleMonthlyFilter,
    VehicleMonthlyItem, VehicleRecordsFilter,
};
use rust_xlsxwriter::{Chart, ChartType, Format, FormatAlign, Workbook, XlsxError};
use std::path::PathBuf;

fn write_vehicle_charges_xlsx(
    path: &PathBuf,
    _stats: &[VehicleMonthlyItem],
    data: &[(VehicleMonthlyItem, Vec<ParkingRecord>)],
    filter: &VehicleMonthlyFilter,
) -> Result<(), XlsxError> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    const LEFT_COLS: u16 = 5;
    const DETAIL_COLS: u16 = 9;
    const TOTAL_COLS: u16 = LEFT_COLS + DETAIL_COLS;

    let title_format = Format::new()
        .set_bold()
        .set_font_size(14)
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter);
    let header_format = Format::new()
        .set_bold()
        .set_background_color(0xF1F5F9)
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter);
    let center_format = Format::new()
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter);
    let money_format = Format::new()
        .set_num_format("¥#,##0.00")
        .set_align(FormatAlign::Right)
        .set_align(FormatAlign::VerticalCenter);

    // 标题
    worksheet.merge_range(0, 0, 0, TOTAL_COLS - 1, "车辆收费统计", &title_format)?;

    // 车牌
    worksheet.write(1, 0, "车牌号")?;
    worksheet.merge_range(1, 1, 1, TOTAL_COLS - 1, &filter.plate, &center_format)?;

    // 时间条件
    let mut time_parts = Vec::new();
    if let Some(s) = &filter.entry_time_start {
        time_parts.push(format!("进场时间起：{}", s));
    }
    if let Some(s) = &filter.entry_time_end {
        time_parts.push(format!("进场时间止：{}", s));
    }
    if let Some(s) = &filter.exit_time_start {
        time_parts.push(format!("出场时间起：{}", s));
    }
    if let Some(s) = &filter.exit_time_end {
        time_parts.push(format!("出场时间止：{}", s));
    }
    let time_text = if time_parts.is_empty() {
        "时间条件：无".to_string()
    } else {
        time_parts.join("，")
    };
    worksheet.merge_range(2, 0, 2, TOTAL_COLS - 1, &time_text, &center_format)?;

    // 表头
    let headers = [
        "月份", "记录数", "应收金额", "实收金额", "未收金额",
        "序号", "车牌号", "进场时间", "出场时间", "停车时长",
        "应收金额", "实收金额", "放行方式", "来源文件",
    ];
    for (col, header) in headers.iter().enumerate() {
        worksheet.write_with_format(4, col as u16, *header, &header_format)?;
    }

    let mut row = 5u32;
    let mut total_count = 0i64;
    let mut total_due = 0i64;
    let mut total_paid = 0i64;
    let mut total_unpaid = 0i64;

    for (item, records) in data {
        total_count += item.count;
        total_due += item.due;
        total_paid += item.paid;
        total_unpaid += item.unpaid;

        let block_start = row;
        let count = records.len().max(1);

        for (i, record) in records.iter().enumerate() {
            let current = row + i as u32;
            if i == 0 {
                worksheet.write_with_format(current, 0, &item.month, &center_format)?;
                worksheet.write_with_format(current, 1, item.count, &center_format)?;
                worksheet.write_with_format(current, 2, item.due as f64, &money_format)?;
                worksheet.write_with_format(current, 3, item.paid as f64, &money_format)?;
                worksheet.write_with_format(current, 4, item.unpaid as f64, &money_format)?;
            }

            worksheet.write(current, 5, (i + 1) as u32)?;
            worksheet.write(current, 6, &record.plate)?;
            worksheet.write(current, 7, &record.entry_time)?;
            worksheet.write(current, 8, &record.exit_time)?;
            worksheet.write(current, 9, &record.duration)?;
            worksheet.write_with_format(current, 10, record.due_amount as f64, &money_format)?;
            worksheet.write_with_format(current, 11, record.paid_amount as f64, &money_format)?;
            worksheet.write(current, 12, &record.release_method)?;
            worksheet.write(current, 13, &record.source_file)?;
        }

        if records.len() > 1 {
            for c in 0..LEFT_COLS {
                worksheet.merge_range(
                    block_start,
                    c,
                    block_start + records.len() as u32 - 1,
                    c,
                    "",
                    &center_format,
                )?;
            }
            // 合并后重新写入实际值到合并单元格的左上角
            worksheet.write_with_format(block_start, 0, &item.month, &center_format)?;
            worksheet.write_with_format(block_start, 1, item.count, &center_format)?;
            worksheet.write_with_format(block_start, 2, item.due as f64, &money_format)?;
            worksheet.write_with_format(block_start, 3, item.paid as f64, &money_format)?;
            worksheet.write_with_format(block_start, 4, item.unpaid as f64, &money_format)?;
        }

        row += count as u32;
    }

    // 合计行
    worksheet.write_with_format(row, 0, "合计", &header_format)?;
    worksheet.write_with_format(row, 1, total_count, &header_format)?;
    worksheet.write_with_format(row, 2, total_due as f64, &money_format)?;
    worksheet.write_with_format(row, 3, total_paid as f64, &money_format)?;
    worksheet.write_with_format(row, 4, total_unpaid as f64, &money_format)?;

    // 列宽
    worksheet.set_column_width(0, 12.0)?;
    worksheet.set_column_width(1, 12.0)?;
    worksheet.set_column_width(2, 14.0)?;
    worksheet.set_column_width(3, 14.0)?;
    worksheet.set_column_width(4, 14.0)?;
    worksheet.set_column_width(5, 8.0)?;
    worksheet.set_column_width(6, 14.0)?;
    worksheet.set_column_width(7, 20.0)?;
    worksheet.set_column_width(8, 20.0)?;
    worksheet.set_column_width(9, 14.0)?;
    worksheet.set_column_width(10, 12.0)?;
    worksheet.set_column_width(11, 12.0)?;
    worksheet.set_column_width(12, 12.0)?;
    worksheet.set_column_width(13, 24.0)?;

    worksheet.set_freeze_panes(5, 0)?;

    workbook.save(path)?;
    Ok(())
}

#[tauri::command]
pub async fn export_vehicle_charges_command(
    file_path: String,
    filter: VehicleMonthlyFilter,
) -> Result<(), String> {
    let path = PathBuf::from(file_path);

    tokio::task::spawn_blocking(move || {
        let stats = db::get_vehicle_monthly_stats(filter.clone())?;
        let mut data: Vec<(VehicleMonthlyItem, Vec<ParkingRecord>)> = Vec::new();
        for item in &stats {
            let result = db::list_vehicle_records(VehicleRecordsFilter {
                plate: filter.plate.clone(),
                month: item.month.clone(),
                entry_time_start: filter.entry_time_start.clone(),
                entry_time_end: filter.entry_time_end.clone(),
                exit_time_start: filter.exit_time_start.clone(),
                exit_time_end: filter.exit_time_end.clone(),
                page: 1,
                per_page: 100000,
            })?;
            data.push((item.clone(), result.items));
        }

        write_vehicle_charges_xlsx(&path, &stats, &data, &filter)
            .map_err(|e| format!("生成 Excel 失败：{}", e))
    })
    .await
    .map_err(|e| format!("导出任务执行异常：{}", e))?
}

fn plate_summary(items: &[VehicleMonthlyItem]) -> VehicleMonthlyItem {
    items.iter().fold(
        VehicleMonthlyItem {
            month: String::new(),
            count: 0,
            due: 0,
            paid: 0,
            unpaid: 0,
        },
        |mut acc, item| {
            acc.count += item.count;
            acc.due += item.due;
            acc.paid += item.paid;
            acc.unpaid += item.unpaid;
            acc
        },
    )
}

fn write_building_charges_xlsx(
    path: &PathBuf,
    details: &[(BuildingChargeItem, Vec<BuildingChargePlateDetail>)],
) -> Result<(), XlsxError> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    let title_format = Format::new()
        .set_bold()
        .set_font_size(14)
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter);
    let header_format = Format::new()
        .set_bold()
        .set_background_color(0xF1F5F9)
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter);
    let center_format = Format::new()
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter);
    let money_format = Format::new()
        .set_num_format("¥#,##0.00")
        .set_align(FormatAlign::Right)
        .set_align(FormatAlign::VerticalCenter);

    // 标题
    worksheet.merge_range(0, 0, 0, 5, "楼栋收费统计", &title_format)?;

    // 表头
    let headers = ["楼栋", "车牌", "记录数", "应收金额", "实收金额", "欠费金额"];
    for (col, header) in headers.iter().enumerate() {
        worksheet.write_with_format(1, col as u16, *header, &header_format)?;
    }

    let mut row = 2u32;
    let mut total_count = 0i64;
    let mut total_due = 0i64;
    let mut total_paid = 0i64;
    let mut total_unpaid = 0i64;

    for (item, plates) in details {
        total_count += item.total_count;
        total_due += item.total_due;
        total_paid += item.total_paid;
        total_unpaid += item.total_unpaid;

        let block_start = row;
        let address = format!("{}-{}-{}", item.building, item.unit, item.room);
        let row_count = 1 + plates.len() as u32;

        if row_count > 1 {
            worksheet.merge_range(
                block_start,
                0,
                block_start + row_count - 1,
                0,
                &address,
                &center_format,
            )?;
        } else {
            worksheet.write_with_format(block_start, 0, &address, &center_format)?;
        }

        worksheet.write(block_start, 1, "合计")?;
        worksheet.write(block_start, 2, item.total_count)?;
        worksheet.write_with_format(block_start, 3, item.total_due as f64, &money_format)?;
        worksheet.write_with_format(block_start, 4, item.total_paid as f64, &money_format)?;
        worksheet.write_with_format(block_start, 5, item.total_unpaid as f64, &money_format)?;

        row += 1;

        for detail in plates {
            let summary = plate_summary(&detail.items);
            worksheet.write(row, 1, &detail.plate)?;
            worksheet.write(row, 2, summary.count)?;
            worksheet.write_with_format(row, 3, summary.due as f64, &money_format)?;
            worksheet.write_with_format(row, 4, summary.paid as f64, &money_format)?;
            worksheet.write_with_format(row, 5, summary.unpaid as f64, &money_format)?;
            row += 1;
        }
    }

    // 总计行
    worksheet.write_with_format(row, 0, "总计", &header_format)?;
    worksheet.write_with_format(row, 1, "", &header_format)?;
    worksheet.write_with_format(row, 2, total_count, &header_format)?;
    worksheet.write_with_format(row, 3, total_due as f64, &money_format)?;
    worksheet.write_with_format(row, 4, total_paid as f64, &money_format)?;
    worksheet.write_with_format(row, 5, total_unpaid as f64, &money_format)?;

    // 列宽
    worksheet.set_column_width(0, 16.0)?;
    worksheet.set_column_width(1, 16.0)?;
    worksheet.set_column_width(2, 12.0)?;
    worksheet.set_column_width(3, 14.0)?;
    worksheet.set_column_width(4, 14.0)?;
    worksheet.set_column_width(5, 14.0)?;

    worksheet.set_freeze_panes(2, 0)?;

    workbook.save(path)?;
    Ok(())
}

#[tauri::command]
pub async fn export_building_charges_command(
    file_path: String,
    filter: BuildingChargesFilter,
) -> Result<(), String> {
    let path = PathBuf::from(file_path);

    tokio::task::spawn_blocking(move || {
        let paginated = db::get_building_charges(BuildingChargesFilter {
            building: filter.building.clone(),
            unit: filter.unit.clone(),
            room: filter.room.clone(),
            entry_time_start: filter.entry_time_start.clone(),
            entry_time_end: filter.entry_time_end.clone(),
            exit_time_start: filter.exit_time_start.clone(),
            exit_time_end: filter.exit_time_end.clone(),
            page: Some(1),
            per_page: Some(100000),
        })?;
        let mut details: Vec<(BuildingChargeItem, Vec<BuildingChargePlateDetail>)> = Vec::new();
        for item in paginated.items {
            let plates = db::get_building_charge_detail(BuildingChargesFilter {
                building: Some(item.building.clone()),
                unit: Some(item.unit.clone()),
                room: Some(item.room.clone()),
                entry_time_start: filter.entry_time_start.clone(),
                entry_time_end: filter.entry_time_end.clone(),
                exit_time_start: filter.exit_time_start.clone(),
                exit_time_end: filter.exit_time_end.clone(),
                page: None,
                per_page: None,
            })?;
            details.push((item, plates));
        }

        write_building_charges_xlsx(&path, &details)
            .map_err(|e| format!("生成 Excel 失败：{}", e))
    })
    .await
    .map_err(|e| format!("导出任务执行异常：{}", e))?
}

fn write_unpaid_interval_stats_xlsx(
    path: &PathBuf,
    stats: &UnpaidIntervalItem,
    intervals: &[UnpaidIntervalItem],
    details: &[UnpaidIntervalDetail],
    filter: &UnpaidIntervalFilter,
) -> Result<(), XlsxError> {
    let title_format = Format::new()
        .set_bold()
        .set_font_size(14)
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter);
    let header_format = Format::new()
        .set_bold()
        .set_background_color(0xF1F5F9)
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter);
    let center_format = Format::new()
        .set_align(FormatAlign::Center)
        .set_align(FormatAlign::VerticalCenter);
    let right_format = Format::new()
        .set_align(FormatAlign::Right)
        .set_align(FormatAlign::VerticalCenter);
    let money_format = Format::new()
        .set_num_format("¥#,##0.00")
        .set_align(FormatAlign::Right)
        .set_align(FormatAlign::VerticalCenter);
    let percent_format = Format::new()
        .set_num_format("0.0%")
        .set_align(FormatAlign::Right)
        .set_align(FormatAlign::VerticalCenter);
    let wrap_format = Format::new()
        .set_align(FormatAlign::Left)
        .set_align(FormatAlign::VerticalCenter)
        .set_text_wrap();

    let is_equal_width = filter.mode.as_deref() == Some("equal-width");
    let summary_sheet_name = if is_equal_width {
        "等宽分箱汇总"
    } else {
        "等频分箱汇总"
    };
    let title_text = if is_equal_width {
        "欠费等宽分箱分析"
    } else {
        "欠费等频分箱分析"
    };
    let method_text = if is_equal_width {
        "分区方法：等宽分箱（Equal-width binning）。将车辆累计欠费金额按固定金额区间均匀划分，每档金额范围相同，便于观察金额边界处的车辆分布。"
    } else {
        "分区方法：等频分箱（Equal-frequency binning）。将车辆按累计欠费金额排序后，按车辆数平均分配到各区间，保证每档车辆数量大致相同，适用于欠费金额长尾分布的场景。"
    };

    let mut workbook = Workbook::new();
    workbook.add_worksheet();
    workbook.add_worksheet();
    workbook.add_worksheet();

    let data_range = {
        let worksheet = workbook.worksheet_from_index(0)?;
        worksheet.set_name(summary_sheet_name)?;

        // 标题
        worksheet.merge_range(0, 0, 0, 4, title_text, &title_format)?;

        // 汇总信息
        worksheet.write(1, 0, "欠费车辆数")?;
        worksheet.write(1, 1, stats.vehicle_count)?;
        worksheet.write(1, 2, "欠费总额")?;
        worksheet.write_with_format(1, 3, stats.total_unpaid as f64, &money_format)?;
        worksheet.write(1, 4, "区间档数")?;
        worksheet.write(1, 5, intervals.len() as u32)?;

        // 时间条件
        let mut time_parts = Vec::new();
        if let Some(s) = &filter.entry_time_start {
            time_parts.push(format!("进场时间起：{}", s));
        }
        if let Some(s) = &filter.entry_time_end {
            time_parts.push(format!("进场时间止：{}", s));
        }
        if let Some(s) = &filter.exit_time_start {
            time_parts.push(format!("出场时间起：{}", s));
        }
        if let Some(s) = &filter.exit_time_end {
            time_parts.push(format!("出场时间止：{}", s));
        }
        let time_text = if time_parts.is_empty() {
            "时间条件：无".to_string()
        } else {
            time_parts.join("，")
        };
        worksheet.merge_range(2, 0, 2, 4, &time_text, &center_format)?;

        // 方法说明
        worksheet.merge_range(3, 0, 3, 4, method_text, &wrap_format)?;
        worksheet.set_row_height(3, 48.0)?;

        // 表头
        let headers = ["欠费区间", "车辆数", "车辆占比", "欠费总额", "金额占比"];
        for (col, header) in headers.iter().enumerate() {
            worksheet.write_with_format(5, col as u16, *header, &header_format)?;
        }

        let total_count = stats.vehicle_count as f64;
        let total_unpaid = stats.total_unpaid as f64;

        let mut vehicle_ratio_sum = 0.0;
        let mut amount_ratio_sum = 0.0;

        let data_start_row = 6u32;
        let mut row = data_start_row;
        for (index, item) in intervals.iter().enumerate() {
            let is_last = index == intervals.len() - 1;
            let vehicle_ratio = if total_count <= 0.0 {
                0.0
            } else if is_last {
                f64::max(1.0 - vehicle_ratio_sum, 0.0)
            } else {
                item.vehicle_count as f64 / total_count
            };
            let amount_ratio = if total_unpaid <= 0.0 {
                0.0
            } else if is_last {
                f64::max(1.0 - amount_ratio_sum, 0.0)
            } else {
                item.total_unpaid as f64 / total_unpaid
            };
            vehicle_ratio_sum += vehicle_ratio;
            amount_ratio_sum += amount_ratio;

            worksheet.write_with_format(row, 0, &item.range, &center_format)?;
            worksheet.write_with_format(row, 1, item.vehicle_count, &right_format)?;
            worksheet.write_with_format(row, 2, vehicle_ratio, &percent_format)?;
            worksheet.write_with_format(row, 3, item.total_unpaid as f64, &money_format)?;
            worksheet.write_with_format(row, 4, amount_ratio, &percent_format)?;
            row += 1;
        }
        let data_end_row = row - 1;

        // 合计行
        worksheet.write_with_format(row, 0, "合计", &header_format)?;
        worksheet.write_with_format(row, 1, stats.vehicle_count, &header_format)?;
        worksheet.write_with_format(row, 2, 1.0, &percent_format)?;
        worksheet.write_with_format(row, 3, stats.total_unpaid as f64, &money_format)?;
        worksheet.write_with_format(row, 4, 1.0, &percent_format)?;

        // 列宽
        worksheet.set_column_width(0, 18.0)?;
        worksheet.set_column_width(1, 12.0)?;
        worksheet.set_column_width(2, 12.0)?;
        worksheet.set_column_width(3, 14.0)?;
        worksheet.set_column_width(4, 12.0)?;

        worksheet.set_freeze_panes(6, 0)?;

        (data_start_row, data_end_row)
    };

    // 插入柱状图到独立的 sheet
    if !intervals.is_empty() {
        let chart_sheet = workbook.worksheet_from_index(1)?;
        chart_sheet.set_name("车辆分布图")?;

        let mut chart = Chart::new(ChartType::Column);
        chart.title().set_name("车辆分布");
        chart.x_axis().set_name("欠费区间");
        chart.y_axis().set_name("车辆数");
        chart.set_width(640);
        chart.set_height(400);

        let (data_start_row, data_end_row) = data_range;
        let categories = (summary_sheet_name, data_start_row, 0, data_end_row, 0);
        let values = (summary_sheet_name, data_start_row, 1, data_end_row, 1);

        let series = chart.add_series();
        series.set_categories(categories);
        series.set_values(values);
        series.set_name("车辆数");

        chart_sheet.insert_chart(1, 1, &chart)?;
    }

    // 第三个 sheet：车辆明细
    {
        let detail_sheet = workbook.worksheet_from_index(2)?;
        detail_sheet.set_name("车辆明细")?;

        let detail_title = if is_equal_width {
            "等宽分箱车辆明细"
        } else {
            "等频分箱车辆明细"
        };
        detail_sheet.merge_range(0, 0, 0, 2, detail_title, &title_format)?;

        let detail_headers = ["欠费区间", "车牌号", "欠费金额"];
        for (col, header) in detail_headers.iter().enumerate() {
            detail_sheet.write_with_format(1, col as u16, *header, &header_format)?;
        }

        let mut row = 2u32;
        for detail in details {
            let block_start = row;
            for vehicle in &detail.vehicles {
                detail_sheet.write_with_format(row, 0, &detail.range, &center_format)?;
                detail_sheet.write_with_format(row, 1, &vehicle.plate, &center_format)?;
                detail_sheet.write_with_format(row, 2, vehicle.unpaid as f64, &money_format)?;
                row += 1;
            }
            let block_end = row - 1;
            if block_end > block_start {
                detail_sheet.merge_range(block_start, 0, block_end, 0, &detail.range, &center_format)?;
            }
        }

        detail_sheet.set_column_width(0, 18.0)?;
        detail_sheet.set_column_width(1, 16.0)?;
        detail_sheet.set_column_width(2, 14.0)?;
        detail_sheet.set_freeze_panes(2, 0)?;
    }

    workbook.save(path)?;
    Ok(())
}

#[tauri::command]
pub async fn export_unpaid_interval_stats_command(
    file_path: String,
    filter: UnpaidIntervalFilter,
) -> Result<(), String> {
    let path = PathBuf::from(file_path);

    tokio::task::spawn_blocking(move || {
        let stats = db::get_unpaid_interval_stats(filter.clone())?;
        let details = db::get_unpaid_interval_details(filter.clone())?;
        let summary = UnpaidIntervalItem {
            range: String::new(),
            min: 0,
            max: None,
            vehicle_count: stats.total_vehicles,
            total_unpaid: stats.total_unpaid,
        };
        write_unpaid_interval_stats_xlsx(&path, &summary, &stats.intervals, &details, &filter)
            .map_err(|e| format!("生成 Excel 失败：{}", e))
    })
    .await
    .map_err(|e| format!("导出任务执行异常：{}", e))?
}
