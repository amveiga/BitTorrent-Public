use super::{peers, torrents};
use crate::bit_torrent::BitTorrent;
use gtk::glib;
use gtk::glib::Receiver as GtkReceiver;
use gtk::glib::Sender as GtkSender;
use gtk::prelude::*;
use gtk::ApplicationWindow;
use gtk::Builder;
use std::collections::HashMap;
use std::env;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

pub fn build_ui(application: &gtk::Application) {
    // Comunication
    let bit_torrent_instance = BitTorrent::new();
    let bit_torrent_instance = Arc::new(Mutex::new(bit_torrent_instance));
    let bit_torrent_instance_clone = Arc::clone(&bit_torrent_instance);

    let (path_tx, path_rx): (Sender<String>, Receiver<String>) = mpsc::channel();

    let (gtk_tx_torrent, gtk_rx_torrent): (
        GtkSender<torrents::TorrentData>,
        GtkReceiver<torrents::TorrentData>,
    ) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

    let gtk_tx_torrent = Mutex::new(gtk_tx_torrent);
    let gtk_tx_torrent = Arc::new(gtk_tx_torrent);

    let (gtk_tx_peers, gtk_rx_peers): (GtkSender<peers::PeersData>, GtkReceiver<peers::PeersData>) =
        glib::MainContext::channel(glib::PRIORITY_DEFAULT);

    let gtk_tx_peers = Mutex::new(gtk_tx_peers);
    let gtk_tx_peers = Arc::new(gtk_tx_peers);

    let remove_senders: HashMap<String, Sender<String>> = HashMap::new();
    let remove_senders = Arc::new(Mutex::new(remove_senders));
    let remove_senders_clone = Arc::clone(&remove_senders);

    thread::spawn(move || {
        for received in path_rx {
            let (remove_tx, remove_rx): (Sender<String>, Receiver<String>) = mpsc::channel();
            remove_senders_clone
                .lock()
                .unwrap()
                .insert(received.clone(), remove_tx);
            bit_torrent_instance_clone.lock().unwrap().new_process(
                &received,
                gtk_tx_torrent.clone(),
                gtk_tx_peers.clone(),
                remove_rx,
            );
        }
    });

    let glade_src = include_str!("../frontend/glade_sitos.glade");
    let builder = Builder::from_string(glade_src);

    let window: ApplicationWindow = builder.object("window").expect("Couldn't get window");
    window.set_application(Some(application));

    let torrents_data = vec![];
    torrents::get_view(
        &builder,
        &torrents_data,
        gtk_rx_torrent,
        path_tx,
        remove_senders,
    );

    let peers_data = vec![];
    peers::get_view(&builder, &peers_data, gtk_rx_peers);

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
    let download_path: gtk::Entry = builder
        .object("d_path")
        .expect("Couldn't get download path");

    download_path.set_text(
        env::var("DOWNLOAD_PATH")
            .unwrap_or_else(|_| "".to_string())
            .as_str(),
    );

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
            glib::clone!(@weak settings_dialog, @weak download_path => @default-return None, move |_| {
                env::set_var("DOWNLOAD_PATH", download_path.text());
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
