extern crate clap;
extern crate hyper;
extern crate rustc_serialize;
extern crate url;
extern crate yaml_rust;

mod client;
mod label;

use std::fs::File;
use std::io::Error as IoError;
use std::io::Read;
use std::process::exit;

use clap::{App, AppSettings, Arg};
use yaml_rust::{Yaml, YamlLoader};

use client::Client;
use label::Label;
use label::Error as LabelError;

fn main() {
    let matches = App::new("ghlabel")
        .setting(AppSettings::ArgRequiredElseHelp)
        .version(env!("CARGO_PKG_VERSION"))
        .version_short("v")
        .about("Automatically creates and deletes labels on GitHub Issues to match a template")
        .after_help("\
Example:

    ghlabel --file labels.yml --token abc123 --user rust-lang --repo rust

The file must contain an array of hashes, each with a name and a color. For
example, here is a template for a subset of the default GitHub Issues labels:

    - name: bug
      color: fc2929
    - name: duplicate
      color: cccccc
    - name: enhancement
      color: 84b6eb

By default, every label in the file will be created (or updated, if the color
changed) on GitHub if it doesn't already exist and every label on GitHub not in
the file will be deleted. Limit this behavior with the --no-create and
--no-delete flags, respectively. No output from the program indicates there
were no changes made.

An OAuth token can be obtained from https://github.com/settings/tokens.
The token used requires the \"repo\" scope if the program will be run on a
private repo. Otherwise, it only requires the \"public_repo\" scope.

"
        )
        .arg(
            Arg::with_name("file")
                .help("Path to a YAML file containing the label template")
                .long("file")
                .short("f")
                .takes_value(true)
                .required(true)
                .empty_values(false)
        )
        .arg(
            Arg::with_name("token")
                .help("OAuth token for authenticating with GitHub")
                .long("token")
                .short("t")
                .takes_value(true)
                .required(true)
                .empty_values(false)
        )
        .arg(
            Arg::with_name("user")
                .help("The name of the user or organization that owns the repository")
                .long("user")
                .short("u")
                .takes_value(true)
                .required(true)
                .empty_values(false)
         )
        .arg(
            Arg::with_name("repo")
                .help("The name of the repository to apply the label template to")
                .long("repo")
                .short("r")
                .takes_value(true)
                .required(true)
                .empty_values(false)
         )
        .arg(
          Arg::with_name("endpoint")
                .help("API endpoint to use (defaults to https://api.github.com)")
                .long("endpoint")
                .short("e")
                .takes_value(true)
                .required(false)
                .empty_values(false)
        )
        .arg(
            Arg::with_name("dry-run")
                .help("Print what the program would do without actually doing it")
                .long("dry-run")
                .short("d")
        )
        .arg(
            Arg::with_name("no-create")
                .help("Do not create labels missing from the repo but present in the file")
                .long("no-create")
        )
        .arg(
            Arg::with_name("no-delete")
                .help("Do not delete labels in the repo that are not in the file")
                .long("no-delete")
        )
        .get_matches();

    let path = matches.value_of("file").unwrap();
    let token = matches.value_of("token").unwrap();
    let user = matches.value_of("user").unwrap();
    let repo = matches.value_of("repo").unwrap();
    let endpoint = matches.value_of("endpoint").unwrap_or("https://api.github.com");

    let dry_run = matches.is_present("dry-run");
    let should_create = !matches.is_present("no-create");
    let should_delete = !matches.is_present("no-delete");

    let file_contents = match read_file(path) {
        Ok(contents) => contents,
        Err(error) => {
            println!("Failed to read labels.yml: {}", error);
            exit(1);
        }
    };

    let yaml = match YamlLoader::load_from_str(&file_contents) {
        Ok(yaml) => yaml,
        Err(error) => {
            println!("Failed to parse YAML data: {}", error);
            exit(1);
        }
    };

    if yaml.is_empty() {
        println!("Expected labels.yml to have some data");
        exit(1);
    }

    let template = match yaml[0].as_vec() {
       Some(template) => template,
       None => {
           println!("Expect contents of labels.yml to be a single array");
           exit(1);
       }
    };

    let labels = match get_labels(&template, user, repo) {
       Ok(labels) => labels,
       Err(_) => {
           println!("Invalid label! Each label must be a hash with the keys `name` and `color`");
           exit(1);
       }
    };

    let client = Client::new(&repo, &token, &user, &endpoint);

    let existing_labels = match client.list() {
        Ok(existing_labels) => {
            existing_labels
        },
        Err(error) => {
            println!("Error getting existing labels from the GitHub API: {:?}", error);
            exit(1);
        },
    };

    if should_create {
        for label in &labels {
            if existing_labels.contains(label) {
                let existing_label = existing_labels.iter().find(|&existing_label| {
                    existing_label.name == label.name
                }).unwrap();

                if label.color != existing_label.color {
                    if dry_run {
                        println!("[DRY RUN] UPDATE {}: {}", label.name, label.color);
                    } else {
                        match client.update(&label) {
                            Ok(_) => println!("UPDATE {}: {}", label.name, label.color),
                            Err(error) => println!("FAILURE {:?}", error),
                        }
                    }
                }
            } else {
                if dry_run {
                    println!("[DRY RUN] CREATE {}: {}", label.name, label.color);
                } else {
                    match client.create(&label) {
                        Ok(_) => println!("CREATE {}: {}", label.name, label.color),
                        Err(error) => println!("FAILURE {:?}", error),
                    }
                }
            }
        }
    }

    if should_delete {
        for existing_label in &existing_labels {
            if !labels.contains(existing_label) {
                if dry_run {
                    println!("[DRY RUN] DELETE {}", existing_label.name);
                } else {
                    match client.delete(existing_label) {
                        Ok(_) => println!("DELETE {}", existing_label.name),
                        Err(error) => println!("FAILURE {:?}", error),
                    }
                }
            }
        }
    }
}

fn read_file<'a>(path: &'a str) -> Result<String, IoError> {
    let mut f = try!(File::open(path));
    let mut s = String::new();
    try!(f.read_to_string(&mut s));
    Ok(s)
}

fn get_labels<'a>(
    template: &'a Vec<Yaml>,
    user: &'a str,
    repo: &'a str,
) -> Result<Vec<Label>, LabelError> {
    let mut labels = vec![];

    for item in template.iter() {
       let (name, color) = try!(get_name_and_color(item));
       let label = try!(Label::new(name, color, user, repo));
       labels.push(label);
    }

    Ok(labels)
}

pub fn get_name_and_color<'a>(yaml: &'a Yaml) -> Result<(&'a str, &'a str), LabelError> {
    match yaml.as_hash() {
        Some(hash) => {
            let name = match hash[&Yaml::from_str("name")].as_str() {
                Some(name) => name,
                None => return Err(LabelError::MissingName)
            };

            let color = match hash[&Yaml::from_str("color")].as_str() {
                Some(color) => color,
                None => return Err(LabelError::MissingColor)
            };

            Ok((name, color))
        },
        None => Err(LabelError::YamlItemNotHash),
    }
}
