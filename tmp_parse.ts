import { parseMonthlyRentalZipFile, parseFreeCarZipFile } from './src/utils/parser';
import fs from 'fs';

async function main() {
  const monthlyBuf = fs.readFileSync('E:/shenyewuye/project/月租车列表2026-09-01_09_23_48.zip');
  const monthly = await parseMonthlyRentalZipFile(new Uint8Array(monthlyBuf));
  console.log('月租车 total', monthly.records.length);
  const building1 = monthly.records.filter(r => r.building === '1');
  console.log('building=1 count', building1.length);
  console.log('sample building=1', building1.slice(0, 5));
  const freeBuf = fs.readFileSync('E:/shenyewuye/project/免费车列表2026-08-28_11_25_04.zip');
  const free = await parseFreeCarZipFile(new Uint8Array(freeBuf));
  console.log('免费车 total', free.records.length);
  const f1 = free.records.filter(r => r.building === '1');
  console.log('free building=1 count', f1.length);
  console.log('sample free building=1', f1.slice(0, 5));
}
main().catch(e => { console.error(e); process.exit(1); });
