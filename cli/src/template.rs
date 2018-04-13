use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::io::Write;
use std::fs::{self, File};

use tera::{self, Tera};
use serde::Serialize;
use url::Url;

#[allow(dead_code)]
mod assets {
    include!(concat!(env!("OUT_DIR"), "/assets.rs"));
}

pub enum DepType {
    Upstream,
    Git(Url),
    Local(PathBuf)
}

impl DepType {
    // Additional templates to apply in the order to apply them.
    fn templates(&self) -> &'static [&'static str] {
        match *self {
            DepType::Upstream => &[],
            DepType::Git(..) => &["master"],
            DepType::Local(..) => &["master"],
        }
    }
}

#[derive(Debug, Error)]
pub enum Error {
    /// The project name is invalid
    InvalidName,
    /// A file or directory with the given name already exists
    ProjectExists,
    /// The template required is not known.
    #[error(no_from, non_std)]
    UnknownTemplate(&'static str),
    /// An I/O error occurred.
    Io(::std::io::Error),
    /// Internal error: template contained invalid UTF-8.
    Utf8(::std::str::Utf8Error),
    /// Internal error: failed to parse an internal template
    Template(tera::Error),
}

fn ignored(entry: &DirEntry) -> bool {
    entry.path()
         .to_str()
         .map(|s| s.ends_with(".ignore"))
         .unwrap_or(false)
}

pub fn is_valid_name(s: &str) -> bool {
    fn name_start(c: char) -> bool {
        (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z')
    }

    fn name_continue(c: char) -> bool {
        (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') ||
            (c >= '0' && c <= '9') || c == '_' || c == '-'
    }

    let mut chars = s.chars();
    if s.is_empty() {
        return false;
    }

    let first = chars.next().unwrap();
    if !name_start(first) {
        return false;
    }

    chars.all(name_continue)
}

fn get_deps(dep: &DepType) -> String {
    let lib_val = match *dep {
        DepType::Upstream => "\"0.3\"".into(),
        DepType::Git(ref repo) => format!(r#"{{ git = "{}" }}"#, repo),
        DepType::Local(ref path) => format!(r#"{{ path = "{}/lib" }}"#, path.display()),
    };

    let codegen_val = match *dep {
        DepType::Upstream => "\"0.3\"".into(),
        DepType::Git(ref repo) => format!(r#"{{ git = "{}" }}"#, repo),
        DepType::Local(ref path) => format!(r#"{{ path = "{}/codegen" }}"#, path.display()),
    };

    format!("rocket = {}\nrocket_codegen = {}", lib_val, codegen_val)
}

use self::assets::{TEMPLATES, DirEntry};

pub fn generate_project(name: &str, dep: DepType) -> Result<(), Error> {
    if !is_valid_name(name) {
        return Err(Error::InvalidName)
    }

    let project_path = Path::new(name);
    if project_path.exists() {
        return Err(Error::ProjectExists)
    }

    // Initialize context for templating. FIXME: Get authors from git.
    let mut context: HashMap<&str, String> = HashMap::new();
    context.insert("name", name.into());
    context.insert("version", "0.0.1".into());
    context.insert("authors", "[\"Sergio Benitez <sb@sergio.bz>\"]".into());
    context.insert("dependencies", get_deps(&dep));

    // Create the project root and apply the base template.
    fs::create_dir(project_path)?;
    apply_template("base", project_path, &context)?;

    for template in dep.templates().iter() {
        apply_template(template, project_path, &context)?;
    }

    // FIXME: Initialize a git repo in the project.
    Ok(())
}

pub fn apply_template<T: Serialize>(
    name: &'static str,
    path: &Path,
    context: &T
) -> Result<(), Error> {
    debug!("Applying template '{}' at '{}'.", name, path.display());

    for subdir in TEMPLATES.subdirs.iter() {
        if subdir.name() != name {
            continue
        }

        // Create the files inside the project.
        for entry in subdir.walk().filter(|e| !ignored(e)) {
            let new_path = path.join(entry.path().strip_prefix(name).unwrap());
            let metadata = fs::metadata(&new_path).ok();

            let is_file = metadata.as_ref().map(|m| m.is_file()).unwrap_or(false);
            if is_file {
                debug!("Removing existing file: {}", new_path.display());
                fs::remove_file(&new_path)?;
            }

            if let DirEntry::Dir(_) = entry {
                let is_dir = metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false);
                if !is_dir {
                    debug!("Creating dir: {}", new_path.display());
                    fs::create_dir(&new_path)?;
                }
            } else if let DirEntry::File(file) = entry {
                debug!("Creating file: {}", new_path.display());
                let template_str = ::std::str::from_utf8(file.contents)?;
                let rendered = Tera::one_off(template_str, &context, false)?;
                let mut file = File::create(&new_path)?;
                file.write_all(rendered.as_bytes())?;
            }
        }

        return Ok(())
    }

    Err(Error::UnknownTemplate(name))
}
