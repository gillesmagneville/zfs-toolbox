// SPDX-License-Identifier: GPL-3.0-or-later
//! The "Overview" page: OpenZFS version, imported pools and a check of the
//! administrator access.

use std::rc::Rc;

use adw::prelude::*;
use gettextrs::{gettext, ngettext};
use gtk::{gio, glib};
use zfs_core::pool::{Health, Pool};
use zfs_core::runner::SystemRunner;
use zfs_core::units::format_bytes;
use zfs_core::version::ZfsVersion;
use zfs_core::Error;

use crate::helper_client;
use crate::i18n::gettext_f;

/// What the page shows, gathered off the main thread.
enum State {
    /// `zfs` is not installed.
    Missing,
    Ready {
        version: Option<ZfsVersion>,
        pools: Result<Vec<Pool>, String>,
    },
    Failed(String),
}

fn load() -> State {
    let runner = SystemRunner;
    match zfs_core::version::detect(&runner) {
        Err(Error::ToolNotFound(_)) => State::Missing,
        Err(error) => State::Failed(error.to_string()),
        Ok(version) => State::Ready {
            version,
            pools: zfs_core::pool::list(&runner).map_err(|e| e.to_string()),
        },
    }
}

pub fn health_label(health: &Health) -> String {
    match health {
        Health::Online => gettext("Online"),
        Health::Degraded => gettext("Degraded"),
        Health::Faulted => gettext("Faulted"),
        Health::Offline => gettext("Offline"),
        Health::Unavail => gettext("Unavailable"),
        Health::Removed => gettext("Removed"),
        Health::Suspended => gettext("Suspended"),
        Health::Other(raw) => raw.clone(),
    }
}

pub struct Overview {
    bin: adw::Bin,
    toasts: adw::ToastOverlay,
}

impl Overview {
    pub fn new(toasts: &adw::ToastOverlay) -> Rc<Self> {
        Rc::new(Self {
            bin: adw::Bin::new(),
            toasts: toasts.clone(),
        })
    }

    pub fn widget(&self) -> &adw::Bin {
        &self.bin
    }

    /// Reloads the page in the background.
    pub fn refresh(self: &Rc<Self>) {
        let spinner = gtk::Spinner::builder()
            .spinning(true)
            .width_request(32)
            .height_request(32)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Center)
            .build();
        self.bin.set_child(Some(&spinner));

        let this = Rc::clone(self);
        glib::spawn_future_local(async move {
            let state = gio::spawn_blocking(load)
                .await
                .unwrap_or_else(|_| State::Failed(gettext("The status check crashed.")));
            this.show(state);
        });
    }

    fn show(self: &Rc<Self>, state: State) {
        let child: gtk::Widget = match state {
            State::Missing => adw::StatusPage::builder()
                .icon_name("dialog-warning-symbolic")
                .title(gettext("OpenZFS is not installed"))
                .description(gettext(
                    "Install the OpenZFS tools (the zfsutils-linux package on Debian and Ubuntu, the zfs package on Red Hat and derivatives), then refresh.",
                ))
                .build()
                .upcast(),
            State::Failed(detail) => adw::StatusPage::builder()
                .icon_name("dialog-error-symbolic")
                .title(gettext("Could not read the OpenZFS status"))
                .description(detail)
                .build()
                .upcast(),
            State::Ready { version, pools } => self.ready_page(version, pools).upcast(),
        };
        self.bin.set_child(Some(&child));
    }

    fn ready_page(
        self: &Rc<Self>,
        version: Option<ZfsVersion>,
        pools: Result<Vec<Pool>, String>,
    ) -> adw::PreferencesPage {
        let page = adw::PreferencesPage::new();

        // OpenZFS versions.
        let zfs_group = adw::PreferencesGroup::builder().title("OpenZFS").build();
        let unknown = gettext("Unknown");
        let tools = version
            .as_ref()
            .map_or(unknown.clone(), |v| v.userland.clone());
        let module = match &version {
            Some(ZfsVersion {
                kernel: Some(k), ..
            }) => k.clone(),
            Some(_) => gettext("Not loaded"),
            None => unknown,
        };
        zfs_group.add(&info_row(&gettext("Tools version"), &tools));
        zfs_group.add(&info_row(&gettext("Kernel module"), &module));
        page.add(&zfs_group);

        // Imported pools.
        let pools_group = adw::PreferencesGroup::builder()
            .title(gettext("Pools"))
            .build();
        match pools {
            Err(detail) => {
                pools_group.add(&info_row(&gettext("Could not list the pools"), &detail));
            }
            Ok(pools) if pools.is_empty() => {
                pools_group.add(&info_row(&gettext("No imported pool"), ""));
            }
            Ok(pools) => {
                let count = pools.len().to_string();
                pools_group.set_description(Some(
                    &ngettext("{} imported pool", "{} imported pools", pools.len() as u32)
                        .replacen("{}", &count, 1),
                ));
                for pool in &pools {
                    let row = adw::ActionRow::builder()
                        .title(&pool.name)
                        .subtitle(gettext_f(
                            "{} used of {}",
                            &[&format_bytes(pool.allocated), &format_bytes(pool.size)],
                        ))
                        .build();
                    let health = gtk::Label::new(Some(&health_label(&pool.health)));
                    health.add_css_class(if pool.health.is_problem() {
                        "error"
                    } else {
                        "success"
                    });
                    row.add_suffix(&health);
                    pools_group.add(&row);
                }
            }
        }
        page.add(&pools_group);

        // Administrator access.
        let admin_group = adw::PreferencesGroup::builder()
            .title(gettext("Administrator access"))
            .description(gettext(
                "Operations that change your pools ask for an administrator password.",
            ))
            .build();
        let test = gtk::Button::builder()
            .label(gettext("Test"))
            .valign(gtk::Align::Center)
            .build();
        let row = adw::ActionRow::builder()
            .title(gettext("Check administrator access"))
            .build();
        row.add_suffix(&test);
        row.set_activatable_widget(Some(&test));
        admin_group.add(&row);
        page.add(&admin_group);

        let this = Rc::clone(self);
        test.connect_clicked(move |button| this.test_admin_access(button));

        page
    }

    fn test_admin_access(self: &Rc<Self>, button: &gtk::Button) {
        button.set_sensitive(false);
        let button = button.clone();
        let toasts = self.toasts.clone();
        glib::spawn_future_local(async move {
            let result =
                gio::spawn_blocking(|| helper_client::call(zfs_core::helper::Request::Ping {}))
                    .await;
            let message = match result {
                Ok(Ok(_)) => gettext("Administrator access works."),
                Ok(Err(error)) => error.message(),
                Err(_) => gettext("The administration helper crashed."),
            };
            toasts.add_toast(adw::Toast::new(&message));
            button.set_sensitive(true);
        });
    }
}

fn info_row(title: &str, value: &str) -> adw::ActionRow {
    let row = adw::ActionRow::builder().title(title).build();
    if !value.is_empty() {
        let label = gtk::Label::builder()
            .label(value)
            .selectable(true)
            .wrap(true)
            .css_classes(["dim-label"])
            .build();
        row.add_suffix(&label);
    }
    row
}
