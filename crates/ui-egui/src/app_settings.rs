//! Per-app settings. Selecting a release edits a draft; only an explicit action installs it.

use crate::app::{Dialog, LauncherUi};
use crate::i18n::{fmt, msg};
use crate::theme::Tokens;
use crate::widgets::{self, Kind};
use artcraft_launcher_core::protocol::{AppPreferences, Command};
use artcraft_launcher_core::{InstallSource, Os, PackageKind, platform::pick_asset};

pub(crate) struct Draft {
    pub app: String,
    tag: String,
    pin: bool,
    prereleases: bool,
    package: Option<PackageKind>,
    pub directory: String,
    custom: bool,
    installation_tab: bool,
}

impl Draft {
    fn preferences(&self) -> AppPreferences {
        AppPreferences {
            pinned_tag: self.pin.then(|| self.tag.clone()),
            install_dir: (self.custom && !self.directory.trim().is_empty()).then(|| self.directory.trim().to_string()),
            package: self.package,
        }
    }
}

impl LauncherUi {
    pub(crate) fn load_app_releases(&mut self, id: &str, page: u32, force: bool) {
        let Some(entry) = self.entries.iter_mut().find(|e| e.app.id == id) else { return };
        if entry.history_loading.is_some() {
            return;
        }
        entry.history_loading = Some(page);
        self.send(Command::LoadReleases { app: id.to_string(), page, force });
    }

    pub(crate) fn open_app_settings(&mut self, id: &str) {
        let Some(entry) = self.entry(id) else { return };
        let preferences = self.settings.apps.get(id).cloned().unwrap_or_default();
        let tag = preferences
            .pinned_tag
            .clone()
            .or_else(|| {
                let installed = entry.installed.as_ref().and_then(|i| i.version.as_ref());
                entry
                    .releases
                    .iter()
                    .chain(entry.release())
                    .find(|r| r.version.as_ref().is_some_and(|v| Some(v) == installed))
                    .map(|r| r.tag.clone())
                    .or_else(|| installed.map(|v| format!("v{v}")))
            })
            .or_else(|| entry.release().map(|r| r.tag.clone()))
            .unwrap_or_default();
        let load = entry.history_page == 0;
        let pin = preferences.pinned_tag.is_some() || !self.settings.apps.contains_key(id);
        self.app_settings = Some(Draft {
            app: id.to_string(),
            tag,
            pin,
            prereleases: self.settings.prereleases,
            package: preferences.package,
            custom: preferences.install_dir.is_some(),
            directory: preferences.install_dir.unwrap_or_default(),
            installation_tab: false,
        });
        self.dialog = Some(Dialog::AppSettings(id.to_string()));
        if load {
            self.load_app_releases(id, 1, false);
        }
    }

