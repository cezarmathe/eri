#[macro_use]
extern crate anyhow;

mod config;
mod data;
mod namespace;
mod template;

use handlebars::Handlebars;

fn main() {
    let eri_config = config::EriConfig::open().unwrap();

    let namespaces: Vec<namespace::Namespace> = eri_config.namespaces().unwrap();

    let mut _handlebars = Handlebars::new();
    for namespace in &namespaces {
        // namespace.gen_data_file(&mut _handlebars).unwrap();
        namespace.render(&mut _handlebars).unwrap();
        // // println!("data: {:?}", namespace);
        // let mut params: Vec<String> = Vec::new();
        // for template in namespace.templates().unwrap() {
        //     template.register(&mut _handlebars).unwrap();
        //     params.append(&mut template.parameter_list(&_handlebars).unwrap());
        //     // template.render(&mut _handlebars).unwrap();
        // }
        // let mut file = std::fs::File::create(namespace.base_path.join("eri.conf")).unwrap();
        // for param in &params {
        //     use std::io::Write;
        //     write!(file, "{} =\n", param).unwrap();
        // }
    }
}
