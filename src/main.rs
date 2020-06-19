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

fn main() {
    let mut app: App = App::new("eri")
        .version("0.1")
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
        _ => LevelFilter::Trace,
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
        .chain(std::io::stdout())
        .apply()
        .unwrap();
    // log::trace!("initialized logging");

    // log::debug!("loading eri configuration");
    let eri_config = match config::EriConfig::open() {
        Ok(value) => {
            // log::trace!("loaded eri configuration: {:#?}", value);
            value
        }
        Err(e) => {
            // log::error!("failed to load eri configuration: {:#?}", e);
            std::process::exit(1);
        }
    };

    // log::debug!("loading namespaces");
    let namespaces: Vec<namespace::Namespace> = match eri_config.namespaces() {
        Ok(value) => {
            // log::trace!("loaded namespaces: {:#?}", value);
            value
        }
        Err(e) => {
            // log::error!("failed to load namespaces: {:#?}", e);
            std::process::exit(1);
        }
    };

    // log::debug!("creating the handlebars template engine");
    let mut handlebars = Handlebars::new();
    // log::trace!("created the handlebars template engine: {:#?}", handlebars);

    // log::trace!("checking subcommand");
    if let Some(_) = matches.subcommand_matches("render") {
        // log::info!("rendering configuration files");
        let before = Local::now();
        for namespace in namespaces {
            namespace.render(&mut handlebars).unwrap();
        }
        let after = Local::now();
        let duration: Duration = after - before;
        // log::info!(
        //     "rendering took {:#?} ms",
        //     duration.num_nanoseconds().unwrap() as f64 / 1000000.0
        // );
    } else if let Some(_) = matches.subcommand_matches("gendata") {
        // log::info!("generating data files for each namespace");
        for namespace in namespaces {
            namespace.gen_data_file(&mut handlebars).unwrap();
        }
    } else {
        // log::trace!("no subcommand, printing the help section");
        app.print_help().unwrap();
    }
}
