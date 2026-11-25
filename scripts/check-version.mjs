// Keeps the three manifests that carry a version number in lockstep so release
// artifacts are named `BiuPlayer_<version>_<os>_<arch>` from one source (REL-06).
import { readFileSync } from "node:fs";

function readJson(path) {
  return JSON.parse(readFileSync(path, "utf8"));
}

const pkg = readJson("package.json");
const tauriConf = readJson("src-tauri/tauri.conf.json");
const cargoToml = readFileSync("src-tauri/Cargo.toml", "utf8");
const cargoVersion = cargoToml.match(/^version\s*=\s*"([^"]+)"/m)?.[1];

const versions = {
  "package.json": pkg.version,
  "src-tauri/tauri.conf.json": tauriConf.version,
  "src-tauri/Cargo.toml": cargoVersion,
};

const expected = pkg.version;
let failed = false;
for (const [file, version] of Object.entries(versions)) {
  if (!version) {
    console.error(`missing version in ${file}`);
    failed = true;
  } else if (version !== expected) {
    console.error(`${file} has ${version}, package.json has ${expected}`);
    failed = true;
  }
}

if (failed) {
  process.exit(1);
}
console.log(`version ${expected} consistent across package.json, tauri.conf.json, Cargo.toml`);
