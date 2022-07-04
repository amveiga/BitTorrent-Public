use crate::bit_torrent::{Bitfield, CommonInformation, PeerList};
use gtk::glib;
use gtk::glib::Receiver as GtkReceiver;
use gtk::pango;
use gtk::prelude::*;
use gtk::ListStore;
use gtk::TreeIter;
use std::fmt::Write;
use std::rc::Rc;
use std::sync::mpsc::Sender;

#[derive(Debug)]
pub struct TorrentData {
    pub name: String,
    pub hash: String,
    pub size: String,
    pub pices: u32,
    pub peers: u32,
    pub done: f64,
    pub pices_done: u32,
    pub connections: u32,
    pub torrent_pathname: String,
}

impl TorrentData {
    pub fn refresh(common_information: &CommonInformation, peers: &PeerList, have: &Bitfield) {
        let mut info_hash = String::new();
        for &byte in &common_information.info_hash {
            write!(&mut info_hash, "{:X}", byte).expect("Unable to write");
        }
        let mut size: String = (common_information.file_length).to_string();
        if common_information.file_length / 1048576 > 1 {
            size = format!("{} MB", (common_information.file_length / 1048576));
        } else if common_information.file_length / 1024 > 1 {
            size = format!("{} KB", (common_information.file_length / 1024));
        }
        let torrent_data = TorrentData {
            name: String::from(&common_information.file_name),
            hash: info_hash,
            size,
            pices: common_information.total_pieces.try_into().unwrap(),
            peers: peers.len().try_into().unwrap(),
            done: ((have.status().0 * 100) as f64 / common_information.total_pieces as f64),
            pices_done: have.status().0.try_into().unwrap(),
            connections: peers.active().try_into().unwrap(),
            torrent_pathname: String::from(&common_information.torrent_pathname),
        };
        common_information
            .tx
            .lock()
            .unwrap()
            .send(torrent_data)
            .unwrap();
    }
}

