#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use hopper_core::{Config, Node};
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let name = Config::desktop_name();
            let mut config = Config::desktop(name)?;
            // Optional override makes two isolated desktops easy to exercise during development.
            if let Some(path) = std::env::var_os("HOPPER_DATA_DIR") {
                config.data_dir = path.into();
            }
            let (node, hello) = tauri::async_runtime::block_on(Node::start(config))?;
            node.set_folder_picker(Box::new(|| {
                Box::pin(async {
                    rfd::AsyncFileDialog::new()
                        .set_title("Share a folder with your paired devices")
                        .pick_folder()
                        .await
                        .map(|file| file.path().to_path_buf())
                })
            }));
            app.manage(node);
            let origin = format!("http://127.0.0.1:{}", hello.port);
            WebviewWindowBuilder::new(app, "main", WebviewUrl::External(hello.url().parse()?))
                .title("Hopper")
                .inner_size(1180.0, 820.0)
                .min_inner_size(780.0, 600.0)
                .disable_drag_drop_handler()
                .on_navigation(move |url| url.origin().ascii_serialization() == origin)
                .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("could not run Hopper");
}
