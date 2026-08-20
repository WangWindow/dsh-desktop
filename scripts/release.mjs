#!/usr/bin/env node

import { existsSync } from "node:fs";
import { readFile, writeFile } from "node:fs/promises";
import { execFileSync } from "node:child_process";
import { resolve } from "node:path";

const args = new Set(process.argv.slice(2));
const allowedArgs = new Set(["--dry-run", "--push", "--help", "-h"]);

for (const arg of args) {
  if (!allowedArgs.has(arg)) {
    fail(`Unknown argument: ${arg}`);
  }
}

if (args.has("--help") || args.has("-h")) {
  console.log(`Usage:
  bun run release
  bun run release -- --dry-run
  bun run release -- --push

Reads the version from src-tauri/tauri.conf.json, updates CHANGELOG.md,
commits the changelog, and creates an annotated v<version> tag.

--dry-run  Print the planned changelog and git operations without changing files.
--push     Push the current branch and the new tag to origin after tagging.`);
  process.exit(0);
}

const dryRun = args.has("--dry-run");
const push = args.has("--push");
const root = git(["rev-parse", "--show-toplevel"]).trim();
const configPath = resolve(root, "src-tauri/tauri.conf.json");
const changelogPath = resolve(root, "CHANGELOG.md");

const config = JSON.parse(await readFile(configPath, "utf8"));
const version = String(config.version ?? "").trim();

if (!/^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$/.test(version)) {
  fail(`Invalid or missing version in ${configPath}`);
}

const tag = `v${version}`;
const branch = git(["branch", "--show-current"]).trim();

if (!branch) {
  fail("Release cannot run from a detached HEAD.");
}

if (!gitSucceeds(["rev-parse", "--verify", "HEAD"])) {
  fail("Release requires at least one existing git commit.");
}

if (gitSucceeds(["rev-parse", "--verify", `refs/tags/${tag}`])) {
  fail(`Tag ${tag} already exists.`);
}

const status = git(["status", "--porcelain"]);
if (status.trim() && !dryRun) {
  fail("Working tree must be clean before releasing. Commit or stash changes first.");
}

if (status.trim() && dryRun) {
  console.warn("Warning: working tree is not clean; dry-run only.");
}

const previousTag = git(["tag", "--list", "v*", "--sort=-version:refname"])
  .split("\n")
  .map((item) => item.trim())
  .find(Boolean);

const range = previousTag ? `${previousTag}..HEAD` : "HEAD";
const commits = git(["log", range, "--pretty=format:%h%x09%s"])
  .split("\n")
  .map((line) => line.trim())
  .filter(Boolean)
  .map(parseCommit);

const changelog = buildChangelog(
  existsSync(changelogPath) ? await readFile(changelogPath, "utf8") : "",
  version,
  commits,
);

console.log(`Release: ${tag}`);
console.log(`Previous tag: ${previousTag ?? "none"}`);
console.log(`Commits included: ${commits.length}`);
console.log("\n" + changelog);

if (dryRun) {
  console.log("Dry-run complete. No files, commits, tags, or pushes were changed.");
  process.exit(0);
}

await writeFile(changelogPath, changelog, "utf8");
git(["add", "CHANGELOG.md"]);
git(["commit", "-m", `chore(release): ${tag}`]);
git(["tag", "--annotate", tag, "--message", `Release ${tag}`]);

console.log(`Created commit and tag ${tag}.`);

if (push) {
  git(["push", "origin", branch, "--follow-tags"]);
  console.log(`Pushed ${branch} and ${tag} to origin.`);
} else {
  console.log(`Nothing was pushed. Run: git push origin ${branch} --follow-tags`);
}

function parseCommit(line) {
  const [hash, subject = ""] = line.split("\t", 2);
  const match = subject.match(/^([a-z]+)(?:\([^)]*\))?(!)?:\s*(.+)$/i);

  if (!match) {
    return { hash, type: "other", breaking: false, subject };
  }

  return {
    hash,
    type: match[1].toLowerCase(),
    breaking: Boolean(match[2]),
    subject: match[3],
  };
}

function buildChangelog(existing, releaseVersion, commits) {
  const sections = new Map([
    ["Breaking Changes", []],
    ["Added", []],
    ["Fixed", []],
    ["Changed", []],
    ["Maintenance", []],
  ]);

  for (const commit of commits) {
    const section = commit.breaking
      ? "Breaking Changes"
      : sectionForType(commit.type);
    sections.get(section).push(`- ${commit.subject} (${commit.hash})`);
  }

  const releaseLines = [`## [${releaseVersion}] - ${today()}`, ""];
  for (const [name, entries] of sections) {
    if (entries.length === 0) {
      continue;
    }

    releaseLines.push(`### ${name}`, "", ...entries, "");
  }

  if (commits.length === 0) {
    releaseLines.push("- No changes recorded.", "");
  }

  const current = existing.trimEnd();
  const lines = current ? current.split("\n") : ["# Changelog"];
  const hasHeader = lines[0].trim() === "# Changelog";
  const prefix = hasHeader ? lines.slice(0, 1) : ["# Changelog"];
  const suffix = hasHeader ? lines.slice(1).join("\n").trim() : current;

  return [
    ...prefix,
    "",
    ...releaseLines,
    ...(suffix ? [suffix] : []),
  ].join("\n").replace(/\n+$/, "") + "\n";
}

function sectionForType(type) {
  if (type === "feat") {
    return "Added";
  }

  if (type === "fix") {
    return "Fixed";
  }

  if (["refactor", "perf"].includes(type)) {
    return "Changed";
  }

  if (["docs", "build", "ci", "chore", "test"].includes(type)) {
    return "Maintenance";
  }

  return "Changed";
}

function today() {
  return new Date().toISOString().slice(0, 10);
}

function git(argumentsList) {
  try {
    return execFileSync("git", argumentsList, {
      cwd: process.cwd(),
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    });
  } catch (error) {
    const details = error.stderr?.toString().trim();
    fail(details || `git ${argumentsList.join(" ")} failed`);
  }
}

function gitSucceeds(argumentsList) {
  try {
    execFileSync("git", argumentsList, {
      cwd: process.cwd(),
      stdio: "ignore",
    });
    return true;
  } catch {
    return false;
  }
}

function fail(message) {
  console.error(`Release failed: ${message}`);
  process.exit(1);
}
