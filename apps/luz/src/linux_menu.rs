//! The Linux desktop's global menu: Luz's menus exported over D-Bus as a `com.canonical.dbusmenu`
//! object and registered with the desktop's `com.canonical.AppMenu.Registrar`, so a top bar that
//! shows the focused app's menus (Nodalix's) shows Luz's. The menus are the in-window bar's
//! ([`menu_bar`]); this file only serves them, and a chosen item runs as a native menu's does
//! ([`luz_ui_egui::menubar::run_native_command`]).
//!
//! The registration names this process rather than a window (a Wayland window has no global id),
//! so only a registrar that pairs a menu with its window by process can show it: Nodalix's, whose
//! process also owns `space.unmade.GlobalMenu`. With any other registrar, none, or one that goes
//! away, the menus stay in (or come back to) the window. Keys stay with the window: the shortcuts
//! run as they do without a global menu.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex, PoisonError};

use luz_ui_egui::LuzApp;
use luz_ui_egui::menubar::{MenuNode, NativeCommand, display_item_label, menu_bar, run_item, run_native_command};
use serde_json::{Value, json};
use zbus::blocking::Connection;
use zbus::zvariant::{OwnedValue, StructureBuilder, Value as DValue};

const REGISTRAR: &str = "com.canonical.AppMenu.Registrar";
const REGISTRAR_PATH: &str = "/com/canonical/AppMenu/Registrar";
const NODALIX_MENU: &str = "space.unmade.GlobalMenu";
const MENU_PATH: &str = "/com/getnodalia/GlobalMenu";
const MENU_INTERFACE: &str = "com.canonical.dbusmenu";

/// One row of the exported menu.
#[derive(Clone, Debug, PartialEq)]
enum Row {
    Item { cmd: String, params: Value, label: String, enabled: bool, checked: Option<bool> },
    Separator,
    Submenu { label: String, children: Vec<i32> },
}

/// The menus as the desktop reads them: rows by id (0 is the root, whose children are the
/// top-level menus).
#[derive(Default)]
struct Shared {
    rows: HashMap<i32, Row>,
    root: Vec<i32>,
    revision: u32,
}

impl Shared {
    /// Number the rows of the bar depth first, from 1, in the in-window bar's order. The same tree
    /// always gets the same ids.
    fn rows_of(bar: &[(String, Vec<MenuNode>)]) -> (HashMap<i32, Row>, Vec<i32>) {
        fn level(nodes: &[MenuNode], next: &mut i32, rows: &mut HashMap<i32, Row>) -> Vec<i32> {
            let mut ids = Vec::new();
            for node in nodes {
                let id = *next;
                *next = next.saturating_add(1);
                let row = match node {
                    MenuNode::Item { id, params, label, enabled, checked, .. } => Row::Item {
                        cmd: id.clone(),
                        params: params.clone(),
                        label: display_item_label(id, params, label).to_string(),
                        enabled: *enabled,
                        checked: *checked,
                    },
                    MenuNode::Separator => Row::Separator,
                    MenuNode::Submenu { label, children } => {
                        let children = level(children, next, rows);
                        Row::Submenu { label: luz_ui_egui::i18n::tr(label).to_string(), children }
                    }
                };
                rows.insert(id, row);
                ids.push(id);
            }
            ids
        }
        let (mut rows, mut next, mut root) = (HashMap::new(), 1i32, Vec::new());
        for (title, nodes) in bar {
            if nodes.is_empty() {
                continue;
            }
            let id = next;
            next = next.saturating_add(1);
            let children = level(nodes, &mut next, &mut rows);
            rows.insert(id, Row::Submenu { label: luz_ui_egui::i18n::tr(title).to_string(), children });
            root.push(id);
        }
        (rows, root)
    }

    fn children(&self, id: i32) -> &[i32] {
        match self.rows.get(&id) {
            _ if id == 0 => &self.root,
            Some(Row::Submenu { children, .. }) => children,
            _ => &[],
        }
    }

