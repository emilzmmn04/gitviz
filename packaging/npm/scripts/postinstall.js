#!/usr/bin/env node

const crypto = require("node:crypto");
const fs = require("node:fs");
const https = require("node:https");
const path = require("node:path");
const { execFileSync } = require("node:child_process");

const PACKAGE_ROOT = path.resolve(__dirname, "..");
const VENDOR_DIR = path.join(PACKAGE_ROOT, "vendor");
const TMP_DIR = path.join(PACKAGE_ROOT, ".tmp");
const BINARY_NAME = process.platform === "win32" ? "gitviz.exe" : "gitviz";
const BINARY_PATH = path.join(VENDOR_DIR, BINARY_NAME);

const TARGETS = {
  "darwin-x64": "x86_64-apple-darwin",
  "darwin-arm64": "aarch64-apple-darwin",
  "linux-x64": "x86_64-unknown-linux-gnu",
  "linux-arm64": "aarch64-unknown-linux-gnu"
};

function loadPackageMetadata() {
  const packageJsonPath = path.join(PACKAGE_ROOT, "package.json");
  return JSON.parse(fs.readFileSync(packageJsonPath, "utf8"));
}

function request(url, redirectCount = 0) {
  return new Promise((resolve, reject) => {
    const req = https.get(
      url,
      {
        headers: {
          "User-Agent": "@emilzmmn04/gitviz postinstall"
        }
      },
      (res) => {
        const status = res.statusCode || 0;

        if (status >= 300 && status < 400 && res.headers.location) {
          if (redirectCount >= 5) {
            reject(new Error(`Too many redirects while requesting ${url}`));
            return;
          }
          resolve(request(res.headers.location, redirectCount + 1));
          return;
        }

        if (status < 200 || status >= 300) {
          reject(new Error(`Request failed (${status}) for ${url}`));
          return;
        }

        const chunks = [];
        res.on("data", (chunk) => chunks.push(chunk));
        res.on("end", () => resolve(Buffer.concat(chunks)));
      }
    );

    req.on("error", reject);
  });
}

function sha256(filePath) {
  const digest = crypto.createHash("sha256");
  digest.update(fs.readFileSync(filePath));
  return digest.digest("hex");
}

function parseChecksumLine(text) {
  const line = text.trim().split(/\r?\n/)[0] || "";
  const parts = line.trim().split(/\s+/);
  if (parts.length === 0 || !parts[0]) {
    throw new Error("Checksum file is empty");
  }
  return parts[0];
}

async function main() {
  if (process.env.GITVIZ_SKIP_POSTINSTALL === "1") {
    process.stdout.write("Skipping gitviz binary download (GITVIZ_SKIP_POSTINSTALL=1).\n");
    return;
  }

  const platformKey = `${process.platform}-${process.arch}`;
  const target = TARGETS[platformKey];
  if (!target) {
    throw new Error(`Unsupported platform/architecture: ${platformKey}`);
  }

  const pkg = loadPackageMetadata();
  const version = pkg.version;
  const releaseBase = `https://github.com/emilzmmn04/gitviz/releases/download/v${version}`;

  const archiveName = `gitviz-v${version}-${target}.tar.gz`;
  const checksumName = `gitviz-v${version}-${target}.sha256`;
  const archiveUrl = `${releaseBase}/${archiveName}`;
  const checksumUrl = `${releaseBase}/${checksumName}`;

  fs.mkdirSync(TMP_DIR, { recursive: true });
  fs.mkdirSync(VENDOR_DIR, { recursive: true });

  const archivePath = path.join(TMP_DIR, archiveName);

  const archive = await request(archiveUrl);
  fs.writeFileSync(archivePath, archive);

  const checksumContents = (await request(checksumUrl)).toString("utf8");
  const expectedHash = parseChecksumLine(checksumContents);
  const actualHash = sha256(archivePath);

  if (actualHash !== expectedHash) {
    throw new Error(`Checksum mismatch for ${archiveName}`);
  }

  execFileSync("tar", ["-xzf", archivePath, "-C", VENDOR_DIR], { stdio: "inherit" });

  if (!fs.existsSync(BINARY_PATH)) {
    throw new Error("Downloaded archive did not contain gitviz binary");
  }

  fs.chmodSync(BINARY_PATH, 0o755);
  fs.rmSync(TMP_DIR, { recursive: true, force: true });

  process.stdout.write(`Installed gitviz ${version} for ${platformKey}.\n`);
}

main().catch((error) => {
  console.error(`gitviz postinstall failed: ${error.message}`);
  process.exit(1);
});
