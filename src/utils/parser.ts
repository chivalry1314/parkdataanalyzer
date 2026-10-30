import * as XLSX from "xlsx";
import JSZip from "jszip";
import type { ParseResult, ParkingRecord } from "../types";

const HEADER_KEY = "车牌号";

const EXPECTED_HEADERS = [
  "车牌号",
  "收费车型",
  "车辆类型",
  "车型",
  "车辆品牌",
  "车系",
  "车主电话",
  "车主姓名",
  "进场时间",
  "进场车道",
  "出场时间",
  "出场车道",
  "停车时长",
  "应收金额",
  "实收金额",
  "领券类型",
  "领券优惠",
  "充电优惠类型",
  "充电优惠金额",
  "缴费方式",
  "放行方式",
  "操作者",
  "备注",
];

function parseMoney(value: unknown): number {
  if (value === null || value === undefined || value === "") return 0;
  const s = String(value).replace(/,/g, "").trim();
  if (s === "--" || s === "-" || s === "") return 0;
  const n = Number(s);
  return Number.isNaN(n) ? 0 : n;
}

function parseDurationMinutes(value: string): number {
  // formats: "02:58:28", "1天02:32:42"
  const s = String(value || "").trim();
  if (!s) return 0;
  let days = 0;
  let rest = s;
  const dayMatch = s.match(/(\d+)天/);
  if (dayMatch) {
    days = Number(dayMatch[1]);
    rest = s.replace(/\d+天/, "");
  }
  const parts = rest.split(":").map((p) => Number(p.trim()));
  let minutes = 0;
  if (parts.length === 3) {
    minutes = parts[0] * 60 + parts[1] + parts[2] / 60;
  } else if (parts.length === 2) {
    minutes = parts[0] + parts[1] / 60;
  }
  return days * 24 * 60 + minutes;
}

function normalizeHeader(value: unknown): string {
  return String(value || "").trim();
}

function findHeaderRow(rows: unknown[][]): { rowIndex: number; headers: string[] } | null {
  for (let i = 0; i < rows.length; i++) {
    const row = rows[i];
    if (!Array.isArray(row)) continue;
    const normalized = row.map(normalizeHeader);
    if (normalized.includes(HEADER_KEY)) {
      return { rowIndex: i, headers: normalized };
    }
  }
  return null;
}

function buildRecord(
  row: unknown[],
  headers: string[],
  sourceFile: string,
  id: number
): ParkingRecord | null {
  const get = (name: string): string => {
    const idx = headers.indexOf(name);
    if (idx === -1) return "";
    const v = row[idx];
    if (v === null || v === undefined) return "";
    return String(v).trim();
  };

  const plate = get("车牌号");
  if (!plate) return null;

  return {
    id,
    plate,
    chargeType: get("收费车型"),
    vehicleType: get("车辆类型"),
    vehicleModel: get("车型"),
    vehicleBrand: get("车辆品牌"),
    vehicleSeries: get("车系"),
    ownerPhone: get("车主电话"),
    ownerName: get("车主姓名"),
    entryTime: get("进场时间"),
    entryLane: get("进场车道"),
    exitTime: get("出场时间"),
    exitLane: get("出场车道"),
    duration: get("停车时长"),
    durationMinutes: parseDurationMinutes(get("停车时长")),
    dueAmount: parseMoney(row[headers.indexOf("应收金额")] ?? 0),
    paidAmount: parseMoney(row[headers.indexOf("实收金额")] ?? 0),
    couponType: get("领券类型"),
    couponDiscount: parseMoney(row[headers.indexOf("领券优惠")] ?? 0),
    chargeDiscountType: get("充电优惠类型"),
    chargeDiscountAmount: parseMoney(row[headers.indexOf("充电优惠金额")] ?? 0),
    paymentMethod: get("缴费方式"),
    releaseMethod: get("放行方式"),
    operator: get("操作者"),
    remark: get("备注"),
    sourceFile,
  };
}

