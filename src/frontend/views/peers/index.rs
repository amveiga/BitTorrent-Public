use crate::bit_torrent::PeerConnection;
use crate::bit_torrent::State;
use gtk::glib;
use gtk::glib::Receiver as GtkReceiver;
use gtk::pango;
use gtk::prelude::*;
use gtk::ListStore;
use gtk::TreeIter;
use std::rc::Rc;

#[derive(Debug)]
pub struct PeersData {
    pub ip: String,
    pub port: String,
    pub connection: ConnectionData,
    pub torrent_pathname: String,
    pub remove: bool,
}
#[derive(Debug)]
pub struct ConnectionData {
    pub down_speed: String,
    pub up_speed: String,
    pub status: String,
}

impl PeersData {
    pub fn refresh(peer_connection: &PeerConnection, remove: bool) {
        let mut down_speed = String::from("-");

        if peer_connection.instant.elapsed().as_millis() > 0 {
            let down_speed_num = (peer_connection.common_information.piece_length as f64
                / (peer_connection.instant.elapsed().as_millis() as f64 / 1000_f64))
                as f64;

            if down_speed_num / 1073741824_f64 > 1_f64 {
                down_speed = format!("{:.2} GB/s", down_speed_num / 1073741824_f64);
            } else if down_speed_num / 1048576_f64 > 1_f64 {
                down_speed = format!("{:.2} MB/s", down_speed_num / 1048576_f64);
            } else if down_speed_num / 1024_f64 > 1_f64 {
                down_speed = format!("{:.2} KB/s", down_speed_num / 1024_f64);
            }
        }

        let status = match peer_connection.state {
            State::UnknownToPeer => "UnknownToPeer",
            State::ProcessingHandshakeResponse => "ProcessingHandshakeResponse",
            State::Choked => "Choked",
            State::Downloading => "Downloading",
            State::FileDownloaded => "FileDownloaded",
            State::Useless(_) => "Useless",
        };

        let peers_data = PeersData {
            ip: peer_connection.peer.ip.clone(),
            port: peer_connection.peer.port.clone().to_string(),
            connection: ConnectionData {
                down_speed,
                up_speed: String::from("-"),
                status: status.to_string(),
            },
            torrent_pathname: peer_connection.common_information.torrent_pathname.clone(),
            remove,
        };
        peer_connection
            .common_information
            .tx_peers
            .lock()
            .unwrap()
            .send(peers_data)
            .unwrap();
    }
}

pub fn get_view(builder: &gtk::Builder, data: &[PeersData], gtk_rx_peers: GtkReceiver<PeersData>) {
    // Peers vbox and label
    let vbox_peers: gtk::Box = builder.object("list_box2").expect("Couldn't get vbox");
    let vbox_peers_label = gtk::Label::new(Some("Peers list"));
    let attr_list_peers = pango::AttrList::new();
    let mut attr = pango::AttrFloat::new_scale(2.0);
    attr.set_start_index(0);
    attr_list_peers.insert(attr);
    let mut attr = pango::AttrInt::new_underline(pango::Underline::Single);
    attr.set_start_index(0);
    attr_list_peers.insert(attr);
    let mut attr = pango::AttrColor::new_foreground(0, 0, 0);
    attr.set_start_index(0);
    attr_list_peers.insert(attr);
    vbox_peers_label.set_attributes(Some(&attr_list_peers));
    vbox_peers.add(&vbox_peers_label);

    let sw_peers = gtk::ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
    sw_peers.set_shadow_type(gtk::ShadowType::EtchedIn);
    sw_peers.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
    vbox_peers.add(&sw_peers);

    // Peers list
    let model_peers = Rc::new(create_model_peers(data));
    let treeview_peers = gtk::TreeView::with_model(&*model_peers);
    treeview_peers.set_vexpand(true);
    treeview_peers.set_hexpand(false);
    treeview_peers.set_search_column(1);

    sw_peers.add(&treeview_peers);

    add_columns_peers(&model_peers, &treeview_peers);

    gtk_rx_peers.attach(None, move |msg| {
        let msg = msg;
        {
            let mut found = false;
            if model_peers.iter_children(None).is_some() {
                let tree_iter = model_peers.iter_children(None).expect("No children");
                let mut has_next = true;
                while has_next {
                    let ip = model_peers
                        .value(&tree_iter, 0)
                        .get::<String>()
                        .expect("Treeview selection, column 0");
                    let port = model_peers
                        .value(&tree_iter, 1)
                        .get::<String>()
                        .expect("Treeview selection, column 1");

                    if ip == msg.ip && port == msg.port {
                        found = true;
                        if !msg.remove {
                            update_row(&model_peers, &tree_iter, &msg);
                        } else {
                            model_peers.remove(&(tree_iter));
                        }
                        break;
                    } else {
                        has_next = model_peers.iter_next(&tree_iter)
                    }
                }
                if !found && !msg.remove {
                    insert_peers_row(&model_peers, &msg);
                }
            } else if !msg.remove {
                insert_peers_row(&model_peers, &msg);
            }
        }
        glib::Continue(true)
    });
}

