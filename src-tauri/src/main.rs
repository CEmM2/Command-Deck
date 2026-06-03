mod runner;
mod store;

use store::{Category, Config, Template};

#[tauri::command]
fn get_config() -> Config {
    store::load_config()
}

#[tauri::command]
fn set_config(cfg: Config) -> Result<Config, String> {
    store::save_config(&cfg)?;
    Ok(cfg)
}

#[tauri::command]
fn list_categories(cfg: Config) -> Result<Vec<Category>, String> {
    store::load_categories(&cfg.templates_dir)
}

#[tauri::command]
fn save_category(cfg: Config, category: String, templates: Vec<Template>) -> Result<(), String> {
    store::save_category(&cfg.templates_dir, &category, &templates)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_config,
            set_config,
            list_categories,
            save_category,
            runner::run_stream,
            runner::run_capture,
            runner::run_in_terminal,
        ])
        .run(tauri::generate_context!())
        .expect("error while running command deck");
}
