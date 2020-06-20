#[macro_use]
extern crate anyhow;

mod config;
mod data;
mod namespace;
mod template;

use chrono::Duration;
use chrono::Local;

use clap::App;
use clap::Arg;
use clap::SubCommand;

use colored::*;

use log::Level;
use log::LevelFilter;

use handlebars::Handlebars;

/// The version of eri
const ERI_VERSION: &str = "0.0.0";

fn main() {
    human_panic::setup_panic!();
    let mut app: App = App::new("eri")
        .version(ERI_VERSION)
        .author("Armand Cezar Mathe <me@cezarmathe.com>")
        .about("Configuration templating for regular people.")
        .arg(
            Arg::with_name("verbosity")
                .short("v")
                .multiple(true)
                .help("Set the verbosity level of the messages outputed by eri. (-v for debug level, -vv for trace level)"),
        )
        .subcommand(
            SubCommand::with_name("render").about("Render the templates specified by eri.conf."),
        )
        .subcommand(
            SubCommand::with_name("gendata")
                .about("Generate the data files requires by each namespace."),
        );
    let matches = app.clone().get_matches();

    let log_level: LevelFilter = match matches.occurrences_of("verbosity") {
        0 => LevelFilter::Info,
        1 => LevelFilter::Debug,
        2 => LevelFilter::Trace,
        _ => {
            println!("The source code is available at https://github.com/cezarmathe/eri.");
            std::process::exit(0);
        }
    };
    fern::Dispatch::new()
        .format(|out, message, record| {
            let prefix: String = match record.level() {
                Level::Error => "ERROR >".red().bold().to_string(),
                Level::Warn => "WARN  >".yellow().bold().to_string(),
                Level::Info => "INFO  >".blue().bold().to_string(),
                Level::Debug => "DEBUG >".cyan().bold().to_string(),
                Level::Trace => "TRACE >".purple().bold().to_string(),
            };
            out.finish(format_args!("{} {}", prefix, message));
        })
        .level(log_level)
        .level_for("users", LevelFilter::Info)
        .chain(std::io::stdout())
        .apply()
        .unwrap();

    let eri_config = match config::EriConfig::open() {
        Ok(value) => value,
        Err(e) => {
            log::error!("Failed to open the eri configuration: {:#?}", e);
            std::process::exit(1);
        }
    };

    let namespaces: Vec<namespace::Namespace> = match eri_config.namespaces() {
        Ok(value) => value,
        Err(e) => {
            log::error!("Failed to load the namespaces: {:#?}", e);
            std::process::exit(1);
        }
    };

    let mut handlebars = Handlebars::new();

    if matches.subcommand_matches("render").is_some() {
        let before = Local::now();
        for namespace in namespaces {
            if let Err(e) = namespace.render(&mut handlebars) {
                log::error!("Failed to render namespace {:#?}: {:#?}", namespace, e);
            }
        }
        let duration: Duration = Local::now() - before;
        if duration.num_seconds() > 0 {
            log::info!(
                "Rendering took {} seconds.",
                duration.num_milliseconds() as f64 / 1000.0
            );
        } else {
            log::info!(
                "Rendering took {} milliseconds.",
                duration.num_microseconds().unwrap() as f64 / 1000.0
            )
        }
    } else if matches.subcommand_matches("gendata").is_some() {
        for namespace in namespaces {
            if let Err(e) = namespace.gen_data_file(&mut handlebars) {
                log::error!(
                    "Failed to generate the data file for the namespace {:#?}: {:#?}",
                    namespace,
                    e
                );
            }
        }
    } else {
        app.print_help().unwrap();
    }
}
