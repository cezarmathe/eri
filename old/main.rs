mod data;

use uclicious::*;

fn main() {
    let string = std::fs::read_to_string("eri.conf").unwrap();
    let mut eri_config_builder = data::EriConfig::builder().unwrap();

    eri_config_builder
        .add_chunk_full(string, Priority::default(), DEFAULT_DUPLICATE_STRATEGY)
        .unwrap();
    let eri_config = eri_config_builder.build().unwrap();
    println!("{:?}", eri_config);

    for include in eri_config.include.unwrap() {
        let string = std::fs::read_to_string(format!("{}/eri.conf", include)).unwrap();
        let mut eri_config_builder = data::EriConfig::builder().unwrap();

        eri_config_builder
            .add_chunk_full(string, Priority::default(), DEFAULT_DUPLICATE_STRATEGY)
            .unwrap();
        let eri_config = eri_config_builder.build().unwrap();
        println!("{:?}", eri_config);
    }
}
