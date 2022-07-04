use gtk::glib;

use gtk::pango;
use gtk::prelude::*;
use std::rc::Rc;

#[derive(Debug)]
pub struct PeersData {
    pub id: String,
    pub ip: String,
    pub port: String,
    pub connection: ConnectionData,
}
#[derive(Debug)]
pub struct ConnectionData {
    pub down_speed: u32,
    pub up_speed: u32,
    pub client_status: String,
    pub peer_status: String,
}

pub fn create_peer_window(
    application: &gtk::Application,
    builder: &gtk::Builder,
    peers_data: &[PeersData],
) {
    // Comunication

    let peers_window: gtk::Window = builder
        .object("peers_window")
        .expect("Couldn't get peers window");

    // Peers vbox and label
    let vbox_peers: gtk::Box = builder.object("peers_list_box").expect("Couldn't get vbox");
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
    let model_peers = Rc::new(create_model_peers(peers_data));
    let treeview_peers = gtk::TreeView::with_model(&*model_peers);
    treeview_peers.set_vexpand(true);
    treeview_peers.set_hexpand(false);
    treeview_peers.set_search_column(1);

    sw_peers.add(&treeview_peers);

    add_columns_peers(&model_peers, &treeview_peers);

    // TODO
    /*let model_peers_clone = model_peers.clone();
    gtk_rx.attach(None, move |msg| {
        match msg {
            msg => {
                let tree_iter = model_peers_clone.iter_children(None).unwrap();
                let mut has_next = true;
                while has_next{
                    let id = model_peers_clone.value(&tree_iter, 0).get::<String>().expect("Treeview selection, column 0");
                    if id == msg.id {
                        model_peers_clone.remove(&(tree_iter));
                        insert_peers_row(&model_peers_clone, &msg);
                        break;
                    } else{
                        has_next = model_peers_clone.iter_next(&tree_iter)
                    }
                }
            }
            _ => {}
        }
        glib::Continue(true)
    });*/

    application.add_window(&peers_window);

    peers_window.show_all();
}

fn create_model_peers(data: &[PeersData]) -> gtk::ListStore {
    let col_types: [glib::Type; 7] = [
        glib::Type::STRING,
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

        let values: [(u32, &dyn ToValue); 7] = [
            (0, &d.id),
            (1, &d.ip),
            (2, &d.port),
            (3, &down_speed),
            (4, &up_speed),
            (5, &d.connection.client_status),
            (6, &d.connection.peer_status),
        ];
        store.set(&store.append(), &values);
    }

    store
}

fn add_columns_peers(_model: &Rc<gtk::ListStore>, treeview: &gtk::TreeView) {
    // Column for ID
    {
        let renderer = gtk::CellRendererText::new();
        let column = gtk::TreeViewColumn::new();
        column.pack_start(&renderer, true);
        column.set_title("ID");
        column.add_attribute(&renderer, "text", 0);
        column.set_sort_column_id(0);
        treeview.append_column(&column);
    }
    // Column for IP
    {
        let renderer = gtk::CellRendererText::new();
        let column = gtk::TreeViewColumn::new();
        column.pack_start(&renderer, true);
        column.set_title("IP");
        column.add_attribute(&renderer, "text", 1);
        column.set_sort_column_id(1);
        treeview.append_column(&column);
    }
    // Column for Port
    {
        let renderer = gtk::CellRendererText::new();
        let column = gtk::TreeViewColumn::new();
        column.pack_start(&renderer, true);
        column.set_title("Port");
        column.add_attribute(&renderer, "text", 2);
        column.set_sort_column_id(2);
        treeview.append_column(&column);
    }
    // Column for Down speed
    {
        let renderer = gtk::CellRendererText::new();
        let column = gtk::TreeViewColumn::new();
        column.pack_start(&renderer, true);
        column.set_title("Down speed");
        column.add_attribute(&renderer, "text", 3);
        column.set_sort_column_id(3);
        treeview.append_column(&column);
    }
    // Column for Up speed
    {
        let renderer = gtk::CellRendererText::new();
        let column = gtk::TreeViewColumn::new();
        column.pack_start(&renderer, true);
        column.set_title("Up speed");
        column.add_attribute(&renderer, "text", 4);
        column.set_sort_column_id(4);
        treeview.append_column(&column);
    }
    // Column for Client Port
    {
        let renderer = gtk::CellRendererText::new();
        let column = gtk::TreeViewColumn::new();
        column.pack_start(&renderer, true);
        column.set_title("Client status");
        column.add_attribute(&renderer, "text", 5);
        column.set_sort_column_id(5);
        treeview.append_column(&column);
    }
    // Column for Peer Status
    {
        let renderer = gtk::CellRendererText::new();
        let column = gtk::TreeViewColumn::new();
        column.pack_start(&renderer, true);
        column.set_title("Peer status");
        column.add_attribute(&renderer, "text", 6);
        column.set_sort_column_id(6);
        treeview.append_column(&column);
    }
}

// TODO
// fn insert_peers_row(list: &Rc<ListStore>, data: &PeersData) {
//     let down_speed: String = format!("{}KB/S", &data.connection.down_speed);
//     let up_speed: String = format!("{}KB/S", &data.connection.up_speed);

//     let values: [(u32, &dyn ToValue); 7] = [
//         (0, &data.id),
//         (1, &data.ip),
//         (2, &data.port),
//         (3, &down_speed),
//         (4, &up_speed),
//         (5, &data.connection.client_status),
//         (6, &data.connection.peer_status),
//     ];
//     list.insert_with_values(Some(100), &values);
// }