    /// `id`'s dbusmenu properties.
    fn properties(&self, id: i32) -> HashMap<String, OwnedValue> {
        let mut p = HashMap::new();
        let mut put = |k: &str, v: DValue<'_>| {
            if let Ok(v) = v.try_to_owned() {
                p.insert(k.to_string(), v);
            }
        };
        match self.rows.get(&id) {
            None if id == 0 => put("children-display", DValue::from("submenu")),
            None => {}
            Some(Row::Separator) => put("type", DValue::from("separator")),
            Some(Row::Submenu { label, .. }) => {
                put("label", DValue::from(mnemonic_free(label)));
                put("children-display", DValue::from("submenu"));
            }
            Some(Row::Item { label, enabled, checked, .. }) => {
                put("label", DValue::from(mnemonic_free(label)));
                put("enabled", DValue::from(*enabled));
                if let Some(on) = checked {
                    put("toggle-type", DValue::from("checkmark"));
                    put("toggle-state", DValue::from(i32::from(*on)));
                }
            }
        }
        p
    }

    /// `id` and `depth` levels of its children (`-1`: all), the reply to `GetLayout`.
    fn layout_item(&self, id: i32, depth: i32) -> LayoutItem {
        let children = if depth == 0 {
            Vec::new()
        } else {
            self.children(id).iter().filter_map(|&c| self.layout(c, depth.saturating_sub(1)).try_to_owned().ok()).collect()
        };
        LayoutItem { id, properties: self.properties(id), children }
    }

    /// [`Shared::layout_item`] as a variant's value: how dbusmenu nests the children.
    fn layout(&self, id: i32, depth: i32) -> DValue<'static> {
        let children: Vec<DValue<'static>> =
            if depth == 0 { Vec::new() } else { self.children(id).iter().map(|&c| self.layout(c, depth.saturating_sub(1))).collect() };
        let children = zbus::zvariant::Array::from(children);
        let built = StructureBuilder::new().add_field(id).add_field(self.properties(id)).append_field(DValue::Array(children)).build();
        built.map(DValue::Structure).unwrap_or_else(|_| DValue::from(0))
    }
}

/// A label with its underscores doubled: dbusmenu reads a single one as a mnemonic.
fn mnemonic_free(label: &str) -> String {
    label.replace('_', "__")
}

/// One item of the menu tree as dbusmenu sends it: `(ia{sv}av)`, each child a variant holding the
/// same structure.
#[derive(Debug, serde::Serialize, zbus::zvariant::Type)]
struct LayoutItem {
    id: i32,
    properties: HashMap<String, OwnedValue>,
    children: Vec<OwnedValue>,
}

/// The `com.canonical.dbusmenu` object.
struct MenuObject {
    shared: Arc<Mutex<Shared>>,
    commands: Mutex<Sender<(String, Value)>>,
    ctx: egui::Context,
}

impl MenuObject {
    fn lock(&self) -> std::sync::MutexGuard<'_, Shared> {
        self.shared.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// An enabled item's command goes to the app; anything else (a hover, a submenu, a disabled
    /// item, a separator) does nothing.
    fn chosen(&self, id: i32, kind: &str) {
        if kind != "clicked" {
            return;
        }
        let command = match self.lock().rows.get(&id) {
            Some(Row::Item { cmd, params, enabled: true, .. }) => (cmd.clone(), params.clone()),
            _ => return,
        };
        // The app gone means nothing is left to run it.
        let _ = self.commands.lock().unwrap_or_else(PoisonError::into_inner).send(command);
        self.ctx.request_repaint();
    }
}

#[zbus::interface(name = "com.canonical.dbusmenu")]
impl MenuObject {
    #[zbus(property)]
    fn version(&self) -> u32 {
        3
    }
    #[zbus(property)]
    fn text_direction(&self) -> String {
        "ltr".into()
    }
    #[zbus(property)]
    fn status(&self) -> String {
        "normal".into()
    }
    #[zbus(property)]
    fn icon_theme_path(&self) -> Vec<String> {
        Vec::new()
    }

    fn get_layout(&self, parent_id: i32, recursion_depth: i32, _property_names: Vec<String>) -> (u32, LayoutItem) {
        let s = self.lock();
        (s.revision, s.layout_item(parent_id, recursion_depth))
    }

    fn get_group_properties(&self, ids: Vec<i32>, _property_names: Vec<String>) -> Vec<(i32, HashMap<String, OwnedValue>)> {
        let s = self.lock();
        // An empty list asks for every item (the dbusmenu contract).
        let ids = if ids.is_empty() { s.rows.keys().copied().collect() } else { ids };
        ids.into_iter().filter(|id| s.rows.contains_key(id)).map(|id| (id, s.properties(id))).collect()
    }

