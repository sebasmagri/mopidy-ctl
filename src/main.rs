extern crate dbus;
extern crate gdk;
extern crate gtk;
extern crate notify_rust;

use std::thread;

use dbus::{Connection, BusType, Message, Error, ConnectionItem};

use gtk::prelude::*;

use notify_rust::Notification;

const PREVIOUS: &'static str = "Previous";
const PLAY_PAUSE: &'static str = "PlayPause";
const STOP: &'static str = "Stop";
const NEXT: &'static str = "Next";

fn player(method: &str) -> Result<Message, Error> {
    let c = Connection::get_private(BusType::Session).unwrap();
    let m = Message::new_method_call("org.mpris.MediaPlayer2.mopidy",
                                     "/org/mpris/MediaPlayer2",
                                     "org.mpris.MediaPlayer2.Player",
                                     method).unwrap();

    c.send_with_reply_and_block(m, 2000)
}

fn display_menu(_: &gtk::StatusIcon, a: u32, b: u32) {
    let menu = gtk::Menu::new();

    for command in vec![PREVIOUS, PLAY_PAUSE, STOP, NEXT].into_iter() {
        let item = gtk::MenuItem::new_with_label(command);
        item.connect_activate(move |_| {
            player(command).unwrap();
        });
        menu.append(&item);
    }
    menu.show_all();

    menu.popup_easy(a, b)
}

fn handle_properties_changed_message(m: &Message) {
    let mut it = m.iter_init();
    println!("{:?}", it.read::<&str>().unwrap());

    let notification_body = match m {
        _ => format!("{:?}", m.get_items())
    };

    Notification::new()
        .summary("mpris-ctl")
        .body(notification_body.as_str())
        .show()
        .unwrap();
}

fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let icon = gtk::StatusIcon::new_from_icon_name("media-playback-start");
    icon.connect_popup_menu(display_menu);

    thread::spawn(move || {
        let monitor_conn = Connection::get_private(BusType::Session).unwrap();
        monitor_conn.add_match("sender=org.mpris.MediaPlayer2.mopidy,path=/org/mpris/MediaPlayer2,interface=org.freedesktop.DBus.Properties,member=PropertiesChanged").unwrap();
        loop {
            let items = monitor_conn.iter(1000);
            for item in items {
                match item {
                    ConnectionItem::Signal(ref msg) => {
                        println!("{:?}", msg);
                        println!("{:?}", msg.get_items());
                        handle_properties_changed_message(msg);
                    },
                    _ => ()
                }
            }
        }
    });

    gtk::main();
}