pub fn get_view(
    builder: &gtk::Builder,
    data: &[TorrentData],
    gtk_rx_torrent: GtkReceiver<TorrentData>,
    path_tx: Sender<String>,
) {
    // Torrent vbox and label
    let vbox_torrent: gtk::Box = builder.object("list_box").expect("Couldn't get vbox");
    let vbox_torrent_label = gtk::Label::new(Some("Torrents list"));
    let attr_list_torrent = pango::AttrList::new();
    let mut attr = pango::AttrFloat::new_scale(2.0);
    attr.set_start_index(0);
    attr_list_torrent.insert(attr);
    let mut attr = pango::AttrInt::new_underline(pango::Underline::Single);
    attr.set_start_index(0);
    attr_list_torrent.insert(attr);
    let mut attr = pango::AttrColor::new_foreground(0, 0, 0);
    attr.set_start_index(0);
    attr_list_torrent.insert(attr);
    vbox_torrent_label.set_attributes(Some(&attr_list_torrent));
    vbox_torrent.add(&vbox_torrent_label);

    let sw_torrent = gtk::ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
    sw_torrent.set_shadow_type(gtk::ShadowType::EtchedIn);
    sw_torrent.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
    vbox_torrent.add(&sw_torrent);

    // Torrents list
    let model_torrent = Rc::new(create_model_torrents(data));
    let treeview_torrent = gtk::TreeView::with_model(&*model_torrent);
    treeview_torrent.set_vexpand(true);
    treeview_torrent.set_hexpand(false);
    treeview_torrent.set_search_column(1);

    sw_torrent.add(&treeview_torrent);

    add_columns_torrents(&model_torrent, &treeview_torrent);

    // Dialogs
    let torrent_dialog: gtk::MessageDialog = builder
        .object("torrent_dialog")
        .expect("Couldn't get torrent dialog");

    // Buttons
    let add_torrent: gtk::FileChooserButton = builder
        .object("btn_add_torrent")
        .expect("Couldn't get add torrent");
    let remove_torrent: gtk::Button = builder
        .object("btn_remove_torrent")
        .expect("Couldn't get remove torrent");
    let torrent_info: gtk::Button = builder
        .object("btn_torrent_info")
        .expect("Couldn't get torrent info");

    // Handlers
    let model_torrent_clone = model_torrent.clone();
    let vbox_torrent_label_copy = vbox_torrent_label.clone();
    let attr_list_torrent_clone = attr_list_torrent.clone();
    add_torrent.connect_file_set(glib::clone!(@weak add_torrent => move |_| {
        let torrent_path = add_torrent.filename().unwrap();
        let torrent_extension = torrent_path.extension().expect("extension");

        if torrent_extension != "torrent"{
            let mut attr = pango::AttrColor::new_foreground(65535, 0, 0);
            attr.set_start_index(0);
            attr_list_torrent_clone.insert(attr);
            vbox_torrent_label_copy.set_attributes(Some(&attr_list_torrent_clone));
            vbox_torrent_label_copy.set_label("Error: File must be a torrent (.torrent)");
            return
        }
        let string_send = torrent_path.to_str().unwrap().to_string();
        path_tx.send(string_send).unwrap();

        let torrent_name = torrent_path.file_name();
        let torrent_name = match torrent_name{
            Some(torrent_name) => {format!("{:?}", torrent_name).replace('"', "").replace(".torrent", "")}
            None => {String::from("No name")}
        };
        let torrent_data = TorrentData{
             name: torrent_name,
             hash: String::from(""),
             size: String::from(""),
             pices: 0,
             peers: 0,
             done: 0.0,
             pices_done: 0,
             connections: 0,
             torrent_pathname: String::from(torrent_path.to_str().unwrap()),
        };

        insert_torrent_row(&model_torrent_clone, &torrent_data);
        let mut attr = pango::AttrColor::new_foreground(0, 0, 0);
        attr.set_start_index(0);
        attr_list_torrent_clone.insert(attr);
        vbox_torrent_label_copy.set_attributes(Some(&attr_list_torrent_clone));
        vbox_torrent_label_copy.set_label("Torrent added successfully");  
    }));

    let model_torrent_clone = model_torrent.clone();
    let treeview_torrent_clone = treeview_torrent.clone();
    let vbox_torrent_label_copy = vbox_torrent_label;
    let attr_list_torrent_clone = attr_list_torrent;
    remove_torrent.connect_clicked(glib::clone!(@weak remove_torrent => move |_| {
        let selected = treeview_torrent_clone.selection().selected();
        if let Some(selected) = selected {
            model_torrent_clone.remove(&(selected.1));
            let mut attr = pango::AttrColor::new_foreground(0, 0, 0);
            attr.set_start_index(0);
            attr_list_torrent_clone.insert(attr);
            vbox_torrent_label_copy.set_attributes(Some(&attr_list_torrent_clone));
                vbox_torrent_label_copy.set_label("Torrent removed successfully");
            };
    }));

    let model_torrent_clone = model_torrent.clone();
    let torrent_dialog_clone = torrent_dialog;
    let treeview_torrent_clone = treeview_torrent;
    torrent_info.connect_clicked(glib::clone!(@weak torrent_info => move |_| {
        let selected = treeview_torrent_clone.selection().selected();
        match selected {
             Some(selected) => {
                let iter:TreeIter = selected.1;
                let name = model_torrent_clone.value(&iter, 0).get::<String>().expect("Treeview selection, column 0");
                let info = &format!(
                    "Name: {}\nHash: {}\nSize: {}\nPices: {}\nPeers: {}\nDone: {}\nPices done: {}\nconnections: {}\nPathname: {}\n",
                    name,
                    model_torrent_clone.value(&iter, 1).get::<String>().expect("Treeview selection, column 1"),
                    model_torrent_clone.value(&iter, 2).get::<String>().expect("Treeview selection, column 2"),
                    model_torrent_clone.value(&iter, 3).get::<u32>().expect("Treeview selection, column 3"),
                    model_torrent_clone.value(&iter, 4).get::<u32>().expect("Treeview selection, column 4"),
                    model_torrent_clone.value(&iter, 5).get::<String>().expect("Treeview selection, column 5"),
                    model_torrent_clone.value(&iter, 6).get::<u32>().expect("Treeview selection, column 6"),
                    model_torrent_clone.value(&iter, 7).get::<u32>().expect("Treeview selection, column 7"),
                    model_torrent_clone.value(&iter, 8).get::<String>().expect("Treeview selection, column 8"),
                );
                torrent_dialog_clone.set_text(Some(name.as_str()));
                torrent_dialog_clone.set_secondary_text(Some(info));
            },
            None => {
                torrent_dialog_clone.set_text(Some("Error"));
                torrent_dialog_clone.set_secondary_text(Some("Please select a torrent from the list"));
            }
        }
    }));

    let model_torrent_clone = model_torrent;
    gtk_rx_torrent.attach(None, move |msg| {
        let msg = msg;
        {
            let tree_iter = model_torrent_clone.iter_children(None).unwrap();
            let mut has_next = true;
            while has_next {
                let torrent_pathname = model_torrent_clone
                    .value(&tree_iter, 8)
                    .get::<String>()
                    .expect("Treeview selection, column 8");
                if torrent_pathname == msg.torrent_pathname {
                    model_torrent_clone.remove(&(tree_iter));
                    insert_torrent_row(&model_torrent_clone, &msg);
                    break;
                } else {
                    has_next = model_torrent_clone.iter_next(&tree_iter)
                }
            }
        }
        glib::Continue(true)
    });
}