    pub(crate) fn app_settings_dialog(&mut self, ui: &mut egui::Ui, id: &str) -> bool {
        let Some(entry) = self.entry(id).cloned() else { return true };
        let Some(mut draft) = self.app_settings.take().filter(|d| d.app == id) else { return true };
        let tokens = Tokens::get(ui.ctx());
        ui.set_width(560.0f32.min((ui.ctx().content_rect().width() - 64.0).max(260.0)));
        ui.label(widgets::heading(entry.app.name, 24.0, tokens.ink_strong));
        widgets::hud(ui, tl!("App settings"), tokens.muted);
        ui.add_space(14.0);
        ui.horizontal(|ui| {
            ui.selectable_value(&mut draft.installation_tab, false, tl!("Version"));
            ui.selectable_value(&mut draft.installation_tab, true, tl!("Installation"));
        });
        ui.separator();
        let busy = entry.job.is_some();
        let mut refresh = false;
        let mut more = false;
        let mut browse = false;
        let mut install = false;
        let mut save = false;
        let mut selected = None;
        let os = self.platform.map(|p| p.os);
        let existing_kind = entry.installed.as_ref().and_then(|installed| match installed.source {
            InstallSource::Msi => Some(PackageKind::Msi),
            InstallSource::MacApp => Some(PackageKind::Dmg),
            InstallSource::Deb => Some(PackageKind::Deb),
            InstallSource::Rpm => Some(PackageKind::Rpm),
            InstallSource::Flatpak => Some(PackageKind::Flatpak),
            InstallSource::AppImage => Some(PackageKind::AppImage),
            InstallSource::Launcher if os == Some(Os::Windows) => Some(PackageKind::PortableZip),
            InstallSource::Launcher if installed.location.to_lowercase().ends_with(".appimage") => Some(PackageKind::AppImage),
            _ => None,
        });
        let kinds: Vec<_> = existing_kind
            .or(draft.package)
            .map(|kind| vec![kind])
            .unwrap_or_else(|| self.platform.map(|p| p.download_kinds().to_vec()).unwrap_or_default());
        let stems: Vec<_> = entry.app.stems().collect();
        let mut releases = entry.releases.clone();
        if let Some(latest) = entry.release()
            && !releases.iter().any(|r| r.tag == latest.tag)
        {
            releases.insert(0, latest.clone());
        }
        if draft.tag.is_empty()
            && let Some(release) = releases.iter().find(|r| {
                (draft.prereleases || !r.prerelease && r.version.as_ref().is_none_or(|v| v.pre.is_empty()))
                    && self.platform.is_some_and(|p| pick_asset(r, &stems, p, &kinds).is_some())
            })
        {
            draft.tag = release.tag.clone();
        }
        let current = entry.installed.as_ref().and_then(|i| i.version.as_ref());
        let mut can_install = false;
        let mut same_version = false;
        let mut msi_downgrade = false;
        egui::ScrollArea::vertical().max_height(ui.ctx().content_rect().height() * 0.55).show(ui, |ui| {
            if !draft.installation_tab {
                ui.label(fmt(
                    tl!("Installed: {version}"),
                    &[("version", &entry.installed_label().unwrap_or_else(|| tl!("Not installed").to_string()))],
                ));
                if self.settings.apps.get(id).is_some_and(|p| p.pinned_tag.is_some()) {
                    widgets::hud(ui, tl!("Version is kept"), tokens.muted);
                }
                ui.add_space(10.0);
                ui.label(tl!("Selected release"));
                ui.add_enabled_ui(!busy, |ui| {
                    egui::ComboBox::from_id_salt(("app-version", id))
                        .width(ui.available_width().min(420.0))
                        .selected_text(if draft.tag.is_empty() { tl!("Select a release").to_string() } else { draft.tag.clone() })
                        .show_ui(ui, |ui| {
                            for release in
                                releases.iter().filter(|r| draft.prereleases || !r.prerelease && r.version.as_ref().is_none_or(|v| v.pre.is_empty()))
                            {
                                let compatible = self.platform.is_some_and(|p| pick_asset(release, &stems, p, &kinds).is_some());
                                let label = if release.version.as_ref().is_some_and(|v| Some(v) == current) {
                                    format!("{} · {}", release.tag, tl!("Installed"))
                                } else if entry.release().is_some_and(|latest| latest.tag == release.tag) {
                                    format!("{} · {}", release.tag, tl!("Latest"))
                                } else {
                                    release.tag.clone()
                                };
                                ui.add_enabled_ui(compatible, |ui| {
                                    ui.selectable_value(&mut draft.tag, release.tag.clone(), label);
                                });
                            }
                        });
                    ui.checkbox(&mut draft.prereleases, tl!("Show pre-releases"));
                });
                ui.horizontal(|ui| {
                    if entry.history_loading.is_some() {
                        ui.spinner();
                        ui.label(tl!("Loading releases…"));
                    }
                    refresh = ui.add_enabled(entry.history_loading.is_none() && !busy, egui::Button::new(tl!("Refresh releases"))).clicked();
                    if entry.history_more {
                        more = ui.add_enabled(entry.history_loading.is_none() && !busy, egui::Button::new(tl!("Load older releases"))).clicked();
                    }
                });
                if let Some(error) = &entry.history_error {
                    ui.colored_label(tokens.bad, msg(error));
                }
                if entry.history_page > 0 && releases.is_empty() {
                    ui.label(tl!("No published releases are available."));
                }
                selected = releases.iter().find(|r| r.tag == draft.tag).cloned();
                if let Some(release) = &selected {
                    let asset = self.platform.and_then(|p| pick_asset(release, &stems, p, &kinds));
                    same_version = release.version.as_ref().is_some_and(|v| Some(v) == current);
                    let downgrade = release.version.as_ref().zip(current).is_some_and(|(target, current)| target < current);
                    msi_downgrade = downgrade && asset.is_some_and(|(_, kind)| kind == PackageKind::Msi);
                    let blocked = downgrade && asset.is_some_and(|(_, kind)| matches!(kind, PackageKind::Rpm | PackageKind::Flatpak));
                    let recovery_available = !msi_downgrade
                        || releases.iter().any(|r| {
                            r.version.as_ref().is_some_and(|v| Some(v) == current)
                                && self
                                    .platform
                                    .and_then(|p| pick_asset(r, &stems, p, &[PackageKind::Msi]))
                                    .is_some_and(|(a, _)| a.sha256.is_some() || r.sums_asset().is_some())
                        });
                    let stable_msi = !msi_downgrade
                        || release.version.as_ref().zip(current).is_some_and(|(target, current)| target.pre.is_empty() && current.pre.is_empty());
                    let checksums = asset.is_some_and(|(asset, _)| asset.sha256.is_some() || release.sums_asset().is_some());
                    can_install = asset.is_some()
                        && checksums
                        && !blocked
                        && recovery_available
                        && stable_msi
                        && !same_version
                        && release.version.is_some()
                        && (!draft.custom || !draft.directory.trim().is_empty())
                        && (draft.prereleases || !release.prerelease && release.version.as_ref().is_none_or(|v| v.pre.is_empty()));
                    if blocked {
                        ui.label(tl!("This package cannot downgrade here. Use a portable installation instead."));
                    } else if !recovery_available {
                        ui.label(tl!("Load the installed release before downgrading its MSI package."));
                    } else if !stable_msi {
                        ui.label(tl!("MSI downgrades require stable releases."));
                    } else if asset.is_none() {
                        ui.label(tl!("No compatible download for this release."));
                    } else if !checksums {
                        ui.label(tl!("This release has no published checksum."));
                    } else if release.version.is_none() {
                        ui.label(tl!("This release's version cannot be verified."));
                    } else if msi_downgrade {
                        ui.label(tl!("MSI downgrade requires removing the current version first. You will confirm before it starts."));
                    }
                    if downgrade && !blocked && !msi_downgrade {
                        ui.label(tl!("An older version will replace the installed program."));
                    }
                    widgets::hud_link(ui, tl!("Release notes"), &release.html_url);
                }
                ui.add_space(12.0);
                ui.add_enabled_ui(!busy, |ui| {
                    ui.checkbox(&mut draft.pin, tl!("Keep the selected version"));
                });
                ui.label(if draft.pin {
                    tl!("Automatic updates and Update all will leave this version unchanged.")
                } else {
                    tl!("Follow the latest release using the global update settings.")
                });
                ui.add_space(12.0);
                let label = fmt(tl!("Switch to {version}"), &[("version", &draft.tag)]);
                ui.add_enabled_ui(can_install && !busy && entry.detected && self.caps.manage_apps, |ui| {
                    install = widgets::button(ui, &label, Kind::Primary, 0.0).clicked();
                });
            } else {
                ui.label(tl!("Current installation folder"));
                if let Some(installed) = &entry.installed {
                    let location = &installed.location;
                    let binary_path = matches!(installed.source, InstallSource::Msi | InstallSource::AppImage | InstallSource::Binary)
                        || installed.source == InstallSource::Launcher && (os == Some(Os::Windows) || location.to_lowercase().ends_with(".appimage"));
                    let folder =
                        if binary_path { location.rsplit_once(['\\', '/']).map_or(location.as_str(), |(folder, _)| folder) } else { location };
                    ui.label(folder);
                    if ui.button(tl!("Open installation folder")).clicked() {
                        self.send(Command::Reveal { app: id.to_string() });
                    }
                    ui.label(tl!("Version changes keep the existing installation folder and package format."));
                } else {
                    ui.label(tl!("Not installed"));
                }
                ui.add_space(12.0);
                ui.add_enabled_ui(!busy && entry.installed.is_none(), |ui| {
                    ui.label(tl!("Package format"));
                    egui::ComboBox::from_id_salt(("app-package", id))
                        .selected_text(draft.package.map_or(tl!("System default"), |k| crate::i18n::t(k.label())))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut draft.package, None, tl!("System default"));
                            let portable = match os {
                                Some(Os::Windows) => Some(PackageKind::PortableZip),
                                Some(Os::Linux) => Some(PackageKind::AppImage),
                                _ => None,
                            };
                            if let Some(kind) = portable {
                                ui.selectable_value(&mut draft.package, Some(kind), crate::i18n::t(kind.label()));
                            }
                        });
                });
                let custom_supported = matches!(existing_kind.or(draft.package), Some(PackageKind::PortableZip | PackageKind::AppImage));
                if !custom_supported && entry.installed.is_none() {
                    draft.custom = false;
                }
                ui.add_enabled_ui(custom_supported && !busy && entry.installed.is_none(), |ui| {
                    ui.checkbox(&mut draft.custom, tl!("Choose a folder for a new installation"));
                    if draft.custom {
                        ui.text_edit_singleline(&mut draft.directory);
                        browse = ui.button(tl!("Browse…")).clicked();
                    }
                });
                if !custom_supported {
                    ui.label(tl!("Custom folders are available for portable ZIP and AppImage packages."));
                }
                ui.add_space(10.0);
                if entry.installed.is_some() && ui.add_enabled(!busy, egui::Button::new(tl!("Uninstall…"))).clicked() {
                    self.dialog = Some(Dialog::ConfirmUninstall(id.to_string()));
                }
            }
        });
        ui.separator();
        if busy {
            ui.label(tl!("Installation in progress. You can close this window."));
        }
        let mut close = false;
        ui.horizontal(|ui| {
            save = ui
                .add_enabled(!busy && (!draft.pin || same_version) && !draft.installation_tab, egui::Button::new(tl!("Save update preferences")))
                .clicked();
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                close = widgets::button(ui, tl!("Close"), Kind::Secondary, 90.0).clicked();
            });
        });
        if refresh {
            self.load_app_releases(id, 1, true);
        }
        if more {
            self.load_app_releases(id, entry.history_page.saturating_add(1), false);
        }
        if browse {
            draft.custom = true;
            self.send(Command::PickInstallDirectory { app: id.to_string() });
        }
        if save {
            self.settings.apps.insert(id.to_string(), draft.preferences());
        }
        if install && let Some(release) = selected {
            if msi_downgrade {
                self.dialog = Some(Dialog::ConfirmMsiDowngrade { app: id.to_string(), tag: release.tag, preferences: draft.preferences() });
            } else {
                self.send(Command::InstallRelease {
                    app: id.to_string(),
                    tag: release.tag,
                    preferences: draft.preferences(),
                    confirmed_msi_downgrade: false,
                });
            }
        }
        self.app_settings = Some(draft);
        close
    }
}
