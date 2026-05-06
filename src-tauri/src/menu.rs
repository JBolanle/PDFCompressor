use std::collections::HashMap;
use std::sync::Mutex;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};

pub struct MenuRegistry(pub Mutex<HashMap<String, MenuItem<tauri::Wry>>>);

pub const MENU_IDS: &[&str] = &[
    "add-files",
    "reveal-in-finder",
    "clear-queue",
    "compress",
    "reset-selected",
];

pub fn build_menu(app: &tauri::AppHandle) -> tauri::Result<(Menu<tauri::Wry>, MenuRegistry)> {
    let add_files = MenuItem::with_id(app, "add-files", "Add Files\u{2026}", true, Some("cmd+o"))?;
    let reveal = MenuItem::with_id(app, "reveal-in-finder", "Reveal in Finder", false, Some("cmd+shift+r"))?;
    let sep = PredefinedMenuItem::separator(app)?;
    let clear_queue = MenuItem::with_id(app, "clear-queue", "Clear Queue", false, Some("cmd+shift+backspace"))?;

    let file_menu = Submenu::with_id_and_items(
        app, "file-menu", "File", true,
        &[&add_files, &reveal, &sep, &clear_queue],
    )?;

    let compress = MenuItem::with_id(app, "compress", "Compress", false, Some("cmd+return"))?;
    let reset = MenuItem::with_id(app, "reset-selected", "Reset Selected", false, Some("cmd+r"))?;

    let queue_menu = Submenu::with_id_and_items(
        app, "queue-menu", "Queue", true,
        &[&compress, &reset],
    )?;

    let menu = Menu::with_items(app, &[&file_menu, &queue_menu])?;

    let mut map = HashMap::new();
    map.insert("add-files".to_string(), add_files);
    map.insert("reveal-in-finder".to_string(), reveal);
    map.insert("clear-queue".to_string(), clear_queue);
    map.insert("compress".to_string(), compress);
    map.insert("reset-selected".to_string(), reset);

    Ok((menu, MenuRegistry(Mutex::new(map))))
}

#[tauri::command]
pub fn set_menu_item_enabled(
    state: tauri::State<'_, MenuRegistry>,
    id: String,
    enabled: bool,
) -> Result<(), String> {
    let map = state.0.lock().map_err(|e| e.to_string())?;
    if let Some(item) = map.get(&id) {
        item.set_enabled(enabled).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_ids_are_unique() {
        let mut seen = std::collections::HashSet::new();
        for id in MENU_IDS {
            assert!(seen.insert(*id), "Duplicate menu ID: {}", id);
        }
    }

    #[test]
    fn menu_ids_contains_all_expected() {
        let ids: std::collections::HashSet<&str> = MENU_IDS.iter().copied().collect();
        assert!(ids.contains("add-files"));
        assert!(ids.contains("reveal-in-finder"));
        assert!(ids.contains("clear-queue"));
        assert!(ids.contains("compress"));
        assert!(ids.contains("reset-selected"));
    }
}
