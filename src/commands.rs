use std::{
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
};

/// Holds the commands that are currently available. It can be thought of as a cache of what would
/// be available through the $PATH in a conventional shell.
#[derive(Debug)]
pub struct Commands {
    paths: HashMap<String, PathBuf>,
}

impl Commands {
    /// Searches the given paths for commands, removing all commands that were stored previously.
    pub fn rescan<'a>(&mut self, search_paths: impl Iterator<Item = &'a Path>) -> io::Result<()> {
        self.paths.clear();

        for search_path in search_paths {
            for path in fs::read_dir(search_path)? {
                let path = path?.path();

                if let Some(command_name) = path.file_name() {
                    self.paths
                        .insert(command_name.to_string_lossy().into(), path);
                }
            }
        }

        Ok(())
    }

    /// Obtains the path to a command based on its name, if it exists.
    pub fn get(&self, command_name: &str) -> Option<PathBuf> {
        self.paths.get(command_name).map(Clone::clone)
    }
}

impl Default for Commands {
    fn default() -> Self {
        Self {
            paths: HashMap::new(),
        }
    }
}
