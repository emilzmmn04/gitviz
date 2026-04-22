# Publishing Bootstrap

This repository can build and publish release artifacts automatically, but a few external accounts, tokens, and repositories must exist first.

## Required For Core Release UX

### GitHub Releases

Already handled by this repository once you push a `vX.Y.Z` tag.

### crates.io

You still need to:

1. Create or log into your crates.io account.
2. Create a `CARGO_REGISTRY_TOKEN`.
3. Add `CARGO_REGISTRY_TOKEN` as a repository secret.
4. Confirm that the crate name `gitviz` is available, or rename the crate before the first publish.

### npm

You still need to:

1. Create or log into the npm account that owns the `@emilzmmn04` scope.
2. Create an automation token with publish access.
3. Add `NPM_TOKEN` as a repository secret.
4. Confirm that the scope/package name `@emilzmmn04/gitviz` is the name you want to keep.

### Homebrew

You still need to:

1. Create the tap repository `emilzmmn04/homebrew-tap`.
2. Ensure the GitHub token used for tap updates can push to that repository.
3. Add `HOMEBREW_TAP_GITHUB_TOKEN` as a repository secret.

## Optional

### APT Repository

You still need to:

1. Generate or choose the GPG key used to sign APT metadata.
2. Add `APT_GPG_PRIVATE_KEY` as a repository secret.
3. Optionally add `APT_GPG_PASSPHRASE` if the key is encrypted.

## Suggested First Release Order

1. GitHub Releases
2. crates.io
3. npm
4. Homebrew
5. APT
