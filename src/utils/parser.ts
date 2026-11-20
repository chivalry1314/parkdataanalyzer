import * as XLSX from "xlsx";
import JSZip from "jszip";
import type {
  FreeCar,
  FreeCarParseResult,
  MonthlyCardLedger,
  MonthlyCardLedgerCheckIssue,
  MonthlyCardLedgerCheckResult,
  MonthlyCardLedgerParseResult,
  MonthlyRentalParseResult,
  MonthlyRentalVehicle,
  ParseResult,
  ParkingRecord,
} from "../types";

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

const MONTHLY_HEADER_KEY = "车牌号";

const MONTHLY_EXPECTED_HEADERS = [
  "车牌号",
  "车位",
  "车辆类型",
  "车主电话",
  "车主姓名",
  "车主地址",
  "开始有效时间",
  "结束有效时间",
  "车位组",
  "车位数量",
];

const FREE_CAR_HEADER_KEY = "车牌号";

const FREE_CAR_EXPECTED_HEADERS = [
  "车牌号",
  "车位",
  "车辆类型",
  "车主电话",
  "车主姓名",
  "车主地址",
  "开始有效时间",
  "结束有效时间",
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

function normalizeMonthlyHeader(value: unknown): string {
  const s = String(value || "").trim();
  return s.replace(/^\*+/, "");
}

function findMonthlyHeaderRow(rows: unknown[][]): { rowIndex: number; headers: string[] } | null {
  for (let i = 0; i < rows.length; i++) {
    const row = rows[i];
    if (!Array.isArray(row)) continue;
    const normalized = row.map(normalizeMonthlyHeader);
    if (normalized.includes(MONTHLY_HEADER_KEY)) {
      return { rowIndex: i, headers: normalized };
    }
  }
  return null;
}

function parseParkingSpaceCount(value: unknown): number {
  if (value === null || value === undefined || value === "") return 0;
  const n = Number(String(value).trim());
  return Number.isNaN(n) ? 0 : n;
}

function parseBuildingUnitRoom(address: string): { building: string; unit: string; room: string } {
  const parts = address.split("-");
  if (parts.length >= 3) {
    return { building: parts[0], unit: parts[1], room: parts[2] };
  }
  if (parts.length === 2) {
    return { building: parts[0], unit: parts[1], room: "" };
  }
  if (parts.length === 1 && parts[0]) {
    return { building: parts[0], unit: "", room: "" };
  }
  return { building: "", unit: "", room: "" };
}

function buildMonthlyRecord(
  row: unknown[],
  headers: string[],
  sourceFile: string,
  id: number
): MonthlyRentalVehicle | null {
  const get = (name: string): string => {
    const idx = headers.indexOf(name);
    if (idx === -1) return "";
    const v = row[idx];
    if (v === null || v === undefined) return "";
    return String(v).trim();
  };

  const plate = get("车牌号");
  if (!plate) return null;

  const ownerAddress = get("车主地址");
  const { building, unit, room } = parseBuildingUnitRoom(ownerAddress);

  return {
    id,
    plate,
    parkingSpace: get("车位"),
    vehicleType: get("车辆类型"),
    ownerPhone: get("车主电话"),
    ownerName: get("车主姓名"),
    ownerAddress,
    startDate: get("开始有效时间"),
    endDate: get("结束有效时间"),
    parkingGroup: get("车位组"),
    parkingSpaceCount: parseParkingSpaceCount(row[headers.indexOf("车位数量")] ?? 0),
    sourceFile,
    building,
    unit,
    room,
  };
}

function findFreeCarHeaderRow(rows: unknown[][]): { rowIndex: number; headers: string[] } | null {
  for (let i = 0; i < rows.length; i++) {
    const row = rows[i];
    if (!Array.isArray(row)) continue;
    const normalized = row.map(normalizeMonthlyHeader);
    if (normalized.includes(FREE_CAR_HEADER_KEY)) {
      return { rowIndex: i, headers: normalized };
    }
  }
  return null;
}

function buildFreeCarRecord(
  row: unknown[],
  headers: string[],
  sourceFile: string,
  id: number
): FreeCar | null {
  const get = (name: string): string => {
    const idx = headers.indexOf(name);
    if (idx === -1) return "";
    const v = row[idx];
    if (v === null || v === undefined) return "";
    return String(v).trim();
  };

  const plate = get("车牌号");
  if (!plate) return null;

  const ownerAddress = get("车主地址");
  const { building, unit, room } = parseBuildingUnitRoom(ownerAddress);

  return {
    id,
    plate,
    parkingSpace: get("车位"),
    vehicleType: get("车辆类型"),
    ownerPhone: get("车主电话"),
    ownerName: get("车主姓名"),
    ownerAddress,
    startDate: get("开始有效时间"),
    endDate: get("结束有效时间"),
    parkingGroup: "",
    parkingSpaceCount: 0,
    sourceFile,
    building,
    unit,
    room,
  };
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

export async function parseFreeCarZipFile(
  data: Uint8Array,
  onProgress?: (current: number, total: number, fileName: string) => void
): Promise<FreeCarParseResult> {
  const zip = await JSZip.loadAsync(data);
  const records: FreeCar[] = [];
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

      const headerInfo = findFreeCarHeaderRow(rows);
      if (!headerInfo) {
        errors.push(`文件 ${name} 中未找到表头，已跳过`);
        continue;
      }

      const { rowIndex, headers } = headerInfo;
      const missing = FREE_CAR_EXPECTED_HEADERS.filter((h) => !headers.includes(h));
      if (missing.length > 0) {
        errors.push(`文件 ${name} 缺少字段：${missing.join("、")}`);
      }

      for (let i = rowIndex + 1; i < rows.length; i++) {
        const row = rows[i];
        if (!Array.isArray(row) || row.every((c) => c === null || c === undefined || String(c).trim() === "")) {
          continue;
        }
        const record = buildFreeCarRecord(row, headers, name, idCounter++);
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

export async function parseMonthlyRentalZipFile(
  data: Uint8Array,
  onProgress?: (current: number, total: number, fileName: string) => void
): Promise<MonthlyRentalParseResult> {
  const zip = await JSZip.loadAsync(data);
  const records: MonthlyRentalVehicle[] = [];
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

      const headerInfo = findMonthlyHeaderRow(rows);
      if (!headerInfo) {
        errors.push(`文件 ${name} 中未找到表头，已跳过`);
        continue;
      }

      const { rowIndex, headers } = headerInfo;
      const missing = MONTHLY_EXPECTED_HEADERS.filter((h) => !headers.includes(h));
      if (missing.length > 0) {
        errors.push(`文件 ${name} 缺少字段：${missing.join("、")}`);
      }

      for (let i = rowIndex + 1; i < rows.length; i++) {
        const row = rows[i];
        if (!Array.isArray(row) || row.every((c) => c === null || c === undefined || String(c).trim() === "")) {
          continue;
        }
        const record = buildMonthlyRecord(row, headers, name, idCounter++);
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

function parseBoolCell(value: unknown): boolean {
  if (value === null || value === undefined) return false;
  const s = String(value).trim();
  return s === "是" || s === "1" || s.toLowerCase() === "true" || s === "yes";
}

function parseIntCell(value: unknown): number {
  if (value === null || value === undefined || value === "") return 0;
  const n = Number(String(value).replace(/,/g, "").trim());
  return Number.isNaN(n) ? 0 : Math.round(n);
}

function parseStringCell(value: unknown): string {
  if (value === null || value === undefined) return "";
  return String(value).replace(/\n/g, " ").trim();
}

function formatDate(date: Date): string {
  const y = date.getFullYear();
  const m = String(date.getMonth() + 1).padStart(2, "0");
  const d = String(date.getDate()).padStart(2, "0");
  return `${y}-${m}-${d}`;
}

function excelSerialToDate(serial: number): Date | null {
  if (serial <= 0 || serial > 50000) return null;
  const epoch = new Date(1899, 11, 30);
  return new Date(epoch.getTime() + serial * 24 * 60 * 60 * 1000);
}

function normalizeDate(value: unknown): string {
  if (value === null || value === undefined || value === "") return "";

  if (typeof value === "number") {
    const date = excelSerialToDate(value);
    return date ? formatDate(date) : String(value);
  }

  const s = String(value).trim();
  if (!s) return "";

  // Excel serial number exported as string, e.g. "45300"
  if (/^\d+(\.\d+)?$/.test(s)) {
    const date = excelSerialToDate(Number(s));
    if (date) return formatDate(date);
  }

  // Common formats: 2024-01-15, 2024/01/15, 2024.01.15, 2024年01月15日, 2024-1-5
  const m = s.match(/^(\d{4})[-/.年](\d{1,2})[-/.月](\d{1,2})[日]?$/);
  if (m) {
    const y = Number(m[1]);
    const mon = Number(m[2]);
    const d = Number(m[3]);
    const date = new Date(y, mon - 1, d);
    if (
      date.getFullYear() === y &&
      date.getMonth() === mon - 1 &&
      date.getDate() === d
    ) {
      return formatDate(date);
    }
  }

  // Compact format: 20240115
  const compact = s.match(/^(\d{4})(\d{2})(\d{2})$/);
  if (compact) {
    const y = Number(compact[1]);
    const mon = Number(compact[2]);
    const d = Number(compact[3]);
    const date = new Date(y, mon - 1, d);
    if (
      date.getFullYear() === y &&
      date.getMonth() === mon - 1 &&
      date.getDate() === d
    ) {
      return formatDate(date);
    }
  }

  // Fallback to native parser for ISO strings etc.
  const date = new Date(s);
  if (!Number.isNaN(date.getTime())) {
    return formatDate(date);
  }

  return s;
}

function buildMonthlyCardLedgerRecord(
  row: unknown[],
  sourceFile: string,
  id: number
): MonthlyCardLedger | null {
  if (!Array.isArray(row) || row.every((c) => c === null || c === undefined || String(c).trim() === "")) {
    return null;
  }

  const plate = parseStringCell(row[5]);
  if (!plate) return null;

  return {
    id,
    seq: parseIntCell(row[0]),
    processDate: normalizeDate(row[1]),
    roomNumber: parseStringCell(row[2]),
    ownerName: parseStringCell(row[3]),
    ownerPhone: parseStringCell(row[4]),
    plate,
    paymentPeriod: parseStringCell(row[6]),
    paymentMonths: parseIntCell(row[7]),
    rent: parseMoney(row[8]),
    managementFee: parseMoney(row[9]),
    actualStartDate: parseStringCell(row[10]),
    actualEndDate: parseStringCell(row[11]),
    actualMonths: parseIntCell(row[12]),
    rentGifted: parseBoolCell(row[13]),
    propertyFeeOwed: parseBoolCell(row[14]),
    operator: parseStringCell(row[15]),
    remark: parseStringCell(row[16]),
    sourceFile,
  };
}

export function parseMonthlyCardLedgerXlsxFile(
  data: Uint8Array,
  sourceFile: string
): MonthlyCardLedgerParseResult {
  const records: MonthlyCardLedger[] = [];
  const errors: string[] = [];
  let idCounter = 1;

  try {
    const workbook = XLSX.read(data, { type: "array", cellText: true, cellDates: false });
    const sheetName = workbook.SheetNames[0];
    const worksheet = workbook.Sheets[sheetName];
    const rows = XLSX.utils.sheet_to_json(worksheet, { header: 1, raw: false }) as unknown[][];

    if (rows.length < 4) {
      errors.push("Excel 文件内容不足，至少需要包含表头和一行数据");
      return { records, errors };
    }

    for (let i = 3; i < rows.length; i++) {
      const record = buildMonthlyCardLedgerRecord(rows[i], sourceFile, idCounter++);
      if (record) {
        records.push(record);
      }
    }
  } catch (err) {
    errors.push(`解析文件失败：${err instanceof Error ? err.message : String(err)}`);
  }

  return { records, errors };
}

function isValidDateString(value: string): boolean {
  if (!value) return false;
  const normalized = normalizeDate(value);
  return /^\d{4}-\d{2}-\d{2}$/.test(normalized);
}

function isFutureDate(value: string): boolean {
  const normalized = normalizeDate(value);
  if (!/^\d{4}-\d{2}-\d{2}$/.test(normalized)) return false;
  const date = new Date(normalized);
  const today = new Date();
  today.setHours(0, 0, 0, 0);
  return date.getTime() > today.getTime();
}

export function checkMonthlyCardLedgerXlsxFile(
  data: Uint8Array,
  sourceFile: string,
  onProgress?: (current: number, total: number) => void
): Promise<MonthlyCardLedgerCheckResult> {
  return new Promise((resolve) => {
    const records: MonthlyCardLedger[] = [];
    const parseErrors: string[] = [];
    let idCounter = 1;

    try {
      const workbook = XLSX.read(data, { type: "array", cellText: true, cellDates: false });
      const sheetName = workbook.SheetNames[0];
      const worksheet = workbook.Sheets[sheetName];
      const rows = XLSX.utils.sheet_to_json(worksheet, { header: 1, raw: false }) as unknown[][];

      if (rows.length < 4) {
        parseErrors.push("Excel 文件内容不足，至少需要包含表头和一行数据");
        resolve({ total: 0, valid: 0, invalid: 0, issues: parseErrors.map((e) => ({ rowIndex: 0, plate: "", field: "文件解析", issue: e, value: "" })) });
        return;
      }

      for (let i = 3; i < rows.length; i++) {
        const record = buildMonthlyCardLedgerRecord(rows[i], sourceFile, idCounter++);
        if (record) {
          (record as unknown as { __rowIndex: number }).__rowIndex = i + 1;
          records.push(record);
        }
      }
    } catch (err) {
      parseErrors.push(`解析文件失败：${err instanceof Error ? err.message : String(err)}`);
    }

    if (parseErrors.length > 0) {
      resolve({
        total: 0,
        valid: 0,
        invalid: 0,
        issues: parseErrors.map((e) => ({ rowIndex: 0, plate: "", field: "文件解析", issue: e, value: "" })),
      });
      return;
    }

    const issues: MonthlyCardLedgerCheckIssue[] = [];
    const seenKeys = new Set<string>();
    const batchSize = 100;
    let index = 0;

    onProgress?.(0, records.length);

    function processBatch() {
      const end = Math.min(index + batchSize, records.length);
      for (let i = index; i < end; i++) {
        const record = records[i];
        const rowIndex = (record as unknown as { __rowIndex: number }).__rowIndex ?? record.seq;
        const recordIssues: MonthlyCardLedgerCheckIssue[] = [];

        if (!record.plate.trim()) {
          recordIssues.push({ rowIndex, plate: "", field: "车牌号", issue: "车牌号不能为空", value: "" });
        } else if (!/[\u4e00-\u9fa5][A-Z][A-Z0-9]{4,5}/.test(record.plate) && !/[A-Z][A-Z0-9]{4,5}/.test(record.plate)) {
          recordIssues.push({ rowIndex, plate: record.plate, field: "车牌号", issue: "车牌号格式不规范", value: record.plate });
        }

        if (!record.processDate) {
          recordIssues.push({ rowIndex, plate: record.plate, field: "办理日期", issue: "办理日期不能为空", value: record.processDate });
        } else if (!isValidDateString(record.processDate)) {
          recordIssues.push({ rowIndex, plate: record.plate, field: "办理日期", issue: "办理日期格式不正确", value: record.processDate });
        } else if (isFutureDate(record.processDate)) {
          recordIssues.push({ rowIndex, plate: record.plate, field: "办理日期", issue: "办理日期不能为未来日期", value: record.processDate });
        }

        if (!record.ownerName.trim()) {
          recordIssues.push({ rowIndex, plate: record.plate, field: "车主姓名", issue: "车主姓名不能为空", value: record.ownerName });
        }

        if (record.rent < 0) {
          recordIssues.push({ rowIndex, plate: record.plate, field: "租金", issue: "租金不能为负数", value: String(record.rent) });
        }

        if (record.managementFee < 0) {
          recordIssues.push({ rowIndex, plate: record.plate, field: "管理费", issue: "管理费不能为负数", value: String(record.managementFee) });
        }

        if (record.rent === 0 && record.managementFee === 0) {
          recordIssues.push({ rowIndex, plate: record.plate, field: "费用", issue: "租金和管理费不能同时为 0", value: `租金=${record.rent}, 管理费=${record.managementFee}` });
        }

        if (record.paymentMonths <= 0) {
          recordIssues.push({ rowIndex, plate: record.plate, field: "缴费月数", issue: "缴费月数应大于 0", value: String(record.paymentMonths) });
        }

        if (record.actualMonths < 0) {
          recordIssues.push({ rowIndex, plate: record.plate, field: "授权月数", issue: "授权月数不能为负数", value: String(record.actualMonths) });
        }

        if (record.actualStartDate || record.actualEndDate) {
          if (!isValidDateString(record.actualStartDate) || !isValidDateString(record.actualEndDate)) {
            recordIssues.push({ rowIndex, plate: record.plate, field: "授权时间", issue: "授权开始/结束日期格式不正确", value: `${record.actualStartDate} ~ ${record.actualEndDate}` });
          } else if (new Date(record.actualStartDate) > new Date(record.actualEndDate)) {
            recordIssues.push({ rowIndex, plate: record.plate, field: "授权时间", issue: "授权开始日期不能晚于结束日期", value: `${record.actualStartDate} ~ ${record.actualEndDate}` });
          }
        }

        if (record.rentGifted && record.rent !== 0) {
          recordIssues.push({ rowIndex, plate: record.plate, field: "租金赠送", issue: "勾选租金赠送时租金应为 0", value: String(record.rent) });
        }

        const duplicateKey = `${record.plate}|${record.processDate}|${record.paymentPeriod}`;
        if (duplicateKey.replace(/\|/g, "")) {
          if (seenKeys.has(duplicateKey)) {
            recordIssues.push({ rowIndex, plate: record.plate, field: "重复记录", issue: "存在重复记录（同一车牌、办理日期、充费时间段）", value: duplicateKey });
          } else {
            seenKeys.add(duplicateKey);
          }
        }

        issues.push(...recordIssues);
      }
      index = end;
      onProgress?.(index, records.length);
      if (index < records.length) {
        setTimeout(processBatch, 0);
      } else {
        const invalid = new Set(issues.map((i) => i.rowIndex)).size;
        resolve({
          total: records.length,
          valid: records.length - invalid,
          invalid,
          issues,
        });
      }
    }

    processBatch();
  });
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
