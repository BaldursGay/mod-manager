use std::{
    fs::{copy, create_dir_all, read_to_string, remove_dir_all, File},
    io::Write,
    path::PathBuf,
};

use anyhow::anyhow;
use tauri::State;
use uuid::Uuid;

use crate::{
    error::Error,
    models::instance::{InstanceIndex, InstanceInfo},
    AppState,
};

#[tauri::command]
pub fn create_instance(
    instance_name: String,
    image_path: Option<PathBuf>,
    state: State<AppState>,
) -> Result<(), Error> {
    let instance_dir = state.config.lock().unwrap().instances_dir.clone();
    let new_instance_id = Uuid::new_v4();

    let new_instance = InstanceInfo {
        id: new_instance_id,
        name: instance_name,
        order_index: 0,
    };

    create_dir_all(&instance_dir.join(new_instance_id.to_string()))?;

    let mut instance_index: InstanceIndex =
        serde_json::from_str(read_to_string(instance_dir.join("instances.index.json"))?.as_str())
            .unwrap();

    instance_index.instances.push(new_instance);

    let mut index_file = File::create(instance_dir.join("instances.index.json"))?;

    index_file.write_all(serde_json::to_string_pretty(&instance_index)?.as_bytes())?;

    match image_path {
        Some(path) => {
            let file_extension = &*path.extension().unwrap().to_str().unwrap();

            copy(
                &path,
                &instance_dir
                    .join(&new_instance_id.to_string())
                    .join(format!("instance.{}", file_extension)),
            )?;
        }
        None => {}
    }

    Ok(())
}

#[tauri::command]
pub fn delete_instance(instance_id: Uuid, state: State<AppState>) -> Result<(), Error> {
    let instance_dir = state.config.lock().unwrap().instances_dir.clone();
    remove_dir_all(instance_dir.join(instance_id.to_string()))?;

    let mut instance_index: InstanceIndex =
        serde_json::from_str(read_to_string(instance_dir.join("instances.index.json"))?.as_str())
            .unwrap();
    instance_index.instances.retain(|i| i.id != instance_id);

    let mut index_file = File::create(instance_dir.join("instances.index.json"))?;
    index_file.write_all(serde_json::to_string_pretty(&instance_index)?.as_bytes())?;

    Ok(())
}

#[tauri::command]
pub fn get_instance_info(instance_id: Uuid, state: State<AppState>) -> Result<InstanceInfo, Error> {
    let index: InstanceIndex = serde_json::from_str(
        read_to_string(
            state
                .config
                .lock()
                .unwrap()
                .instances_dir
                .clone()
                .join("instances.index.json"),
        )?
        .as_str(),
    )
    .unwrap();

    let info: InstanceInfo = match index.instances.into_iter().find(|i| i.id == instance_id) {
        Some(index) => index,
        None => {
            return Err(Error::Other(anyhow!(format!(
                "Failed to find instance with ID `{}`",
                instance_id,
            ))))
        }
    };

    Ok(info)
}

#[tauri::command]
pub fn get_instances_index(state: State<AppState>) -> Result<InstanceIndex, Error> {
    Ok(state.instance_index.lock().unwrap().clone())
}

#[tauri::command]
pub fn refresh_instances_index(state: State<AppState>) -> Result<(), Error> {
    let new_index = get_instances_index_from_path(
        &state
            .config
            .lock()
            .unwrap()
            .instances_dir
            .join("instances.index.json"),
    )?;

    let mut index_lock = state.instance_index.lock().unwrap();
    index_lock.instances = new_index.instances;

    Ok(())
}

pub fn get_instances_index_from_path(path: &PathBuf) -> Result<InstanceIndex, Error> {
    let index_string = read_to_string(&path)?;
    let index: InstanceIndex = serde_json::from_str(&index_string)?;

    Ok(index)
}
