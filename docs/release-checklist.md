# gitviz Release Checklist

Bootstrap requirements live in [docs/publishing-bootstrap.md](publishing-bootstrap.md).

1. Ensure `.github/workflows/ci.yml` is green.
2. Ensure `.github/workflows/package-smoke.yml` is green.
3. Ensure `.github/workflows/release-dry-run.yml` succeeds for the target version.
4. Update the version in root `Cargo.toml`.
5. Update the version in `packaging/npm/package.json`.
6. Update README/install messaging if release channels or commands changed.
7. Run `cargo test`.
8. Run `cargo clippy --all-targets --all-features -- -D warnings`.
9. Create the release tag as `vX.Y.Z`.
10. Verify GitHub release assets are present.
11. Verify `cargo install gitviz --version X.Y.Z` succeeds.
12. Verify `npm i -g @emilzmmn04/gitviz` succeeds.
13. Verify Homebrew install succeeds.
14. Verify APT package availability.
