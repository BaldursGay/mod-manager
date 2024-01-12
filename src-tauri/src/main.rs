// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::{
    config::{
        get_config, get_config_file_json, get_config_path, set_game_directory,
        set_instances_directory, AppConfig,
    },
    game::autodetect_game_folder,
    instance::{get_instances_index, refresh_instances_index},
    models::instance::InstanceIndex,
    util::open_from_path,
};
use instance::get_instances_index_from_path;
use std::{fs::File, io::prelude::*, path::PathBuf, sync::Mutex};
use tauri::{api::path::app_data_dir, Config};

mod config;
mod error;
mod game;
mod instance;
mod models;
mod util;

pub struct AppState {
    pub config: Mutex<AppConfig>,
    pub instance_index: Mutex<InstanceIndex>,
}

impl AppState {
    pub fn new() -> Result<Self, error::Error> {
        let config_path = get_config_path()?;

        let instances_dir = match app_data_dir(&Config::default()) {
            Some(dir) => dir,
            None => return Err(error::Error::TauriDirectory),
        }
        .join("com.lilydev.bg3mm")
        .join("instances");

        let config = if !config_path.exists() {
            if !&instances_dir.exists() {
                match std::fs::create_dir_all(&instances_dir) {
                    std::io::Result::Ok(_) => println!("Successfully created instances directory"),
                    Err(err) => return Err(error::Error::Io(err)),
                }
            }

            let game_dir: Option<PathBuf> = match game::detect_game_folder() {
                Ok(dir) => Some(dir),
                Err(_) => None,
            };

            AppConfig {
                game_dir,
                instances_dir,
            }
        } else {
            get_config(&config_path)?
        };

        let mut config_file = File::create(&config_path)?;
        config_file.write_all(toml::to_string_pretty(&config)?.as_bytes())?;

        let instances_index_path = config.instances_dir.join("instances.index.json");

        let instance_index: InstanceIndex = if instances_index_path.exists() {
            get_instances_index_from_path(&instances_index_path)?
        } else {
            let index = InstanceIndex { instances: vec![] };
            let mut index_file = File::create(&instances_index_path)?;
            index_file.write_all(serde_json::to_string_pretty(&index)?.as_bytes())?;

            index
        };

        Ok(AppState {
            config: Mutex::from(config),
            instance_index: Mutex::from(instance_index),
        })
    }
}

fn main() -> Result<(), error::Error> {
    let app_state: AppState = AppState::new()?;

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            autodetect_game_folder,
            get_config_file_json,
            get_instances_index,
            open_from_path,
            refresh_instances_index,
            set_game_directory,
            set_instances_directory,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}
