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
- `cargo build --locked --release --bin sd-daemon`: passed in 20m 19s.
  Baseline EXE SHA256: `BBEEB444AFF9E274E3D865D354B24D5525E0CAAAE7A0CF44407EBF5511987E44`.
- `cargo build --locked --release -p spacedrive`: failed at the unconditional
  optional-DLL resource glob. Output is in ignored `baseline-tauri-build.log`.
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

The first Tauri integration check failed before compiling the application:
`tauri.windows.conf.json` unconditionally required `../../.deps/bin/*.dll`,
although upstream setup makes that codec bundle optional. The build script now
omits only that glob when no DLLs exist, preserves other resources and explicit
list overrides, and passes the effective configuration to code generation.
No dummy DLL or codec feature was added. Windows codec-dependent previews still
require their optional dependencies.

After this correction, the complete isolated Tauri `cargo check` passed in
12.75 seconds. Existing unused-import and missing-sidecar warnings remain.
The separate native debug build has started. An actual executable launch and
daemon connection are not proven by this check.

The unchanged daemon binary is preserved under the source project's ignored
`tests/.fixtures/spacedrive-baseline/`. The Windows build and test-launch changes
were fast-forwarded into `workspace-mdi` only after both baseline build handles
finished. The daemon's explicit `--data-dir` previously evaluated
`default_data_dir()` eagerly through `unwrap_or`, creating `~/.spacedrive` even
when that directory was not selected. A match now evaluates the default only
when needed. This daemon rebuild passed in 13m29s.

## Native startup observations

The isolated desktop debug build passed. The first native launch displayed the
startup screen with the external daemon absent. It did not create `~/.spacedrive`,
but the configured WebView directory was not created: the main window used the
test application identifier's AppData profile instead. Test launches now create
configured windows explicitly and apply `WebviewWindowBuilder::data_directory`
after reading their configuration. A rebuilt native launch created the expected
`tests/.fixtures/spacedrive-native/instances/mdifork/webview` directory. Logs
confirmed the selected instance directory and `127.0.0.1:7718`. The ordinary data
directory remained absent. This proves startup/profile selection, not a working
file listing; the UI remained on its startup screen without the daemon.

The named-instance daemon smoke test exposed two additional startup behaviors:
volume monitoring attempted automatic drive speed tests, and networking
discovery started despite `--no-default-sources`. The test daemon was stopped.
That switch only controls source registration, so the dedicated test config now
disables volume monitoring, networking and statistics. Bootstrap also now honors
the existing `services.networking_enabled` setting instead of unconditionally
restarting a disabled network service. That daemon rebuild is in progress;
the corrected daemon/desktop connection and live listing remain unverified.
Both desktop test windows were closed after these checks.

## Repeatable native test launcher

`scripts/start-workspace-native-test.ps1` prepares a fresh ignored
`.workspace-test/<id>` directory. It writes the disabled discovery/volume-monitor
settings before starting the daemon, uses a named instance, checks a Pong reply
and listening-process ownership, and passes that instance to the desktop.
It records binary hashes, process IDs, paths and logs in `launch.json`. Closing
the test desktop lets the script stop only its own daemon; test data is retained.
Use the rebuilt daemon that honors networking_enabled.

```powershell
pwsh -File scripts/start-workspace-native-test.ps1 -PrepareOnly
pwsh -File scripts/start-workspace-native-test.ps1 -DesktopPath <built-desktop.exe>
```

The PowerShell parser and PrepareOnly path passed. The generated config uses
AppConfig schema version 6 and contains no library identity or credentials.
The complete launch/cleanup path has not run yet; the daemon rebuild remains
the next gate. This helper requires PowerShell 7.4 or newer on Windows.

Each prepared run also creates `test-files/A/alpha.txt` and
`test-files/B/bravo.txt`, with both directory paths in the manifest. Use these
for the first real listings and later pane independence checks instead of
registering the user's home folder or a complete drive.

## Connected native baseline, 2026-10-17

The networking-config daemon rebuild passed in 12m26s. The complete launcher
then reached its running state, verified Pong and port ownership, and opened the
native Home screen. Closing that window ended the launcher successfully with
both child processes gone. Run evidence is in the ignored directory
`.workspace-test/1be447bf8fe74bca9620f001443a5f12`.

- Daemon SHA256: `EE8F0466FDB1CBBEB83B802C4CBD3F794404A4A05EF193EE315C8C8E2DE9A1E2`.
- Desktop SHA256: `5956C35B58CE3E7B9011799D6B705E7665E9B688A8CDAE4BC12807DF5EDD2524`.
- Main-window WebView directory exists under the selected instance.
- Logs confirm disabled volume monitoring, peer networking and statistics.
  No automatic speed-test or peer endpoint startup was observed in that run.
- Native Home rendered its sidebar, theme and empty storage state. No live
  file listing or two-pane independence is claimed.

This run also found that library SourceManager initialization resolved adapters
and stores from the default directory instead of CoreContext.data_dir. That
created default-directory adapter data despite the selected instance. The
processes were stopped; those files were not deleted or treated as user data
to migrate. SourceManager now resolves both locations from its active context,
and the startup diagnostic reports the effective networking state. Rebuild and
runtime verification of this correction are pending.

The `workspace-dev` Cargo profile disables LTO and dependency optimization for
native development iteration. It leaves release unchanged. Its first daemon
build is in progress; passing this profile will not prove a release artifact.
