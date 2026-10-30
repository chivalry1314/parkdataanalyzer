const fs = require('fs');
const path = require('path');

const tag = process.argv[2] || process.env.GITHUB_REF_NAME || '';
const version = tag.replace(/^v/, '').trim();

if (!version || !/^\d+\.\d+\.\d+/.test(version)) {
  console.error(`Invalid version from tag: ${tag}`);
  process.exit(1);
}

const root = path.resolve(__dirname, '../..');
const tauriPath = path.join(root, 'src-tauri', 'tauri.conf.json');
const cargoPath = path.join(root, 'src-tauri', 'Cargo.toml');
const packagePath = path.join(root, 'package.json');

// Update tauri.conf.json
const tauri = JSON.parse(fs.readFileSync(tauriPath, 'utf8'));
tauri.version = version;
fs.writeFileSync(tauriPath, JSON.stringify(tauri, null, 2) + '\n');
console.log(`Updated ${tauriPath}: version = ${version}`);

// Update Cargo.toml
let cargo = fs.readFileSync(cargoPath, 'utf8');
cargo = cargo.replace(/^version\s*=\s*"[^"]+"/m, `version = "${version}"`);
fs.writeFileSync(cargoPath, cargo);
console.log(`Updated ${cargoPath}: version = ${version}`);

// Update package.json
const pkg = JSON.parse(fs.readFileSync(packagePath, 'utf8'));
pkg.version = version;
fs.writeFileSync(packagePath, JSON.stringify(pkg, null, 2) + '\n');
console.log(`Updated ${packagePath}: version = ${version}`);
