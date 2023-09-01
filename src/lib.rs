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
        icon: ROption::RNone,
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
