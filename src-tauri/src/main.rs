// No console window in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod config;
mod error;
mod session;
mod storage;
#[cfg(windows)]
mod win;

use tauri::Manager;

use commands::AppState;

fn main() {
    tauri::Builder::default()
        // Two copies writing the same vault would race, so a second launch focuses the first.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.unminimize();
                let _ = w.set_focus();
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::new(config::load()))
        .setup(|app| {
            #[cfg(windows)]
            {
                let window = app.get_webview_window("main").expect("main window is defined in tauri.conf.json");
                let hwnd = window.hwnd()?.0 as windows_sys::Win32::Foundation::HWND;
                win::block_capture(hwnd);
                let handle = app.handle().clone();
                win::watch_session(hwnd, move || {
                    // Runs inside the window procedure, so never wait on the lock here: if it is
                    // busy, the background thread takes care of it within two seconds.
                    let state = handle.state::<AppState>();
                    let guard = state.try_lock();
                    if let Some(mut inner) = guard {
                        commands::lock_now(&handle, &mut inner, "system");
                    } else {
                        let handle = handle.clone();
                        std::thread::spawn(move || {
                            let state = handle.state::<AppState>();
                            commands::lock_now(&handle, &mut state.lock(), "system");
                        });
                    }
                });
            }
            commands::spawn_background(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::status,
            commands::drive_suggestions,
            commands::pick_folder,
            commands::estimate_strength,
            commands::generate_passphrase,
            commands::create_vault,
            commands::use_existing_vault,
            commands::forget_vault,
            commands::unlock,
            commands::lock,
            commands::touch,
            commands::list_entries,
            commands::reveal_entry,
            commands::save_entry,
            commands::delete_entry,
            commands::check_seed,
            commands::suggest_words,
            commands::copy_field,
            commands::export_backup,
            commands::change_passwords,
            commands::practice_unlock,
            commands::save_settings,
            commands::recovery_info,
        ])
        .run(tauri::generate_context!())
        .expect("Keyplate failed to start");
}
