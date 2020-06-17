#[macro_use]
extern crate anyhow;

mod config;
mod data;
mod namespace;
mod template;

use handlebars::Handlebars;

fn main() {
    let eri_config = config::EriConfig::open().unwrap();
    println!("{:?}", eri_config);

    let mut namespaces: Vec<namespace::Namespace> = Vec::new();
    for (name, data) in eri_config.namespace {
        namespaces.push(namespace::Namespace::new(&name, &eri_config.export, &data).unwrap());
    }
    println!("{:?}", namespaces);

    // let template: String = std::fs::read_to_string("vault.hcl").unwrap();
    // let mut handlebars: Handlebars = Handlebars::new();
    // handlebars.register_template_string("vault.hcl", template).unwrap();
    // println!("\nTemplate: {:?}", handlebars.get_template("vault.hcl").unwrap().elements);

    // println!("\nApplied template:\n{}", handlebars.render("vault.hcl", &eri_config.data.get("vault").unwrap().0).unwrap());
}
