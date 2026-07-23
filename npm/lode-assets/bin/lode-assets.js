#!/usr/bin/env node
const cp = require("node:child_process");

const args = ["assets", ...process.argv.slice(2)];
const result = cp.spawnSync("lode", args, { stdio: "inherit", shell: false });

if (result.error) {
  if (result.error.code === "ENOENT") {
    console.error("lode-assets requires the LODE CLI. Install lode, then run: lode assets " + process.argv.slice(2).join(" "));
    process.exit(1);
  }
  console.error(result.error.message);
  process.exit(1);
}

if (typeof result.status === "number") {
  process.exit(result.status);
}
process.exit(result.signal ? 1 : 0);