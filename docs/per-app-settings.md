# Per-app versions and installation settings

Open an app's **⋮ → App settings…** menu in either grid or table view.

Table view keeps the Open and menu buttons visible when resizing; optional information
columns return as the window grows.

The **Version** tab loads published GitHub releases for that app. The dropdown labels the
installed and latest releases. Older pages load on demand, up to 600 releases. Drafts are
excluded; pre-releases are optional. Releases without compatible assets or verifiable
versions cannot be installed. Refresh retries a failed request without changing installations.

Selecting a version edits a draft. **Switch to …** explicitly installs it, checks its published
SHA-256, and confirms the installed version. **Keep the selected version** prevents automatic
updates and **Update all** from replacing it. Preferences become active only after a successful
switch. To change the policy without reinstalling, select the installed release and use
**Save update preferences**. Following the latest release uses the global update settings.

Paused or failed version downloads retain their exact release across restarts. Resuming them
does not silently select the latest release. If a kept version's metadata is no longer loaded,
open its settings before reinstalling it.

The **Installation** tab shows the actual folder, opens it in the file manager, and offers the
existing uninstall confirmation. New portable ZIP and AppImage installations can use a chosen
parent folder. Windows ZIPs install under `<parent>/<app-id>`. Existing installations retain
their location and package format; this feature does not migrate installations.

## Package limits

| Format | Version changes | Custom folder |
| --- | --- | --- |
| Windows portable ZIP | Upgrade and downgrade; executable VERSIONINFO must match the selected release | New installations only |
| Windows MSI | Upgrade; confirmed stable-release downgrade with verified recovery installer | Installer-managed; downgrade preserves the existing folder |
| macOS DMG | Existing bundle replacement and version checks | Existing application location |
| Linux AppImage | Replace verified download at existing location | New installations only |
| Linux DEB | Existing apt installation, including its downgrade support | System-managed |
| Linux RPM / Flatpak | Upgrade; downgrade blocked | System-managed |
| FreeBSD tarball | Existing installer path | Existing prefix |

MSI downgrades require both the selected and installed releases to be loaded with compatible,
checksummed MSI assets. **Switch to …** opens a confirmation; **Uninstall and switch** starts
preparation. Both installers are verified and copied into the portable data's `msi-recovery`
folder before removal. Their app identity, product/upgrade codes, versions, architecture,
installation scope and directory property must match the installed app. The original MSI is
then removed and the target installed at the same location. Failed installation or version
checks trigger a separate attempt to restore the original version. Recovery failure retains
the installers, a recovery record and installer log, and reports their folder. A dismissed
administrator prompt or power loss can prevent recovery; rollback is not guaranteed.

Windows portable replacement backs up recorded program files and rolls back on copy,
version-verification or manifest-save errors. Unrecorded settings and documents are preserved,
including during uninstall. The launcher rejects paths with links/junctions, unsafe ZIP paths,
duplicate names and collisions with unrecorded files. Failed rollback retains recovery files
and reports their location. External installers and power loss have platform-specific limits;
this feature does not promise transactional rollback for them.

Windows orchestration and failure paths are covered by local tests. The actual registered
PhotoCraft MSI was inspected read-only; a real MSI uninstall/install cycle still needs a
disposable Windows environment. Linux/macOS/FreeBSD installer execution still
requires native validation on those platforms. The browser keeps its existing download-only
interface; native settings and installation actions are desktop-only.

## Local development copy

Copy the built desktop executable to a separate folder and place `portable.txt` beside it.
Start it with `--dev-instance`. Its preferences use `ArtCraftLauncherData/app.ron`, its native
data/cache also stay under `ArtCraftLauncherData`, it uses a separate single-instance port,
and it cannot update itself to an official release. The portable marker is required to keep
native data separate. Normal launcher startup and preferences remain unchanged.

Checks: `cargo xtask ci`. Set `RUST_TEST_THREADS=1` for serial UI tests; set
`ARTCRAFT_LAUNCHER_SNAPSHOTS=<folder>` to save rendered English/German settings screenshots.
