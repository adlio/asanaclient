# Publishing to crates.io

## Automated Release

Pushing a version tag triggers the release workflow (`.github/workflows/release.yml`), which creates a GitHub release and publishes to crates.io.

1. Update `CHANGELOG.md` with the new version's changes under a `## [x.y.z]` heading
2. Bump the version in `Cargo.toml`
3. Commit: `chore: bump version to x.y.z`
4. Tag and push: `git tag vx.y.z && git push origin main --tags`

## Manual Release

To publish manually without the workflow:

```bash
cargo publish
```

## crates.io Token

The workflow uses a `CARGO_REGISTRY_TOKEN` repository secret. The token is scoped to **publish-update** for the **asanaclient** crate only.

To rotate the token:

1. Revoke the old token at https://crates.io/settings/tokens
2. Create a new token with scope **publish-update** and crate **asanaclient**
3. Update the repository secret: `gh secret set CARGO_REGISTRY_TOKEN`
