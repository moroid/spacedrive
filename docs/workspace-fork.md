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
- Rust 1.97.1 MSVC: installed as an additional toolchain without changing the
  default toolchain.
- `bun run --filter @sd/tauri typecheck`: passed.
- `bun run --filter @sd/tauri build`: passed. Existing Sass deprecation,
  annotation, mixed-import, and large-chunk warnings remain.
- `cargo build --locked --release --bin sd-daemon`: started; outcome pending.
- Tauri executable build, isolated data startup, and visual baseline: pending.

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
