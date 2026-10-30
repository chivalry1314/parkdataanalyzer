# 停车收费记录解析工具

基于 Tauri + Vue 3 + TypeScript + TailwindCSS 的桌面应用，可解析 ZIP 压缩包中的 Excel 进出场记录表，支持记录查看与统计分析。

## 功能

- 选择 ZIP 压缩文件（内含一个或多个 `.xlsx` 进出场记录表）
- 自动识别表头并解析所有停车收费记录
- 记录列表：支持搜索、分页、导出 CSV
- 统计分析：
  - 总记录数、应收/实收/未收金额
  - 车辆类型、收费车型、放行方式、缴费方式分布
  - 月度/每日收费趋势

## 技术栈

- 前端：Vue 3、Vite、TypeScript、TailwindCSS
- 桌面壳：Tauri v2
- 解析：JSZip、xlsx
- 持久化：SQLite（rusqlite）

## 数据持久化

解析后的记录会保存到本地 SQLite 数据库（位于应用数据目录下的 `parkingsyslog.db`），关闭应用后数据不会丢失。支持：

- 重新打开应用时自动加载历史记录
- 选择新 ZIP 文件时覆盖已有数据
- 一键清空数据库

## 开发

```bash
npm install
npm run tauri dev
```

## 构建

```bash
npm run tauri build
# 仅构建 MSI
npm run tauri build -- --bundles msi
```

构建完成后：

- 可执行文件：`src-tauri/target/release/parkingsyslog.exe`
- MSI 安装包：`src-tauri/target/release/bundle/msi/ParkingSysLog_0.1.0_x64_en-US.msi`

## 数据来源

示例压缩包：`E:/shenyewuye/project/进出场记录表.zip`

解析字段包括：车牌号、收费车型、车辆类型、车型、车辆品牌、车系、车主电话、车主姓名、进场时间、进场车道、出场时间、出场车道、停车时长、应收金额、实收金额、领券类型、领券优惠、充电优惠类型、充电优惠金额、缴费方式、放行方式、操作者、备注。