fn create_model_peers(data: &[PeersData]) -> gtk::ListStore {
    let col_types: [glib::Type; 6] = [
        glib::Type::STRING,
        glib::Type::STRING,
        glib::Type::STRING,
        glib::Type::STRING,
        glib::Type::STRING,
        glib::Type::STRING,
    ];

    let store = gtk::ListStore::new(&col_types);

    for (_d_idx, d) in data.iter().enumerate() {
        let down_speed: String = format!("{}KB/S", &d.connection.down_speed);
        let up_speed: String = format!("{}KB/S", &d.connection.up_speed);

        let values: [(u32, &dyn ToValue); 6] = [
            (0, &d.ip),
            (1, &d.port),
            (2, &down_speed),
            (3, &up_speed),
            (4, &d.connection.status),
            (5, &d.torrent_pathname),
        ];
        store.set(&store.append(), &values);
    }

    store
}

fn add_columns_peers(_model: &Rc<gtk::ListStore>, treeview: &gtk::TreeView) {
    // Column for IP
    {
        let renderer = gtk::CellRendererText::new();
        let column = gtk::TreeViewColumn::new();
        column.pack_start(&renderer, true);
        column.set_title("IP");
        column.add_attribute(&renderer, "text", 0);
        column.set_sort_column_id(0);
        treeview.append_column(&column);
    }
    // Column for Port
    {
        let renderer = gtk::CellRendererText::new();
        let column = gtk::TreeViewColumn::new();
        column.pack_start(&renderer, true);
        column.set_title("Port");
        column.add_attribute(&renderer, "text", 1);
        column.set_sort_column_id(1);
        treeview.append_column(&column);
    }
    // Column for Down speed
    {
        let renderer = gtk::CellRendererText::new();
        let column = gtk::TreeViewColumn::new();
        column.pack_start(&renderer, true);
        column.set_title("Down speed");
        column.add_attribute(&renderer, "text", 2);
        column.set_sort_column_id(2);
        treeview.append_column(&column);
    }
    // Column for Up speed
    {
        let renderer = gtk::CellRendererText::new();
        let column = gtk::TreeViewColumn::new();
        column.pack_start(&renderer, true);
        column.set_title("Up speed");
        column.add_attribute(&renderer, "text", 3);
        column.set_sort_column_id(3);
        treeview.append_column(&column);
    }
    // Column for Status
    {
        let renderer = gtk::CellRendererText::new();
        let column = gtk::TreeViewColumn::new();
        column.pack_start(&renderer, true);
        column.set_title("Status");
        column.add_attribute(&renderer, "text", 4);
        column.set_sort_column_id(4);
        treeview.append_column(&column);
    }
    // Column for Torrent path
    {
        let renderer = gtk::CellRendererText::new();
        let column = gtk::TreeViewColumn::new();
        column.pack_start(&renderer, true);
        column.set_title("Torrent path");
        column.add_attribute(&renderer, "text", 5);
        column.set_sort_column_id(5);
        treeview.append_column(&column);
    }
}

// TODO
fn insert_peers_row(list: &Rc<ListStore>, data: &PeersData) {
    let values: [(u32, &dyn ToValue); 6] = [
        (0, &data.ip),
        (1, &data.port),
        (2, &data.connection.down_speed),
        (3, &data.connection.up_speed),
        (4, &data.connection.status),
        (5, &data.torrent_pathname),
    ];
    list.insert_with_values(Some(100), &values);
}

fn update_row(list: &Rc<ListStore>, tree_iter: &TreeIter, data: &PeersData) {
    let values: [(u32, &dyn ToValue); 6] = [
        (0, &data.ip),
        (1, &data.port),
        (2, &data.connection.down_speed),
        (3, &data.connection.up_speed),
        (4, &data.connection.status),
        (5, &data.torrent_pathname),
    ];

    list.set(tree_iter, &values)
}
