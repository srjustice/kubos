/*
 * Copyright (C) 2018 Kubos Corporation
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use app_entry::*;
use error::*;
use fs_extra;
use kubos_app::RunLevel;
use std::cell::RefCell;
use std::fs;
use std::io::Read;
use std::os::unix;
use std::path::{Path, PathBuf};
use std::process::Command;
use toml;
use uuid::Uuid;

/// The default application registry directory in KubOS
pub const K_APPS_DIR: &'static str = "/home/system/kubos/apps";

/// AppRegistry
#[derive(Deserialize, Serialize, Debug)]
pub struct AppRegistry {
    #[doc(hidden)]
    pub entries: RefCell<Vec<AppRegistryEntry>>,
    /// The managed root directory of the AppRegistry
    pub apps_dir: String,
}

impl AppRegistry {
    /// Create a new AppRegistry located at a specific directory in the filesystem
    ///
    /// # Arguments
    ///
    /// * `apps_dir` - The root directory that applications are loaded from
    ///
    /// # Examples
    ///
    /// ```
    /// # use kubos_app::registry::AppRegistry;
    /// let registry = AppRegistry::new_from_dir("/my/apps");
    /// ```
    pub fn new_from_dir(apps_dir: &str) -> Result<AppRegistry, AppError> {
        let registry = AppRegistry {
            entries: RefCell::new(Vec::new()),
            apps_dir: String::from(apps_dir),
        };

        let active_dir = PathBuf::from(format!("{}/active", apps_dir));
        if !active_dir.exists() {
            fs::create_dir_all(&active_dir)?;
        }

        registry
            .entries
            .borrow_mut()
            .extend(registry.discover_apps()?);

        Ok(registry)
    }

    /// Create a new AppRegistry located at the default directory (see [`K_APPS_DIR`])
    ///
    /// [`K_APPS_DIR`]: constant.K_APPS_DIR.html
    /// # Examples
    ///
    /// ```
    /// # use kubos_app::registry::AppRegistry;
    /// let registry = AppRegistry::new();
    /// ```
    pub fn new() -> Result<AppRegistry, AppError> {
        Self::new_from_dir(K_APPS_DIR)
    }

    fn discover_apps(&self) -> Result<Vec<AppRegistryEntry>, AppError> {
        let mut reg_entries: Vec<AppRegistryEntry> = Vec::new();

        for entry in fs::read_dir(&self.apps_dir)? {
            if let Ok(entry) = entry {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_dir() && entry.file_name().to_str() != Some("active") {
                        reg_entries.extend(self.discover_versions(entry.path())?);
                    }
                }
            }
        }

        Ok(reg_entries)
    }

    fn discover_versions(&self, app_dir: PathBuf) -> Result<Vec<AppRegistryEntry>, AppError> {
        let mut reg_entries: Vec<AppRegistryEntry> = Vec::new();

        for version in fs::read_dir(app_dir)? {
            if version.is_err() {
                continue;
            }

            let version = version.unwrap();

            match version
                .file_type()
                .and_then(|file_type| Ok(file_type.is_dir()))
            {
                Ok(true) => {
                    if let Ok(entry) = AppRegistryEntry::from_dir(&version.path()) {
                        if entry.active_version == true {
                            self.set_active(&entry.app.uuid, &version.path().to_string_lossy())?;
                        }
                        reg_entries.push(entry);
                    } else {
                        // Don't really care if this fails, since this is just trying to help
                        // prevent orphan files
                        let _ = fs::remove_dir_all(version.path());
                    }
                }
                _ => {
                    // Don't really care if this fails, since this is just trying to help
                    // prevent orphan files
                    let _ = fs::remove_dir_all(version.path());
                }
            }
        }

        Ok(reg_entries)
    }

    // Create or update the active version symlink for an application
    fn set_active(&self, uuid: &str, app_dir: &str) -> Result<(), AppError> {
        let active_symlink = PathBuf::from(format!("{}/active/{}", self.apps_dir, uuid));
        if active_symlink.exists() {
            if let Err(err) = fs::remove_file(active_symlink.clone()) {
                return Err(AppError::RegisterError {
                    err: format!(
                        "Couldn't remove symlink {}: {:?}",
                        active_symlink.display(),
                        err
                    ),
                });
            }
        }

        if let Err(err) = unix::fs::symlink(app_dir, active_symlink.clone()) {
            return Err(AppError::RegisterError {
                err: format!(
                    "Couldn't symlink {} to {}: {:?}",
                    active_symlink.display(),
                    app_dir,
                    err
                ),
            });
        }

        Ok(())
    }

    /// Register an application binary with the AppRegistry, extracting metadata and installing it
    /// into the proper folder structure under the AppRegistry directory.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to an application binary
    ///
    /// # Examples
    ///
    /// ```
    /// # use kubos_app::registry::AppRegistry;
    /// let registry = AppRegistry::new();
    /// registry.register("/home/kubos/my-app-bin");
    /// ```
    pub fn register(&self, path: &str, uuid: Option<String>) -> Result<AppRegistryEntry, AppError> {
        let app_path = Path::new(path);
        if !app_path.exists() {
            return Err(AppError::RegisterError {
                err: format!("{} does not exist", path),
            });
        }

        if !app_path.is_dir() {
            return Err(AppError::RegisterError {
                err: format!("{} is not a directory", path),
            });
        }

        // Load the metadata
        let mut data = String::new();
        fs::File::open(app_path.join("manifest.toml"))
            .and_then(|mut fp| fp.read_to_string(&mut data))?;

        let metadata: AppMetadata = match toml::from_str(&data) {
            Ok(val) => val,
            Err(error) => {
                return Err(AppError::ParseError {
                    entity: "manifest.toml".to_owned(),
                    err: error.to_string(),
                })
            }
        };

        // Make sure the file which should be called for execution is present in the directory
        let app_name = metadata.name.clone();
        if !app_path.join(app_name.clone()).exists() {
            return Err(AppError::RegisterError {
                err: format!("Application file {} not found in given path", app_name),
            });
        }

        let mut entries = self.entries.borrow_mut();
        let app_uuid = match uuid {
            Some(val) => {
                for entry in entries.iter_mut() {
                    // Find the existing active version of the app and make it inactive.
                    // Use the existing UUID for our new app
                    if entry.active_version && entry.app.uuid == val {
                        entry.active_version = false;
                        entry.save()?;
                        break;
                    }
                }

                val
            }
            None => Uuid::new_v4().hyphenated().to_string(),
        };

        let app_dir_str = format!(
            "{}/{}/{}",
            self.apps_dir,
            app_uuid,
            metadata.version.as_str()
        );
        let app_dir = Path::new(&app_dir_str);

        if !app_dir.exists() {
            fs::create_dir_all(app_dir)?;
        }

        // Copy everything into the official registry directory
        let files: Vec<PathBuf> = fs::read_dir(app_path)?
            .filter_map(|file| 
                if let Some(entry) = file.ok() {
                    Some(entry.path())
                } else {
                    None
                }
            ).collect();

        fs_extra::copy_items(&files, app_dir, &fs_extra::dir::CopyOptions::new()).map_err(
            |error| AppError::RegisterError {
                err: format!("Error copying files into registry dir: {}", error),
            },
        )?;

        self.set_active(&app_uuid, &app_dir_str)?;

        let reg_entry = AppRegistryEntry {
            app: App {
                uuid: app_uuid,
                metadata: metadata,
                pid: 0,
                path: format!("{}/{}", app_dir_str, app_name),
            },
            active_version: true,
        };

        // Add the new registry entry
        entries.push(reg_entry);
        // Create the app.toml file and save the metadata information
        entries[entries.len() - 1].save()?;
        Ok(entries[entries.len() - 1].clone())
    }

    /// Uninstall an application from the AppRegistry
    ///
    /// # Arguments
    ///
    /// * `app_uuid` - The UUID generated for the app
    /// * `version` - The version of the app to delete
    ///
    /// # Examples
    ///
    /// ```
    /// # use kubos_app::registry::AppRegistry;
    /// let registry = AppRegistry::new();
    /// registry.uninstall("01234567-89ab-cdef0-1234-56789abcdef0", "1.0");
    /// ```
    ///
    pub fn uninstall(&self, app_uuid: &str, version: &str) -> Result<bool, AppError> {
        let mut errors = None;

        // Delete the application files
        let app_dir = format!("{}/{}/{}", self.apps_dir, app_uuid, version);

        if let Err(error) = fs::remove_dir_all(app_dir) {
            errors = Some(format!("Failed to remove app directory: {}", error));
        }

        // Remove the app entry from the registry list
        let mut entries = self.entries.borrow_mut();
        match entries.binary_search_by(|ref e| {
            e.app
                .uuid
                .cmp(&String::from(app_uuid))
                .then(e.app.metadata.version.cmp(&String::from(version)))
        }) {
            Ok(index) => {
                entries.remove(index);
            }
            Err(_) => {
                if let Some(error) = errors {
                    errors = Some(format!(
                        "{}. {} version {} not found in registry",
                        error, app_uuid, version
                    ));
                } else {
                    errors = Some(format!(
                        "{} version {} not found in registry",
                        app_uuid, version
                    ));
                }
            }
        }

        match errors {
            Some(err) => Err(AppError::UninstallError { err }),
            None => Ok(true),
        }
    }

    /// Start an application. If successful, returns the pid of the application process.
    ///
    /// # Arguments
    ///
    /// * `app_uuid` - The UUID generated for the app when it was registered
    /// * `run_level` - Which Run Level to run the app with
    ///
    /// # Examples
    ///
    /// ```
    /// # use kubos_app::registry::{AppRegistry, RunLevel};
    /// let registry = AppRegistry::new();
    /// registry.start_app("01234567-89ab-cdef0-1234-56789abcdef0", RunLevel::OnCommand);
    /// ```
    pub fn start_app(
        &self,
        app_uuid: &str,
        run_level: RunLevel,
        args: Option<Vec<String>>,
    ) -> Result<u32, AppError> {
        // Look up the active version of the requested application
        let app = {
            let entries = self.entries.borrow();
            match entries
                .iter()
                .find(|ref e| e.active_version && e.app.uuid == app_uuid)
            {
                Some(entry) => entry.app.clone(),
                None => {
                    return Err(AppError::StartError {
                        err: format!("No active version found for UUID {}", app_uuid),
                    })
                }
            }
        };

        let app_path = PathBuf::from(&app.path);
        if !app_path.exists() {
            let msg = match self.uninstall(&app.uuid, &app.metadata.version) {
                Ok(_) => format!(
                    "{} does not exist. {} version {} automatically uninstalled",
                    app.path, app.uuid, app.metadata.version
                ),
                Err(error) => format!("{} does not exist. {}", app.path, error),
            };

            return Err(AppError::StartError { err: msg });
        }

        let mut cmd = Command::new(app_path);

        cmd.env("KUBOS_APP_UUID", app.uuid.clone())
            .arg("-r")
            .arg(format!("{}", run_level));

        if let Some(add_args) = args {
            cmd.args(&add_args);
        }

        match cmd.spawn() {
            Ok(child) => Ok(child.id()),
            Err(err) => {
                return Err(AppError::StartError {
                    err: format!("Failed to spawn app: {:?}", err),
                })
            }
        }
    }

    /// Call the active version of all registered applications with the "OnBoot" run level
    ///
    /// # Examples
    ///
    /// ```
    /// # use kubos_app::registry::{AppRegistry, RunLevel};
    /// let registry = AppRegistry::new();
    /// registry.run_onboot();
    /// ```
    pub fn run_onboot(&self) -> Result<(), AppError> {
        let mut apps_started = 0;
        let mut apps_not_started = 0;

        let active_symlink = PathBuf::from(format!("{}/active", self.apps_dir));
        if !active_symlink.exists() {
            return Err(AppError::FileError {
                err: "Failed to get list of active UUIDs".to_owned(),
            });
        }

        for entry in fs::read_dir(active_symlink)? {
            match entry {
                Ok(file) => {
                    let uuid = file.file_name();
                    match self.start_app(&uuid.to_string_lossy(), RunLevel::OnBoot, None) {
                        Ok(_) => apps_started += 1,
                        Err(error) => {
                            eprintln!("Failed to start {}: {:?}", uuid.to_string_lossy(), error);
                            apps_not_started += 1
                        }
                    }
                }
                Err(_) => apps_not_started += 1,
            }
        }

        info!(
            "Apps started: {}, Apps failed: {}",
            apps_started, apps_not_started
        );

        if apps_not_started != 0 {
            return Err(AppError::FileError {
                err: format!("Failed to start {} app/s", apps_not_started),
            });
        }

        Ok(())
    }
}
