#!/usr/bin/env node

const { spawnSync } = require("node:child_process");
const fs = require("node:fs");
const path = require("node:path");

const binaryName = process.platform === "win32" ? "gitviz.exe" : "gitviz";
const binaryPath = path.resolve(__dirname, "..", "vendor", binaryName);

if (!fs.existsSync(binaryPath)) {
  console.error("gitviz binary is missing. Reinstall the package: npm i -g @emilzmmn04/gitviz");
  process.exit(1);
}

const child = spawnSync(binaryPath, process.argv.slice(2), {
  stdio: "inherit"
});

if (child.error) {
  console.error(`Failed to execute gitviz: ${child.error.message}`);
  process.exit(1);
}

if (child.signal) {
  process.kill(process.pid, child.signal);
}

process.exit(child.status ?? 1);