export async function parseZipFile(
  data: Uint8Array,
  onProgress?: (current: number, total: number, fileName: string) => void
): Promise<ParseResult> {
  const zip = await JSZip.loadAsync(data);
  const records: ParkingRecord[] = [];
  const errors: string[] = [];
  let idCounter = 1;

  const xlsxFiles: { name: string; content: JSZip.JSZipObject }[] = [];
  zip.forEach((relativePath, file) => {
    if (!file.dir && relativePath.toLowerCase().endsWith(".xlsx")) {
      xlsxFiles.push({ name: relativePath.split("/").pop() || relativePath, content: file });
    }
  });

  xlsxFiles.sort((a, b) => a.name.localeCompare(b.name));
  const totalFiles = xlsxFiles.length;

  for (let i = 0; i < xlsxFiles.length; i++) {
    const { name, content } = xlsxFiles[i];
    onProgress?.(i + 1, totalFiles, name);
    try {
      const buffer = await content.async("uint8array");
      const workbook = XLSX.read(buffer, { type: "array", cellText: true, cellDates: false });
      const sheetName = workbook.SheetNames[0];
      const worksheet = workbook.Sheets[sheetName];
      const rows = XLSX.utils.sheet_to_json(worksheet, { header: 1, raw: false }) as unknown[][];

      const headerInfo = findHeaderRow(rows);
      if (!headerInfo) {
        errors.push(`文件 ${name} 中未找到表头，已跳过`);
        continue;
      }

      const { rowIndex, headers } = headerInfo;
      const missing = EXPECTED_HEADERS.filter((h) => !headers.includes(h));
      if (missing.length > 0) {
        errors.push(`文件 ${name} 缺少字段：${missing.join("、")}`);
      }

      for (let i = rowIndex + 1; i < rows.length; i++) {
        const row = rows[i];
        if (!Array.isArray(row) || row.every((c) => c === null || c === undefined || String(c).trim() === "")) {
          continue;
        }
        const record = buildRecord(row, headers, name, idCounter++);
        if (record) {
          records.push(record);
        }
      }
    } catch (err) {
      errors.push(`解析文件 ${name} 失败：${err instanceof Error ? err.message : String(err)}`);
    }
  }

  return { records, errors };
}

export function exportRecordsToCsv(records: ParkingRecord[]): string {
  const headers = [
    "车牌号",
    "收费车型",
    "车辆类型",
    "车型",
    "车辆品牌",
    "车系",
    "车主电话",
    "车主姓名",
    "进场时间",
    "进场车道",
    "出场时间",
    "出场车道",
    "停车时长",
    "应收金额",
    "实收金额",
    "领券类型",
    "领券优惠",
    "充电优惠类型",
    "充电优惠金额",
    "缴费方式",
    "放行方式",
    "操作者",
    "备注",
    "来源文件",
  ];
  const escape = (v: string | number) => {
    const s = String(v ?? "").replace(/"/g, '""');
    return s.includes(",") || s.includes('"') || s.includes("\n") ? `"${s}"` : s;
  };
  const lines = [headers.join(",")];
  for (const r of records) {
    lines.push(
      [
        r.plate,
        r.chargeType,
        r.vehicleType,
        r.vehicleModel,
        r.vehicleBrand,
        r.vehicleSeries,
        r.ownerPhone,
        r.ownerName,
        r.entryTime,
        r.entryLane,
        r.exitTime,
        r.exitLane,
        r.duration,
        r.dueAmount,
        r.paidAmount,
        r.couponType,
        r.couponDiscount,
        r.chargeDiscountType,
        r.chargeDiscountAmount,
        r.paymentMethod,
        r.releaseMethod,
        r.operator,
        r.remark,
        r.sourceFile,
      ]
        .map(escape)
        .join(",")
    );
  }
  return "\ufeff" + lines.join("\n");
}
