#[macro_use] extern crate clap;
#[macro_use] extern crate derive_error;
extern crate serde;
extern crate tera;
extern crate url;
extern crate walkdir;

#[macro_use] mod utils;
mod template;

use std::path::Path;

use clap::{Arg, App, SubCommand, AppSettings, ArgGroup};
use template::{DepType, generate_project};

const NEW_DESCRIPTION: &'static str =
"Generate a new Cargo project with Rocket dependencies.

Without any flags, the generated Cargo project will use the latest version of \
Rocket's libraries as its dependencies. You can use the --git flag to instead \
use a version from a git repository, and --local to instead use a version from \
a local path.";

fn main() {
    let app_matches = App::new("Rocket CLI")
        .version(crate_version!())
        .author("Sergio Benitez <sb@sergio.bz>")
        .about("A command line interface for Rocket.")
        .setting(AppSettings::SubcommandRequired)
        .setting(AppSettings::ColoredHelp)
        .setting(AppSettings::GlobalVersion)
        .subcommand(SubCommand::with_name("new")
                    .about(NEW_DESCRIPTION.lines().next().unwrap())
                    .long_about(NEW_DESCRIPTION)
                    .setting(AppSettings::ArgRequiredElseHelp)
                    .setting(AppSettings::ColoredHelp)
                    .arg(Arg::with_name("name")
                         .help("The name of the new Cargo project")
                         .required(true)
                         .index(1))
                    .arg(Arg::with_name("repo")
                         .long("git")
                         .short("g")
                         .takes_value(true)
                         .default_value("https://github.com/SergioBenitez/Rocket")
                         .help("Use Rocket dependencies from git"))
                    .arg(Arg::with_name("path")
                         .long("local")
                         .short("l")
                         .takes_value(true)
                         .help("Use Rocket dependencies from a local path"))
                    .group(ArgGroup::with_name("dep")
                           .args(&["repo", "path"])))
        .get_matches();

    if let Some(matches) = app_matches.subcommand_matches("new") {
        let name = matches.value_of("name").unwrap();
        let dep_type = if let Some(url) = matches.value_of("repo") {
            DepType::Git(url.parse().expect("invalid git URL"))
        } else if let Some(raw_path) = matches.value_of("path") {
            let path = match Path::new(raw_path).canonicalize() {
                Ok(path) => path,
                Err(e) => panic!("Local path {:?} is invalid: {}.", raw_path, e)
            };

            DepType::Local(path)
        } else {
            DepType::Upstream
        };

        if let Err(e) = generate_project(name, dep_type) {
            use std::error::Error;
            panic!("error: {} {}", e, e.description());
        }

        println!("Rocket project '{}' created.", name);
    }
}