fn create_model_torrents(data: &[TorrentData]) -> gtk::ListStore {
    let col_types: [glib::Type; 9] = [
        glib::Type::STRING,
        glib::Type::STRING,
        glib::Type::STRING,
        glib::Type::U32,
        glib::Type::U32,
        glib::Type::STRING,
        glib::Type::U32,
        glib::Type::U32,
        glib::Type::STRING,
    ];

    let store = gtk::ListStore::new(&col_types);

    for (_d_idx, d) in data.iter().enumerate() {
        let done_percentage: String = format!("{:.2}%", &d.done);
        let values: [(u32, &dyn ToValue); 9] = [
            (0, &d.name),
            (1, &d.hash),
            (2, &d.size),
            (3, &d.pices),
            (4, &d.peers),
            (5, &done_percentage),
            (6, &d.pices_done),
            (7, &d.connections),
            (8, &d.torrent_pathname),
        ];
        store.set(&store.append(), &values);
    }

    store
}

fn add_columns_torrents(_model: &Rc<gtk::ListStore>, treeview: &gtk::TreeView) {
    // Column for Name
    {
        let renderer = gtk::CellRendererText::new();

        let column = gtk::TreeViewColumn::new();
        column.pack_start(&renderer, true);
        column.set_title("Name");
        column.add_attribute(&renderer, "text", 0);
        column.set_sort_column_id(0);
        treeview.append_column(&column);
    }
    // Column for Hash
    {
        let renderer = gtk::CellRendererText::new();
        let column = gtk::TreeViewColumn::new();
        column.pack_start(&renderer, true);
        column.set_title("Hash");
        column.add_attribute(&renderer, "text", 1);
        column.set_sort_column_id(1);
        treeview.append_column(&column);
    }
    // Column for Size
    {
        let renderer = gtk::CellRendererText::new();
        let column = gtk::TreeViewColumn::new();
        column.pack_start(&renderer, true);
        column.set_title("Size");
        column.add_attribute(&renderer, "text", 2);
        column.set_sort_column_id(2);
        treeview.append_column(&column);
    }
    // Column for Pices
    {
        let renderer = gtk::CellRendererText::new();
        let column = gtk::TreeViewColumn::new();
        column.pack_start(&renderer, true);
        column.set_title("Pices");
        column.add_attribute(&renderer, "text", 3);
        column.set_sort_column_id(3);
        treeview.append_column(&column);
    }
    // Column for Peers
    {
        let renderer = gtk::CellRendererText::new();
        let column = gtk::TreeViewColumn::new();
        column.pack_start(&renderer, true);
        column.set_title("Peers");
        column.add_attribute(&renderer, "text", 4);
        column.set_sort_column_id(4);
        treeview.append_column(&column);
    }
    // Column for Pices done
    {
        let renderer = gtk::CellRendererText::new();
        let column = gtk::TreeViewColumn::new();
        column.pack_start(&renderer, true);
        column.set_title("Done(%)");
        column.add_attribute(&renderer, "text", 5);
        column.set_sort_column_id(5);
        treeview.append_column(&column);
    }
    // Column for Pices done
    {
        let renderer = gtk::CellRendererText::new();
        let column = gtk::TreeViewColumn::new();
        column.pack_start(&renderer, true);
        column.set_title("Pices done");
        column.add_attribute(&renderer, "text", 6);
        column.set_sort_column_id(6);
        treeview.append_column(&column);
    }
    // Column for Connections
    {
        let renderer = gtk::CellRendererText::new();
        let column = gtk::TreeViewColumn::new();
        column.pack_start(&renderer, true);
        column.set_title("Connections");
        column.add_attribute(&renderer, "text", 7);
        column.set_sort_column_id(7);
        treeview.append_column(&column);
    }
    // Column for Torrent path
    {
        let renderer = gtk::CellRendererText::new();
        let column = gtk::TreeViewColumn::new();
        column.pack_start(&renderer, true);
        column.set_title("Torrent path");
        column.add_attribute(&renderer, "text", 8);
        column.set_sort_column_id(8);
        treeview.append_column(&column);
    }
}

fn insert_torrent_row(list: &Rc<ListStore>, data: &TorrentData) {
    let done_percentage: String = format!("{:.2}%", &data.done);
    let values: [(u32, &dyn ToValue); 9] = [
        (0, &data.name),
        (1, &data.hash),
        (2, &data.size),
        (3, &data.pices),
        (4, &data.peers),
        (5, &done_percentage),
        (6, &data.pices_done),
        (7, &data.connections),
        (8, &data.torrent_pathname),
    ];

    list.insert_with_values(Some(100), &values);
}
