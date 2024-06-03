// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

use tauri::menu::{SubmenuBuilder, Menu, MenuItem };

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(move |app| {
            let handle = app.handle();
            let menu = Menu::new(handle)?;

            let submenu = SubmenuBuilder::new(handle, "System")
              .item(&MenuItem::new(handle, "Summary", true, None::<&str>)?)
              .item(&MenuItem::new(handle, "Status", true, None::<&str>)?)
              .item(&MenuItem::new(handle, "Logs", true, None::<&str>)?)
              .build()?;
            menu.append(&submenu)?;

            let submenu = SubmenuBuilder::new(handle, "Network")
              .item(&MenuItem::new(handle, "Status", true, None::<&str>)?)
              .item(&MenuItem::new(handle, "Logs", true, None::<&str>)?)
              .build()?;
            menu.append(&submenu)?;

            let submenu = SubmenuBuilder::new(handle, "Config")
              .item(&MenuItem::new(handle, "Settings", true, None::<&str>)?)
              .item(&MenuItem::new(handle, "Schedule", true, None::<&str>)?)
              .separator()
              .item(&MenuItem::new(handle, "Setup", true, None::<&str>)?)
              .build()?;
            menu.append(&submenu)?;

            let submenu = SubmenuBuilder::new(handle, "Backups")
              .item(&MenuItem::new(handle, "This Month", true, None::<&str>)?)
              .item(&MenuItem::new(handle, "Last Month", true, None::<&str>)?)
              .separator()
              .item(&MenuItem::new(handle, "Archives", true, None::<&str>)?)
              .build()?;
            menu.append(&submenu)?;

            let submenu = SubmenuBuilder::new(handle, "About")
              .item(&MenuItem::new(handle, "Manual", true, None::<&str>)?)
              .item(&MenuItem::new(handle, "License", true, None::<&str>)?)
              .item(&MenuItem::new(handle, "Website", true, None::<&str>)?)
              .separator()
              .item(&MenuItem::new(handle, "Version", true, None::<&str>)?)
              .build()?;
            menu.append(&submenu)?;


            let _ = app.set_menu(menu);
            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
