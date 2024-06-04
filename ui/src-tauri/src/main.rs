// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::menu::{SubmenuBuilder, Menu, MenuItem};

const BAK9_CMD: &str = "bak9";

#[tauri::command]
fn run_bak9_scheduled() -> Result<u32, String> {
  Ok(
    std::process::Command::new(BAK9_CMD) 
      .spawn().unwrap().id())
}

fn main() {
    let mut builder = tauri::Builder::default();

    builder = builder.setup(move |app| {
            let handle = app.handle();
            let menu = Menu::new(handle)?;

            let submenu = SubmenuBuilder::new(handle, "System")
              .item(&MenuItem::new(handle, "Summary", true, None::<&str>)?)
              .item(&MenuItem::new(handle, "Status", true, None::<&str>)?)
              .item(&MenuItem::new(handle, "Logs", true, None::<&str>)?)
              .separator()
              .item(&MenuItem::new(handle, "Backup now ...", true, None::<&str>)?)
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
              .item(&MenuItem::new(handle, "Setup ...", true, None::<&str>)?)
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
        });

    builder
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![run_bak9_scheduled])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

