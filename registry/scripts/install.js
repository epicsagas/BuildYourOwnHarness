#!/usr/bin/env node
// BYOH plugin bootstrap — runs on SessionStart via .claude-plugin/hooks.json.
// Installs the `byoh` binary if missing, cross-platform. Always non-fatal: a
// failure must never break the Claude Code session (it just logs + exits 0).
//
// Uses only Node.js built-ins — no npm install. Claude Code is built on Node,
// so `node` is always on the PATH it uses to launch hook commands.
//
// BYOH is GitHub-Release-only (not on crates.io, no Homebrew tap yet), so this
// downloads the hand-written install.sh / install.ps1 from the latest release.

"use strict";

const { spawnSync } = require("child_process");
const { createWriteStream, chmodSync, readFileSync } = require("fs");
const { join } = require("path");
const https = require("https");
const os = require("os");

const REPO = "epicsagas/BuildYourOwnHarness";
const BINARY = "byoh";
const INSTALLER_SH = `https://github.com/${REPO}/releases/latest/download/install.sh`;
const INSTALLER_PS1 = `https://github.com/${REPO}/releases/latest/download/install.ps1`;

function log(msg) {
  process.stderr.write(`[byoh plugin] ${msg}\n`);
}

/** True if `byoh` is installed and runnable. */
function hasBinary() {
  const r = spawnSync(BINARY, ["--help"], { stdio: "pipe", shell: false });
  return r.status === 0;
}

/** Semver string from `byoh --version`, or null. */
function getBinaryVersion() {
  try {
    const r = spawnSync(BINARY, ["--version"], { stdio: "pipe", shell: false });
    if (r.status === 0 || (r.stdout && r.stdout.length)) {
      const out = (r.stdout ? r.stdout.toString() : "") + (r.stderr ? r.stderr.toString() : "");
      const m = out.match(/(\d+\.\d+\.\d+)/);
      return m ? m[1] : null;
    }
  } catch (_) {}
  return null;
}

/** Plugin manifest version (source of truth for "is the binary behind?"). */
function getPluginVersion() {
  try {
    const p = join(process.env.CLAUDE_PLUGIN_ROOT || "", ".claude-plugin", "plugin.json");
    return JSON.parse(readFileSync(p, "utf8")).version || null;
  } catch (_) {}
  return null;
}

function semverGt(a, b) {
  const pa = a.split(".").map(Number);
  const pb = b.split(".").map(Number);
  for (let i = 0; i < 3; i++) {
    if (pa[i] > pb[i]) return true;
    if (pa[i] < pb[i]) return false;
  }
  return false;
}

function downloadFile(url, dest) {
  return new Promise((resolve, reject) => {
    const file = createWriteStream(dest);
    const follow = (u) => {
      https
        .get(u, (res) => {
          if (res.statusCode === 301 || res.statusCode === 302) {
            follow(res.headers.location);
            res.resume();
            return;
          }
          if (res.statusCode !== 200) {
            reject(new Error(`HTTP ${res.statusCode} for ${u}`));
            return;
          }
          res.pipe(file);
          file.on("finish", () => file.close(resolve));
        })
        .on("error", reject);
    };
    follow(url);
  });
}

/** Download + run the platform installer (install.sh / install.ps1). */
async function install() {
  if (os.platform() === "win32") {
    const tmp = join(os.tmpdir(), "byoh-installer.ps1");
    log("Downloading Windows installer...");
    await downloadFile(INSTALLER_PS1, tmp);
    const r = spawnSync("powershell", ["-ExecutionPolicy", "Bypass", "-File", tmp], {
      stdio: "inherit",
    });
    if (r.status !== 0) throw new Error("PowerShell installer failed");
  } else {
    const tmp = join(os.tmpdir(), "byoh-installer.sh");
    log("Downloading installer...");
    await downloadFile(INSTALLER_SH, tmp);
    chmodSync(tmp, 0o755);
    const r = spawnSync("sh", [tmp], { stdio: "inherit" });
    if (r.status !== 0) throw new Error("Shell installer failed");
  }
}

async function main() {
  // 1. Binary missing — fresh install.
  if (!hasBinary()) {
    log(`${BINARY} not found — installing...`);
    try {
      await install();
    } catch (e) {
      log(`Install failed: ${e.message}`);
      log(`Install manually: https://github.com/${REPO}#installation`);
      process.exit(0); // non-fatal
    }
    return;
  }

  // 2. Binary present — upgrade if the plugin ships a newer version.
  const pluginVersion = getPluginVersion();
  if (pluginVersion) {
    const binaryVersion = getBinaryVersion();
    if (binaryVersion && semverGt(pluginVersion, binaryVersion)) {
      log(`Updating ${BINARY} ${binaryVersion} → ${pluginVersion}...`);
      try {
        await install();
        const now = getBinaryVersion();
        if (now) log(`Updated to ${now}`);
      } catch (e) {
        log(`Update failed: ${e.message} — continuing with ${binaryVersion}`);
        // non-fatal; old binary still works
      }
    }
  }
}

main().catch((e) => {
  log(`Unexpected error: ${e.message}`);
  process.exit(0); // non-fatal — never break the session
});