    fn get_property(&self, id: i32, name: String) -> OwnedValue {
        self.lock().properties(id).remove(&name).unwrap_or_else(|| OwnedValue::from(0i32))
    }

    fn event(&self, id: i32, event_id: String, _data: OwnedValue, _timestamp: u32) {
        self.chosen(id, &event_id);
    }

    fn event_group(&self, events: Vec<(i32, String, OwnedValue, u32)>) -> Vec<i32> {
        for (id, kind, _, _) in events {
            self.chosen(id, &kind);
        }
        Vec::new()
    }

    fn about_to_show(&self, _id: i32) -> bool {
        false
    }

    fn about_to_show_group(&self, ids: Vec<i32>) -> (Vec<i32>, Vec<i32>) {
        let s = self.lock();
        let unknown = ids.into_iter().filter(|id| *id != 0 && !s.rows.contains_key(id)).collect();
        (Vec::new(), unknown)
    }
}

/// Whether the registrar on the bus is Nodalix's, the one registrar that finds a window's menu by
/// its process.
fn nodalix_registrar(conn: &Connection) -> bool {
    let Ok(dbus) = zbus::blocking::fdo::DBusProxy::new(conn) else { return false };
    let owner = |name: &str| zbus::names::BusName::try_from(name).ok().and_then(|n| dbus.get_name_owner(n).ok()).map(|o| o.to_string());
    matches!((owner(REGISTRAR), owner(NODALIX_MENU)), (Some(r), Some(n)) if r == n)
}

/// Hand the menu to the registrar, and record whether it took it.
fn register(conn: &Connection, shown: &AtomicBool, ctx: &egui::Context) {
    let path = zbus::zvariant::ObjectPath::from_static_str_unchecked(MENU_PATH);
    let accepted = nodalix_registrar(conn)
        && conn.call_method(Some(REGISTRAR), REGISTRAR_PATH, Some(REGISTRAR), "RegisterWindow", &(std::process::id(), path)).is_ok();
    shown.store(accepted, Ordering::Relaxed);
    ctx.request_repaint();
    log::info!("global menu: {}", if accepted { "shown in the desktop's top bar" } else { "kept in the window" });
}

/// The global menu, kept by the app for its lifetime.
pub struct GlobalMenu {
    commands: Receiver<(String, Value)>,
    shared: Arc<Mutex<Shared>>,
    conn: Connection,
    shown: Arc<AtomicBool>,
}

impl GlobalMenu {
    /// Export the menus and register them with the desktop's registrar, now and whenever a
    /// registrar appears. `None` (the menus stay in the window) without a session bus.
    pub fn install(ctx: &egui::Context, app: &LuzApp) -> Option<Self> {
        let (rows, root) = Shared::rows_of(&menu_bar(app));
        let shared = Arc::new(Mutex::new(Shared { rows, root, revision: 1 }));
        let (tx, commands) = channel();
        let object = MenuObject { shared: shared.clone(), commands: Mutex::new(tx), ctx: ctx.clone() };
        let conn = match zbus::blocking::connection::Builder::session().and_then(|b| b.serve_at(MENU_PATH, object)).and_then(|b| b.build()) {
            Ok(conn) => conn,
            Err(e) => {
                log::info!("global menu: no session bus ({e}); the menus stay in the window");
                return None;
            }
        };
        let shown = Arc::new(AtomicBool::new(false));
        register(&conn, &shown, ctx);
        // The registrar may start after Luz, or restart with the shell: register whenever it
        // appears, and take the menus back into the window while it is gone.
        let (watch_conn, watch_shown, watch_ctx) = (conn.clone(), shown.clone(), ctx.clone());
        let spawned = std::thread::Builder::new().name("global-menu".into()).spawn(move || {
            let Ok(dbus) = zbus::blocking::fdo::DBusProxy::new(&watch_conn) else { return };
            let Ok(changes) = dbus.receive_name_owner_changed() else { return };
            for change in changes {
                let Ok(args) = change.args() else { continue };
                if args.name().as_str() != REGISTRAR && args.name().as_str() != NODALIX_MENU {
                    continue;
                }
                if args.new_owner().is_some() {
                    register(&watch_conn, &watch_shown, &watch_ctx);
                } else {
                    watch_shown.store(false, Ordering::Relaxed);
                    watch_ctx.request_repaint();
                }
            }
        });
        if let Err(e) = spawned {
            log::warn!("global menu: couldn't watch the registrar ({e})");
        }
        Some(Self { commands, shared, conn, shown })
    }

