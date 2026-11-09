use anchor_interface_syn::{Generator, GeneratorOptions};
use texture_common_syn::utils::write_stream_to_file;

pub fn main() {
    const IDL: &str = "wormhole-core-bridge-solana-interface.json";
    const OUT: &str = "src/generated.rs";
    let generator = {
        let opts = GeneratorOptions::builder().idl(IDL).build();
        Generator::from(&opts)
    };
    write_stream_to_file(generator.gen_program_stream(), OUT).expect("write_stream_to_file");
    println!("cargo:rerun-if-changed={}", IDL);
    println!("cargo:rerun-if-changed={}", OUT);
}
