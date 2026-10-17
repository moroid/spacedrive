# Workspace fork

This fork adds the Windows MDI and task workspace described in the source
project's `docs/08_Spacedriveフォーク移行計画案.md`. The first gate is an unchanged
Windows build, followed by two independent panes. The application has not
passed those gates yet.

## Provenance

- Upstream: https://github.com/spacedriveapp/spacedrive
- Baseline: `4d41a379f612e23ac43a66ffd67df52ca77f57d4`
- Fork: https://github.com/moroid/spacedrive
- Development branch: `workspace-mdi`
- Workspace source: local Workspace Filer commit `f19dbc9`
- User-provided archive SHA256:
  `26FE088FC9A0F7A722C2246CA6B10BEEF825E7F9EC41504F2A6E9C381B5723BE`
- Archive comment matches the upstream baseline commit.

The upstream LICENSE and NOTICE remain intact. Before distributing a changed
application, record copied source and third-party asset licenses, replace the
upstream product identity and update endpoints, and include required notices.

## Baseline checks on Windows, 2026-10-17

- Bun 1.3.14: `bun install --frozen-lockfile` succeeded. The tracked lockfile
  did not change. Upstream pins Bun 1.3.4; use that version for reproduction.
- Bun 1.3.4: obtained with `bun x bun@1.3.4`; its frozen install also passed
  without tracked lockfile changes.
- Typecheck and production build also passed under Bun 1.3.4. The production
  build took 22.15 seconds; output is in ignored `baseline-frontend-build.log`.
- Rust 1.97.1 MSVC: installed as an additional toolchain without changing the
  default toolchain.
- `bun run --filter @sd/tauri typecheck`: passed.
- `bun run --filter @sd/tauri build`: passed. Existing Sass deprecation,
  annotation, mixed-import, and large-chunk warnings remain.
- `cargo build --locked --release --bin sd-daemon`: started; outcome pending.
- `cargo build --locked --release -p spacedrive`: started; waiting for the
  daemon build's Cargo lock. Output is in ignored `baseline-tauri-build.log`.
- Isolated data startup and visual baseline: pending. The unchanged Tauri app
  resolves its data directory to `~/.spacedrive` and its daemon address to
  `127.0.0.1:6969`. Establish isolation before launching; changing USERPROFILE
  alone does not establish that the Windows known-folder API is isolated.

No runtime code changed. A frontend build does not prove that the daemon or
Windows application works. Do not use the user's Workspace Filer database or
an existing Spacedrive library for baseline testing.

## Next gate

Finish the unchanged daemon and Tauri builds. Establish a dedicated test data
directory and capture the native baseline. Then isolate pane navigation,
selection, key dispatch, and toolbar portals before adding two live listings.

## Initial UI dependency inventory

| Dependency | Installed version | License evidence |
| --- | --- | --- |
| Spacedrive source | baseline above | Root LICENSE and NOTICE, Apache-2.0 |
| `@spacedrive/primitives` | 0.2.3 | npm gitHead `54e01a5940e41ad0c301328105ad03b48b51c7ed`; SpaceUI LICENSE at that commit, MIT |
| `@spacedrive/tokens` | 0.2.3 | Same npm gitHead and SpaceUI LICENSE |
| `@phosphor-icons/react` | 2.1.10 | Installed package LICENSE and MIT metadata |
| React | 19.1.0 | Installed package LICENSE and MIT metadata |
| Tailwind CSS | 4.2.2 | Installed package LICENSE and MIT metadata |

The two SpaceUI packages omit license metadata and license files in their npm
payloads. Their published gitHead resolves to the public SpaceUI repository;
the license at that revision is preserved in `docs/licenses/spaceui-MIT.txt`.
Source: https://github.com/spacedriveapp/spaceui/blob/54e01a5940e41ad0c301328105ad03b48b51c7ed/LICENSE

This inventory covers the first UI dependencies, not all transitive libraries,
images, fonts, or the final distribution. Complete that inventory before release.

## Native test launch preparation

The `workspace-isolation` worktree preserves the original baseline build input
while preparing native testing. Set both `SD_WORKSPACE_TEST_DATA_DIR` (an absolute
directory) and `SD_WORKSPACE_TEST_DAEMON_ADDR` (a numeric loopback endpoint on a
dedicated, nonzero port). Missing one setting, relative paths, non-loopback
addresses and the normal port 6969 are rejected before app startup.

Start the named daemon separately with a dedicated base directory, `--instance`
and `--no-default-sources`. Use its actual instance directory and listening
address for the two settings. The Tauri test process does not start a default
daemon when the external endpoint is unavailable. Its application identifier
and configured main-window WebView directory also differ from the normal app.
The ordinary launch retains upstream defaults when neither setting is present.

`cargo test --locked -p sd-tauri-core --target-dir target-isolation` passed all
four tests, covering paired settings, default behavior, rejected destinations,
and a Unicode path. Rust formatting passes with `newline_style=Auto` to preserve
the Windows checkout's existing newlines. The default newline-style check also
flags unchanged upstream CRLF files.

Tauri integration compilation, external-daemon startup, dynamic secondary-window
profiles, actual data isolation and native screen capture remain unverified.
These settings are development test infrastructure, not portable release support.

Follow-up: all dynamic window builders now select the same isolated WebView
directory as the main window, preserving the app's shared browser state while
separating it from the normal installation. Test launches reject daemon service
installation and removal before those commands reach the OS. The configuration
tests still pass. A separate `cargo check --locked -p spacedrive --target-dir
target-isolation` is running; `isolation-tauri-check.log` holds its output.
