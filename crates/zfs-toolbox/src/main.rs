// SPDX-License-Identifier: GPL-3.0-only
//! ZFS Toolbox: a GTK4 desktop application to manage OpenZFS on Linux.

mod config;
mod helper_client;
mod i18n;
mod overview;
mod window;

use adw::prelude::*;
use gettextrs::gettext;
use gtk::{gio, glib};

fn show_about(app: &adw::Application) {
    let mut about = gtk::AboutDialog::builder()
        .program_name("ZFS Toolbox")
        .logo_icon_name(config::APP_ID)
        .version(config::VERSION)
        .comments(gettext(
            "Manage OpenZFS pools, datasets, snapshots and replication",
        ))
        .website(config::REPOSITORY)
        .license_type(gtk::License::Gpl30Only)
        .authors(["Gilles Magneville"])
        .modal(true);
    if let Some(window) = app.active_window() {
        about = about.transient_for(&window);
    }
    about.build().present();
}

fn main() -> glib::ExitCode {
    i18n::init();
    glib::set_application_name("ZFS Toolbox");

    let app = adw::Application::builder()
        .application_id(config::APP_ID)
        .build();

    app.add_action_entries([
        gio::ActionEntry::builder("about")
            .activate(|app: &adw::Application, _, _| show_about(app))
            .build(),
        gio::ActionEntry::builder("quit")
            .activate(|app: &adw::Application, _, _| app.quit())
            .build(),
    ]);
    app.set_accels_for_action("app.quit", &["<Control>q"]);

    app.connect_activate(|app| {
        if let Some(window) = app.active_window() {
            window.present();
        } else {
            window::build(app).present();
        }
    });

    app.run()
}