    /// Once a frame: run the chosen items, hide the in-window menus while the desktop shows
    /// them, and take a changed menu tree to the desktop.
    pub fn update(&mut self, app: &mut LuzApp) {
        while let Ok((cmd, params)) = self.commands.try_recv() {
            if run_native_command(app, &cmd, params) == NativeCommand::Quit {
                // Quit from the top bar quits as the in-window item does, through the same
                // unsaved-changes check.
                let _ = run_item(app, "app.quit", json!({}));
            }
        }
        app.native_menu = self.shown.load(Ordering::Relaxed);
        let (rows, root) = Shared::rows_of(&menu_bar(app));
        let revision = {
            let mut s = self.shared.lock().unwrap_or_else(PoisonError::into_inner);
            if s.rows == rows && s.root == root {
                return;
            }
            s.rows = rows;
            s.root = root;
            s.revision = s.revision.wrapping_add(1);
            s.revision
        };
        // The desktop reads the menus again when told they changed; a failure leaves it on the
        // last ones it read.
        let _ = self.conn.emit_signal(None::<&str>, MENU_PATH, MENU_INTERFACE, "LayoutUpdated", &(revision, 0i32));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: &str, label: &str, enabled: bool) -> MenuNode {
        MenuNode::Item { id: id.into(), params: Value::Null, label: label.into(), shortcut: None, enabled, checked: Some(false) }
    }

    fn bar() -> Vec<(String, Vec<MenuNode>)> {
        vec![
            (
                "File".into(),
                vec![
                    item("file.import", "Import_Photos", true),
                    MenuNode::Separator,
                    MenuNode::Submenu {
                        label: "Open Recent".into(),
                        children: vec![item("app.quit", "Quit Luz", true), item("x.off", "Off", false)],
                    },
                ],
            ),
            ("Empty".into(), Vec::new()),
            ("Edit".into(), vec![item("edit.undo", "Undo", true)]),
        ]
    }

    #[test]
    fn rows_follow_the_in_window_bar() {
        let (rows, root) = Shared::rows_of(&bar());
        // An empty menu is left out.
        assert_eq!(root, vec![1, 7]);
        let s = Shared { rows, root, ..Default::default() };
        let text = |id: i32, k: &str| s.properties(id).get(k).and_then(|v| <&str>::try_from(v).ok().map(str::to_string));
        assert_eq!(text(1, "label").as_deref(), Some("File"));
        assert_eq!(s.children(1), &[2, 3, 4]);
        // Mnemonics are escaped, separators typed, checks shown.
        assert_eq!(text(2, "label").as_deref(), Some("Import__Photos"));
        assert_eq!(text(3, "type").as_deref(), Some("separator"));
        assert_eq!(text(2, "toggle-type").as_deref(), Some("checkmark"));
        assert_eq!(s.children(4), &[5, 6]);
        // The layout carries the whole tree, typed as dbusmenu reads it.
        use zbus::zvariant::Type as _;
        assert_eq!(LayoutItem::SIGNATURE.to_string(), "(ia{sv}av)");
        assert_eq!(s.layout_item(0, -1).children.len(), 2);
        assert!(s.layout_item(0, 0).children.is_empty());
    }

    #[test]
    fn a_click_sends_the_command_and_a_disabled_item_nothing() {
        let (rows, root) = Shared::rows_of(&bar());
        let (tx, rx) = channel();
        let object = MenuObject {
            shared: Arc::new(Mutex::new(Shared { rows, root, ..Default::default() })),
            commands: Mutex::new(tx),
            ctx: egui::Context::default(),
        };
        object.chosen(2, "clicked");
        object.chosen(6, "clicked");
        object.chosen(3, "clicked");
        object.chosen(2, "hovered");
        assert_eq!(rx.try_iter().map(|(c, _)| c).collect::<Vec<_>>(), vec!["file.import".to_string()]);
    }
}
