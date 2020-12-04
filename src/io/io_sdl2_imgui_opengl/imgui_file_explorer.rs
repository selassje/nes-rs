extern crate imgui;

use std::cmp::Ordering;
use std::env;
use std::path::*;
use std::{fs, io};

use imgui::*;

pub trait UiFileExplorer {
    /// Can filter over several extensions.
    /// Anything that can be treated as a reference to a string `AsRef<str>` can be used as argument!
    /// Return the selected path, if any.
    fn file_explorer<T, S>(&self, target: T, extensions: &[S]) -> io::Result<Option<PathBuf>>
    where
        T: AsRef<Path>,
        S: AsRef<str>;
}

fn has_extension<P: AsRef<Path>, S: AsRef<str>>(path: P, extensions: &[S]) -> bool {
    let path = path.as_ref();
    if let Some(test_ext) = path.extension() {
        if let Some(test_ext) = test_ext.to_str() {
            extensions
                .iter()
                .any(|ext| test_ext.to_lowercase() == ext.as_ref().to_lowercase())
        } else {
            false
        }
    } else {
        false
    }
}

fn is_hidden<P: AsRef<Path>>(path: P) -> bool {
    let path = path.as_ref();
    if let Some(name) = path.file_name() {
        name.to_string_lossy().starts_with('.')
    } else {
        false
    }
}

cfg_if::cfg_if! {
    if #[cfg(unix)] {
        pub const TOP_FOLDER: &str = "C:";
    } else {
        pub const TOP_FOLDER: &str = "C:";
    }
}

/// Ui extends
impl<'ui> UiFileExplorer for Ui<'ui> {
    fn file_explorer<T, S>(&self, target: T, extensions: &[S]) -> io::Result<Option<PathBuf>>
    where
        T: AsRef<Path>,
        S: AsRef<str>,
    {
        let current_dir = env::current_dir().unwrap_or_else(|_| target.as_ref().to_owned());
        view_dirs(&self, target, extensions, &current_dir)
    }
}

fn view_dirs<'a, T: AsRef<Path>, S: AsRef<str>>(
    ui: &Ui<'a>,
    target: T,
    extensions: &[S],
    default_dir: &Path,
) -> io::Result<Option<PathBuf>> {
    let target = target.as_ref();
    let mut files = Vec::new();

    for direntry in fs::read_dir(target)? {
        match direntry {
            Ok(direntry) => {
                let path = direntry.path();
                if !is_hidden(&path) && (path.is_dir() || has_extension(&path, extensions)) {
                    files.push(path);
                }
            }
            Err(e) => eprintln!("Error on listing files: {:?}", e),
        }
    }

    // Sort folder first
    files.sort_by(|path1, path2| match (path1.is_dir(), path2.is_dir()) {
        (true, true) => path1.cmp(path2),
        (false, false) => path1.cmp(path2),
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
    });

    let mut ret = Ok(None);
    for i in files {
        if i.is_dir() {
            if let Some(dirname) = i.file_name() {
                if let Some(dirname) = dirname.to_str() {
                    let im_dirname = ImString::new(dirname);
                    let open = default_dir.starts_with(&i);
                    let tree_node =
                        imgui::TreeNode::new(&im_dirname).opened(open, imgui::Condition::Once);
                    tree_node.build(ui, || {
                        let out = view_dirs(ui, &i, extensions, default_dir);
                        match ret {
                            // Do not overwrite return value when it is already set
                            Ok(Some(_)) => (),
                            _ => ret = out,
                        }
                    });
                } else {
                    eprintln!("Could not get str out of directory: {:?}", i);
                }
            } else {
                eprintln!("Could not get dirname for path: {:?}", i);
            }
        } else if let Some(file_name) = i.file_name() {
            if let Some(file_name) = file_name.to_str() {
                ui.bullet_text(im_str!(""));
                ui.same_line(0.0);
                if ui.small_button(&ImString::new(file_name)) {
                    ret = Ok(Some(i.clone()));
                }
            } else {
                eprintln!("Could not get str out of file: {:?}", i);
            }
        } else {
            eprintln!("Could not get file_name for path: {:?}", i);
        }
    }
    ret
}
