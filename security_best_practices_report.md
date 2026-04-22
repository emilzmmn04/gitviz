# Security Review Report

## Executive Summary

This repository does not appear to contain committed secrets, private keys, or obviously unsafe public-facing code paths. I did not find a reason to avoid sharing the GitHub repository publicly.

The main residual risks are supply-chain hardening issues around GitHub Actions and the npm installer path, not direct exploitable flaws in the Rust application itself.

## Scope

- Local repository review
- GitHub workflow and release automation review
- npm installer review
- Secret/material exposure scan

## Critical

No critical findings.

## High

No high-severity findings.

## Medium

### M-01: npm postinstall verifies a checksum fetched from the same mutable release channel

Affected file:
- [packaging/npm/scripts/postinstall.js](/Users/emilzimmermann/gitviz/packaging/npm/scripts/postinstall.js:92)

Relevant lines:
- [packaging/npm/scripts/postinstall.js](/Users/emilzimmermann/gitviz/packaging/npm/scripts/postinstall.js:92)
- [packaging/npm/scripts/postinstall.js](/Users/emilzimmermann/gitviz/packaging/npm/scripts/postinstall.js:105)
- [packaging/npm/scripts/postinstall.js](/Users/emilzimmermann/gitviz/packaging/npm/scripts/postinstall.js:108)

The npm installer downloads both the archive and its `.sha256` file from the same GitHub release and trusts that checksum for verification. This protects against transit corruption, but it does not provide independent authenticity if the release assets themselves are replaced or the release channel is compromised.

Impact:
- A compromised release publishing path could swap both the binary and checksum and still pass local verification.

Suggested remediation:
- Prefer shipping platform-specific packages with embedded hashes, or verify a detached signature/public signing key that is not fetched from the same mutable asset set.
- If you keep this model, document it clearly so users understand they are trusting GitHub release integrity, not an out-of-band signature chain.

## Low

### L-01: GitHub Actions are pinned to version tags, not immutable commit SHAs

Affected files:
- [ci.yml](/Users/emilzimmermann/gitviz/.github/workflows/ci.yml:14)
- [package-smoke.yml](/Users/emilzimmermann/gitviz/.github/workflows/package-smoke.yml:14)
- [prelaunch-checklist.yml](/Users/emilzimmermann/gitviz/.github/workflows/prelaunch-checklist.yml:21)
- [release.yml](/Users/emilzimmermann/gitviz/.github/workflows/release.yml:34)

Representative examples:
- [ci.yml](/Users/emilzimmermann/gitviz/.github/workflows/ci.yml:14)
- [ci.yml](/Users/emilzimmermann/gitviz/.github/workflows/ci.yml:17)
- [release.yml](/Users/emilzimmermann/gitviz/.github/workflows/release.yml:34)
- [release.yml](/Users/emilzimmermann/gitviz/.github/workflows/release.yml:112)

The workflows use references such as `actions/checkout@v4`, `actions/setup-node@v4`, and `softprops/action-gh-release@v2`. This is common, but it leaves you exposed to an upstream tag-retarget or compromised action release.

Impact:
- Compromise of an upstream GitHub Action tag could affect CI or release automation.

Suggested remediation:
- Pin third-party actions to full commit SHAs and optionally annotate each line with the human-friendly version comment.

### L-02: Release workflow grants `contents: write` to the full workflow instead of only write-needing jobs

Affected file:
- [release.yml](/Users/emilzimmermann/gitviz/.github/workflows/release.yml:8)

Relevant lines:
- [release.yml](/Users/emilzimmermann/gitviz/.github/workflows/release.yml:8)
- [release.yml](/Users/emilzimmermann/gitviz/.github/workflows/release.yml:96)
- [release.yml](/Users/emilzimmermann/gitviz/.github/workflows/release.yml:160)

The release workflow sets `permissions: contents: write` at the workflow level. Only the jobs that create a GitHub release or push to `gh-pages` actually need elevated repository write permissions.

Impact:
- If a build step in an earlier job were ever compromised, it would inherit broader repository write access than necessary.

Suggested remediation:
- Default the workflow to read-only permissions and set `contents: write` only on `create-release` and other jobs that actually need it.

## Informational

### I-01: No committed secrets were found in the repository scan

Evidence:
- No `.env`, key, certificate, or common token material was found in the working tree.
- The repository ignore rules exclude generated artifacts and local npm download directories: [.gitignore](/Users/emilzimmermann/gitviz/.gitignore:1)

### I-02: The Rust app uses subprocess arguments directly instead of shell interpolation

Affected file:
- [src/git/commands.rs](/Users/emilzimmermann/gitviz/src/git/commands.rs:5)

The Git commands are executed with `std::process::Command` and argument arrays, not by constructing shell command strings. That materially reduces command injection risk for repository path and revision arguments.

## Recommendation

It is reasonable to share the GitHub repository publicly and use the repo as the main destination instead of maintaining a separate landing page.

Before posting:

1. Add 2-4 strong screenshots or one short terminal GIF to the README and your X post.
2. If you want better hardening, fix M-01 first, then L-01 and L-02.
3. If you do not want to maintain a public website, remove the landing-page mention from the README so the repo tells one clear story.
