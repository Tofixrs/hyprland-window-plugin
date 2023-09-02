use abi_stable::std_types::{ROption, RString, RVec};
use anyrun_plugin::*;
use anyrun_interface::{self, HandleResult, Match, PluginInfo};
use anyrun_macros::{get_matches, handler, info, init};
use fuzzy_matcher::FuzzyMatcher;
use hyprland::data::{Clients};
use hyprland::shared::HyprData;
use hyprland::dispatch::*;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
pub struct Config {
    max_entries: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            max_entries: 5
        }
    }
}


#[init]
fn init(config_dir: RString) -> Config {
    match fs::read_to_string(format!("{}/applications.ron", config_dir)) {
        Ok(content) => ron::from_str(&content).unwrap_or_else(|why| {
            eprintln!("Error parsing applications plugin config: {}", why);
            Config::default()
        }),
        Err(why) => {
            eprintln!("Error reading applications plugin config: {}", why);
            Config::default()
        }
    }
}

#[info]
fn info() -> PluginInfo {
    PluginInfo {
        name: "Windows".into(),
        icon: "window-symbolic".into()
    }
}

#[get_matches]
fn get_matches(input: RString, config: &Config) -> RVec<Match> {
    let matcher = fuzzy_matcher::skim::SkimMatcherV2::default().smart_case();
    let clients = Clients::get().expect("failed to get clients");
    

    let mut clients = clients.iter().filter_map(|client| {
        let score = matcher.fuzzy_match(&client.class, &input).unwrap_or(0);

        if score > 0 {
            Some((client, score))
        } else {
            None
        }
    }).collect::<Vec<_>>();

    clients.sort_by(|a, b| b.1.cmp(&a.1));
    clients.truncate(config.max_entries);
    clients.into_iter().map(|(client, _)| Match {
        title: client.class.clone().into(),  
        icon: get_icon_name(&client.class).into(),
        description: ROption::RNone,
        id: ROption::RNone,
        use_pango: false,
    }).collect()

}

#[handler]
fn handler(selection: Match) -> HandleResult {
    let window = WindowIdentifier::ClassRegularExpression(&selection.title);
    Dispatch::call(DispatchType::FocusWindow(window)).expect("failed to focus window");
    
    HandleResult::Close
}

fn get_desktop_file_path(class: &str) -> Option<PathBuf> {
    let data_dirs = glib::system_data_dirs();
    for dir in data_dirs.iter() {
        let desktop_file_path = dir.join(format!("applications/{class}.desktop"));
        if desktop_file_path.exists() {
            return Some(desktop_file_path)
        }
    }

    None
}

fn get_icon_name(class: &str) -> Option<RString> {
    let desktop_file_path = get_desktop_file_path(class)?;
    let desktop_file = freedesktop_entry_parser::parse_entry(desktop_file_path).ok()?;
    return desktop_file.section("Desktop Entry").attr("Icon").map(|s| s.to_string().into());
}
