# gitviz Release Checklist

1. Ensure `.github/workflows/ci.yml` is green.
2. Ensure `.github/workflows/package-smoke.yml` is green.
3. Ensure `.github/workflows/release-dry-run.yml` succeeds for the target version.
4. Update the version in root `Cargo.toml`.
5. Update the version in `packaging/npm/package.json`.
6. Update landing page copy if feature or install messaging changed.
7. Run `cargo test`.
8. Run `cargo clippy --all-targets --all-features -- -D warnings`.
9. Create the release tag as `vX.Y.Z`.
10. Verify GitHub release assets are present.
11. Verify npm install succeeds.
12. Verify Homebrew install succeeds.
13. Verify APT package availability.
