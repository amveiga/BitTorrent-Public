use super::{peers, torrents};
use crate::bit_torrent::BitTorrent;
use gtk::glib;
use gtk::glib::Receiver as GtkReceiver;
use gtk::glib::Sender as GtkSender;
use gtk::prelude::*;
use gtk::ApplicationWindow;
use gtk::Builder;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

pub fn build_ui(application: &gtk::Application) {
    // Comunication
    let mut bit_torrent_instance = BitTorrent::new();
    let (path_tx, path_rx): (Sender<String>, Receiver<String>) = mpsc::channel();

    let (gtk_tx_torrent, gtk_rx_torrent): (
        GtkSender<torrents::TorrentData>,
        GtkReceiver<torrents::TorrentData>,
    ) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

    let gtk_tx_torrent = Mutex::new(gtk_tx_torrent);
    let gtk_tx_torrent = Arc::new(gtk_tx_torrent);

    thread::spawn(move || {
        for received in path_rx {
            bit_torrent_instance.new_leecher(&received, Arc::clone(&gtk_tx_torrent));
        }
    });

    // TODO
    // let (gtk_tx_peers, gtk_rx_peers): (GtkSender<peers::PeersData>, GtkReceiver<peers::PeersData>) =
    //     glib::MainContext::channel(glib::PRIORITY_DEFAULT);

    let glade_src = include_str!("../frontend/glade_sitos.glade");
    let builder = Builder::from_string(glade_src);

    let window: ApplicationWindow = builder.object("window").expect("Couldn't get window");
    window.set_application(Some(application));

    let data = vec![];
    torrents::get_view(&builder, &data, gtk_rx_torrent, path_tx);

    // Dialogs
    let help_dialog: gtk::MessageDialog = builder
        .object("help_dialog")
        .expect("Couldn't get help dialog");
    let settings_dialog: gtk::MessageDialog = builder
        .object("settings_dialog")
        .expect("Couldn't get settings dialog");
    let torrent_dialog: gtk::MessageDialog = builder
        .object("torrent_dialog")
        .expect("Couldn't get torrent dialog");

    // Peer view
    let peers_info: gtk::Button = builder
        .object("btn_peers")
        .expect("Couldn't get peers info");

    // Events
    let builder_clone = builder.clone();
    peers_info.connect_clicked(glib::clone!(@weak application => move |_| {
        let peers_data = vec![
            peers::PeersData {
            id: String::from("ID 1"),
            ip: String::from("IP 1"),
            port: String::from("8080"),
            connection: peers::ConnectionData {
                down_speed: 10,
                up_speed: 10,
                client_status: String::from("Unchocked"),
                peer_status: String::from("Chocked"),
            },
            },
            peers::PeersData {
                id: String::from("ID 2"),
                ip: String::from("IP 2"),
                port: String::from("8080"),
                connection: peers::ConnectionData {
                    down_speed: 15,
                    up_speed: 15,
                    client_status: String::from("Chocked"),
                    peer_status: String::from("Unchocked"),
                },
            },
        ];
        peers::create_peer_window(&application, &builder_clone, &peers_data);
    }));

    builder.connect_signals(move |_, handler_name| match handler_name {
        "on_help_btn_open_clicked" => Box::new(
            glib::clone!(@weak help_dialog => @default-return None, move |_| {
                help_dialog.run();
                None
            }),
        ),
        "on_help_btn_close_clicked" => Box::new(
            glib::clone!(@weak help_dialog => @default-return None, move |_| {
                help_dialog.hide();
                None
            }),
        ),
        "on_btn_settings_open_clicked" => Box::new(
            glib::clone!(@weak settings_dialog => @default-return None, move |_| {
                settings_dialog.run();
                None
            }),
        ),
        "on_btn_settings_close_clicked" => Box::new(
            glib::clone!(@weak settings_dialog => @default-return None, move |_| {
                settings_dialog.hide();
                None
            }),
        ),
        "on_btn_torrent_info_clicked" => Box::new(
            glib::clone!(@weak torrent_dialog => @default-return None, move |_| {
                torrent_dialog.run();
                None
            }),
        ),
        "on_torrent_btn_close_clicked" => Box::new(
            glib::clone!(@weak torrent_dialog => @default-return None, move |_| {
                torrent_dialog.hide();
                None
            }),
        ),
        _ => Box::new(|_| None),
    });

    window.show_all();
}
