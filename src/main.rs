use clap::Parser;
use colored::ColoredString;
use colored::Colorize;
use colored::CustomColor;
use directories::ProjectDirs;
use rand::RngExt;
use serde::Deserialize;
use std::path::PathBuf;
// use std::any::type_name_of_val;
use std::fs;
use terminal_size::{Width, terminal_size};
use toml;

//  ____      ____                _           _
// |  _ \ ___|  _ \ ___ _ __ ___ (_)_ __   __| | ___ _ __
// | |_) / __| |_) / _ \ '_ ` _ \| | '_ \ / _` |/ _ \ '__|
// |  _ <\__ \  _ <  __/ | | | | | | | | | (_| |  __/ |
// |_| \_\___/_| \_\___|_| |_| |_|_|_| |_|\__,_|\___|_|
//

#[derive(Debug, Deserialize)]
#[serde(default)]
struct ConfigToml {
    config: Config,
    // data: Data,
    passage: Vec<Passage>,
}
impl Default for ConfigToml {
    fn default() -> Self {
        Self {
            config: Config::default(),
            // data: Data::default(),
            passage: [].to_vec(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
struct Config {
    color: Option<ConfigColor>,
    align: Align,
    text_width: usize,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            color: None,
            align: Align::Center,
            text_width: 50,
        }
    }
}

#[derive(Deserialize, Debug, Default, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Align {
    Left,
    MidLeft,
    #[default]
    Center,
    MidRight,
    Right,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
enum ConfigColor {
    Hex(String),
    Rgb([u8; 3]),
}
fn convert_hex_to_rgb(hex: String) -> Option<[u8; 3]> {
    let trimmed = hex.trim().strip_prefix('#').unwrap_or(hex.trim());

    let expanded = match trimmed.len() {
        3 => {
            let mut buf = String::with_capacity(6);
            for c in trimmed.chars() {
                buf.push(c);
                buf.push(c);
            }
            buf
        }
        6 => trimmed.to_string(),
        _ => return None,
    };

    let r = u8::from_str_radix(&expanded[0..2], 16).ok()?;
    let g = u8::from_str_radix(&expanded[2..4], 16).ok()?;
    let b = u8::from_str_radix(&expanded[4..6], 16).ok()?;

    Some([r, g, b])
}

#[derive(Clone, Debug, Deserialize)]
struct Passage {
    verse: String,
    location: Option<String>,
}
impl Passage {
    fn new(v: String, l: Option<String>) -> Self {
        Self {
            verse: v,
            location: l,
        }
    }
}

#[derive(Debug, Parser)]
struct Args {
    #[arg(short, long)]
    version: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proj_dirs = ProjectDirs::from("com", "wendel-w", "rsreminder")
        .ok_or("Could not determine system config directory")?;

    let config_path = proj_dirs.config_dir().join("config.toml");
    // let config_path = "config.toml";

    let config_content = fs::read_to_string(config_path)?;
    let config: ConfigToml = toml::from_str(&config_content)?;

    let args = Args::parse();

    if args.version {
        println!("Version: {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    // Picking random verse
    let random_passage: Passage;
    let passages = config.passage.clone();
    let nr_passages = passages.len();
    println!("{}", nr_passages);
    if nr_passages <= 0 {
        println!("No passages found in config!");
        return Ok(());
    }
    let mut rng = rand::rng();
    let random_index = rng.random_range(0..nr_passages);
    random_passage = Passage::new(
        passages[random_index].verse.clone(),
        passages[random_index].location.clone(),
    );

    print_verse(random_passage, config.config);
    Ok(())
}

fn set_up_color(rcolor: Option<ConfigColor>) -> Option<CustomColor> {
    let mut color: Option<CustomColor> = None;
    match rcolor {
        Some(rcolor) => match rcolor {
            ConfigColor::Rgb(v) => {
                color = Some(CustomColor::new(v[0], v[1], v[2]));
            }
            ConfigColor::Hex(h) => {
                let hex = convert_hex_to_rgb(h);
                match hex {
                    Some(v) => {
                        color = Some(CustomColor::new(v[0], v[1], v[2]));
                    }
                    None => {
                        color = None;
                    }
                }
            }
        },
        None => {}
    }
    color
}

fn set_up_margin(align: Align, config_text_width: usize) -> (usize, String) {
    //set up margin according to align
    let term_width = terminal_size()
        .map(|(Width(w), _)| w as usize)
        .unwrap_or(80);

    let text_width = (term_width * 90 / 100).min(config_text_width);
    let margin_length;
    match align {
        Align::Left => margin_length = 0,
        Align::MidLeft => margin_length = (term_width - text_width) / 3,
        Align::Center => margin_length = (term_width - text_width) / 2,

        Align::MidRight => margin_length = (term_width - text_width) / 3 * 2,
        Align::Right => margin_length = term_width - text_width,
    }
    let margin = " ".repeat(margin_length);
    return (text_width, margin);
}

fn format_optional_color(text: String, color: Option<CustomColor>) -> ColoredString {
    match color {
        Some(c) => {
            return text.custom_color(c);
        }
        None => {
            return text.normal();
        }
    }
}

fn print_verse(passage: Passage, config: Config) {
    let (text_width, margin) = set_up_margin(config.align, config.text_width);

    // Setting up color
    let rcolor = set_up_color(config.color.clone());

    println!();

    for line in textwrap::wrap(&passage.verse, text_width) {
        println!(
            "{margin}{}",
            format_optional_color(line.to_string(), rcolor)
        );
    }
    match passage.location {
        Some(l) => {
            println!("{margin}{0:>text_width$}", format_optional_color(l, rcolor));
        }
        None => {}
    }

    println!();
}
