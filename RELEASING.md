# Releasing

`gpui-router` and `gpui-router-macros` are published from this workspace.
Publish `gpui-router-macros` first whenever it changed: `gpui-router` resolves
that dependency from crates.io while `cargo package` runs, so the macros version
has to exist before the router can even be packaged.

## Checklist

1. `main` is green (`build` and `gpui-kit` workflows), the working tree is clean and `CHANGELOG.md` has an `Unreleased` entry for the change.
2. Decide the version — features and swallowed panics get a minor bump, fixes a patch bump.
3. Bump `crates/router/Cargo.toml` (and `crates/router-macros/Cargo.toml` when the macros changed), plus the matching `[workspace.dependencies]` versions, then run `cargo check --locked` so `Cargo.lock` records the new versions.
4. Move the changelog entries from `Unreleased` into the new version and commit: `chore: release X.Y.Z`.
5. Publish in order:

   ```sh
   cargo publish -p gpui-router-macros   # only when it changed
   cargo publish -p gpui-router
   ```

   The `package` job in `.github/workflows/build.yml` already runs `cargo package` for both crates, so manifest problems surface before this step.

6. Tag and create the GitHub release:

   ```sh
   git tag vX.Y.Z && git push origin vX.Y.Z
   gh release create vX.Y.Z --title "vX.Y.Z" --notes-file notes.md
   ```

7. Verify the published pair from outside the workspace — a scratch crate that depends on the released version, in both backends when relevant:

   ```sh
   cargo new --lib /tmp/release-check && cd /tmp/release-check
   cargo add gpui-router@X.Y.Z
   cargo test
   ```

## Notes

- `crates/gpui-kit-compat` is `publish = false`; it exists so the `gpui-pre` backend and the `IntoLayout` derive stay covered by CI.
- The publish step in `.github/workflows/build.yml` is commented out on purpose: releases are cut by hand, so the checklist above is the process.
- `gpui-kit` pins `gpui-pre` with an exact version while `gpui-router` requires it with a caret. Both have to resolve to the same 0.3.x release; a new `gpui-pre` minor line needs a follow-up `gpui-router` release.
- `rust-version = "1.88"` is the highest `rust-version` declared in the resolved dependency tree of `gpui-router`; the `msrv` job in `.github/workflows/build.yml` checks it.
- The matcher cache can be measured on demand: `cargo test --release --features runtime_shaders -- --ignored --nocapture`.
