use std::collections::HashMap;
use std::sync::Mutex;
use tauri::include_image;
use tauri::menu::{AboutMetadata, CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};

pub struct MenuRegistry(pub Mutex<HashMap<String, MenuItem<tauri::Wry>>>);

pub const MENU_IDS: &[&str] = &[
    "add-files",
    "reveal-in-finder",
    "clear-queue",
    "compress",
    "reset-selected",
    "check-for-update",
];

pub fn build_menu(
    app: &tauri::AppHandle,
) -> tauri::Result<(Menu<tauri::Wry>, MenuRegistry, CheckMenuItem<tauri::Wry>)> {
    // ── App menu (compress[pdf]) ──────────────────────────────────────────
    let about = PredefinedMenuItem::about(
        app,
        Some("About compress[pdf]"),
        Some(AboutMetadata {
            icon: Some(include_image!("icons/icon.png")),
            name: Some("compress[pdf]".to_string()),
            version: Some(env!("CARGO_PKG_VERSION").to_string()),
            copyright: Some("Copyright \u{00A9} 2026 Olajumoke Bolanle".to_string()),
            license: Some("GNU Affero General Public License v3.0".to_string()),
            website: Some("https://github.com/JBolanle/PDFCompressor/".to_string()),
            credits: Some(
                concat!(
            "A fast, offline PDF compressor for macOS.\n\n",
            "This software bundles Ghostscript, \u{00A9} Artifex Software, Inc. (AGPL v3).\n",
            "Source code: https://github.com/ArtifexSoftware/ghostpdl\n\n",
            "This program comes with ABSOLUTELY NO WARRANTY. ",
            "It is free software; you may redistribute it under the terms of the AGPL v3."
        )
                .to_string(),
            ),
            ..Default::default()
        }),
    )?;
    let hide = PredefinedMenuItem::hide(app, Some("Hide compress[pdf]"))?;
    let hide_others = PredefinedMenuItem::hide_others(app, None)?;
    let show_all = PredefinedMenuItem::show_all(app, None)?;
    let quit = PredefinedMenuItem::quit(app, Some("Quit compress[pdf]"))?;
    let app_sep1 = PredefinedMenuItem::separator(app)?;
    let app_sep2 = PredefinedMenuItem::separator(app)?;

    let app_menu = Submenu::with_id_and_items(
        app,
        "app-menu",
        "compress[pdf]",
        true,
        &[
            &about,
            &app_sep1,
            &hide,
            &hide_others,
            &show_all,
            &app_sep2,
            &quit,
        ],
    )?;

    // ── File menu ─────────────────────────────────────────────────────────
    let add_files = MenuItem::with_id(app, "add-files", "Add Files\u{2026}", true, Some("cmd+o"))?;
    let reveal = MenuItem::with_id(
        app,
        "reveal-in-finder",
        "Reveal in Finder",
        false,
        Some("cmd+shift+r"),
    )?;
    let sep = PredefinedMenuItem::separator(app)?;
    let clear_queue = MenuItem::with_id(
        app,
        "clear-queue",
        "Clear Queue",
        false,
        Some("cmd+shift+backspace"),
    )?;

    let file_menu = Submenu::with_id_and_items(
        app,
        "file-menu",
        "File",
        true,
        &[&add_files, &reveal, &sep, &clear_queue],
    )?;

    // ── Queue menu ────────────────────────────────────────────────────────
    let compress = MenuItem::with_id(app, "compress", "Compress", false, Some("cmd+return"))?;
    let reset = MenuItem::with_id(
        app,
        "reset-selected",
        "Reset Selected",
        false,
        Some("cmd+r"),
    )?;

    let queue_menu =
        Submenu::with_id_and_items(app, "queue-menu", "Queue", true, &[&compress, &reset])?;

    // ── Window menu ───────────────────────────────────────────────────────
    let minimize = PredefinedMenuItem::minimize(app, None)?;
    let close_window = PredefinedMenuItem::close_window(app, None)?;
    let win_sep = PredefinedMenuItem::separator(app)?;

    let window_menu = Submenu::with_id_and_items(
        app,
        "window-menu",
        "Window",
        true,
        &[&minimize, &win_sep, &close_window],
    )?;

    // ── Help menu ─────────────────────────────────────────────────────────
    let auto_update = CheckMenuItem::with_id(
        app,
        "check-for-update-auto",
        "Check for Updates Automatically",
        true,
        false,
        None::<&str>,
    )?;
    let help_sep = PredefinedMenuItem::separator(app)?;
    let check_for_updates = MenuItem::with_id(
        app,
        "check-for-update",
        "Check for Updates\u{2026}",
        true,
        None::<&str>,
    )?;

    let help_menu = Submenu::with_id_and_items(
        app,
        "help-menu",
        "Help",
        true,
        &[&auto_update, &help_sep, &check_for_updates],
    )?;

    let menu = Menu::with_items(
        app,
        &[&app_menu, &file_menu, &queue_menu, &window_menu, &help_menu],
    )?;

    let mut map = HashMap::new();
    map.insert("add-files".to_string(), add_files);
    map.insert("reveal-in-finder".to_string(), reveal);
    map.insert("clear-queue".to_string(), clear_queue);
    map.insert("compress".to_string(), compress);
    map.insert("reset-selected".to_string(), reset);
    map.insert("check-for-update".to_string(), check_for_updates);

    Ok((menu, MenuRegistry(Mutex::new(map)), auto_update))
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
        assert!(ids.contains("check-for-update"));
    }

    #[test]
    fn menu_ids_sync_with_build_menu() {
        // Note: build_menu requires AppHandle and cannot be called in unit tests.
        // Instead we maintain a parallel list here that must be kept in sync with
        // the map.insert calls in build_menu.
        // Hardcoded list of IDs that are inserted in build_menu's HashMap
        let build_menu_ids: &[&str] = &[
            "add-files",
            "reveal-in-finder",
            "clear-queue",
            "compress",
            "reset-selected",
            "check-for-update",
        ];

        // Assert length matches
        assert_eq!(
            MENU_IDS.len(),
            build_menu_ids.len(),
            "MENU_IDS and build_menu HashMap have different lengths"
        );

        // Assert all MENU_IDS appear in build_menu_ids
        let build_menu_set: std::collections::HashSet<&str> =
            build_menu_ids.iter().copied().collect();
        for id in MENU_IDS {
            assert!(
                build_menu_set.contains(id),
                "ID '{}' in MENU_IDS is not inserted in build_menu",
                id
            );
        }

        // Assert all build_menu_ids appear in MENU_IDS
        let menu_ids_set: std::collections::HashSet<&str> = MENU_IDS.iter().copied().collect();
        for id in build_menu_ids {
            assert!(
                menu_ids_set.contains(id),
                "ID '{}' inserted in build_menu is not in MENU_IDS",
                id
            );
        }
    }
}
