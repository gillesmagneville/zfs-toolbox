// SPDX-License-Identifier: GPL-3.0-only
//! Main window: a sidebar of sections and the page of the selected section.

use adw::prelude::*;
use gettextrs::gettext;
use gtk::gio;

use crate::overview::Overview;

/// A section not implemented yet, shown as a placeholder page.
struct Upcoming {
    name: &'static str,
    title: String,
    icon: &'static str,
    description: String,
}

fn upcoming_sections() -> Vec<Upcoming> {
    vec![
        Upcoming {
            name: "pools",
            title: gettext("Pools"),
            icon: "drive-multidisk-symbolic",
            description: gettext("Create, import, export and inspect storage pools."),
        },
        Upcoming {
            name: "datasets",
            title: gettext("Datasets"),
            icon: "folder-symbolic",
            description: gettext(
                "Manage filesystems and volumes, mount them and load their encryption keys.",
            ),
        },
        Upcoming {
            name: "snapshots",
            title: gettext("Snapshots"),
            icon: "document-open-recent-symbolic",
            description: gettext("Take snapshots and schedule them with a retention policy."),
        },
        Upcoming {
            name: "replication",
            title: gettext("Replication"),
            icon: "emblem-synchronizing-symbolic",
            description: gettext("Copy datasets to another pool or another server."),
        },
        Upcoming {
            name: "health",
            title: gettext("Health"),
            icon: "utilities-system-monitor-symbolic",
            description: gettext("Follow the state of your disks and run scrubs."),
        },
    ]
}

pub fn build(app: &adw::Application) -> adw::ApplicationWindow {
    let toasts = adw::ToastOverlay::new();
    let stack = gtk::Stack::builder()
        .hexpand(true)
        .vexpand(true)
        .transition_type(gtk::StackTransitionType::Crossfade)
        .build();

    let overview = Overview::new(&toasts);
    stack.add_titled(overview.widget(), Some("overview"), &gettext("Overview"));

    for section in upcoming_sections() {
        let page = adw::StatusPage::builder()
            .icon_name(section.icon)
            .title(&section.title)
            .description(format!(
                "{}\n\n{}",
                section.description,
                gettext("This section will be available in a future version.")
            ))
            .build();
        stack.add_titled(&page, Some(section.name), &section.title);
    }

    toasts.set_child(Some(&stack));

    let sidebar = gtk::StackSidebar::builder()
        .stack(&stack)
        .width_request(200)
        .build();
    let body = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    body.append(&sidebar);
    body.append(&gtk::Separator::new(gtk::Orientation::Vertical));
    body.append(&toasts);

    let refresh = gtk::Button::builder()
        .icon_name("view-refresh-symbolic")
        .tooltip_text(gettext("Refresh"))
        .build();
    let menu = gio::Menu::new();
    menu.append(Some(&gettext("About ZFS Toolbox")), Some("app.about"));
    menu.append(Some(&gettext("Quit")), Some("app.quit"));
    let menu_button = gtk::MenuButton::builder()
        .icon_name("open-menu-symbolic")
        .menu_model(&menu)
        .tooltip_text(gettext("Main menu"))
        .build();

    let header = adw::HeaderBar::new();
    header.pack_start(&refresh);
    header.pack_end(&menu_button);

    let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
    content.append(&header);
    content.append(&body);

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("ZFS Toolbox")
        .default_width(960)
        .default_height(640)
        .content(&content)
        .build();

    {
        let overview = overview.clone();
        refresh.connect_clicked(move |_| overview.refresh());
    }
    overview.refresh();
    window
}
