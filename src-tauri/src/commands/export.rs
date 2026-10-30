use crate::db;
use crate::db::models::{ParkingRecord, VehicleMonthlyFilter, VehicleMonthlyItem, VehicleRecordsFilter};
use rust_xlsxwriter::{Format, FormatAlign, Workbook, XlsxError};
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
                worksheet.write_with_format(current, 2, item.due as f64 / 100.0, &money_format)?;
                worksheet.write_with_format(current, 3, item.paid as f64 / 100.0, &money_format)?;
                worksheet.write_with_format(current, 4, item.unpaid as f64 / 100.0, &money_format)?;
            }

            worksheet.write(current, 5, (i + 1) as u32)?;
            worksheet.write(current, 6, &record.plate)?;
            worksheet.write(current, 7, &record.entry_time)?;
            worksheet.write(current, 8, &record.exit_time)?;
            worksheet.write(current, 9, &record.duration)?;
            worksheet.write_with_format(current, 10, record.due_amount as f64 / 100.0, &money_format)?;
            worksheet.write_with_format(current, 11, record.paid_amount as f64 / 100.0, &money_format)?;
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
            worksheet.write_with_format(block_start, 2, item.due as f64 / 100.0, &money_format)?;
            worksheet.write_with_format(block_start, 3, item.paid as f64 / 100.0, &money_format)?;
            worksheet.write_with_format(block_start, 4, item.unpaid as f64 / 100.0, &money_format)?;
        }

        row += count as u32;
    }

    // 合计行
    worksheet.write_with_format(row, 0, "合计", &header_format)?;
    worksheet.write_with_format(row, 1, total_count, &header_format)?;
    worksheet.write_with_format(row, 2, total_due as f64 / 100.0, &money_format)?;
    worksheet.write_with_format(row, 3, total_paid as f64 / 100.0, &money_format)?;
    worksheet.write_with_format(row, 4, total_unpaid as f64 / 100.0, &money_format)?;

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
