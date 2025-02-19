use std::fs::File;
use std::path::PathBuf;
use std::{env, fs};

use anyhow::Context;
use clap::Parser;
use oci_spec::image::{self as spec, Arch};
use oci_tar_builder::Builder;

pub const WASM_LAYER_MEDIA_TYPE: &str =
    "application/vnd.bytecodealliance.wasm.component.layer.v0+wasm";

pub fn main() {
    let args = Args::parse();

    let out_dir;
    if let Some(out_path) = args.out_path.as_deref() {
        out_dir = PathBuf::from(out_path);
        fs::create_dir_all(out_dir.parent().unwrap()).unwrap();
    } else {
        out_dir = env::current_dir().unwrap();
    }

    let entry_point = args.name.clone() + ".wasm";

    let mut builder = Builder::default();
    for module_path in args.module.iter() {
        let module_path = PathBuf::from(module_path);
        builder.add_layer_with_media_type(&module_path, WASM_LAYER_MEDIA_TYPE.to_string());
    }

    for layer_config in args.layer.iter() {
        //split string on equals sign
        let layer_options: Vec<&str> = layer_config.split('=').collect();

        let layer_type = layer_options.first().unwrap();
        let layer_path = PathBuf::from(layer_options.last().unwrap());
        builder.add_layer_with_media_type(&layer_path, layer_type.to_string());
    }

    if let Some(components_path) = args.components.as_deref() {
        let paths = fs::read_dir(components_path).unwrap();

        for path in paths {
            let path = path.unwrap().path();
            let ext = path.extension().unwrap().to_str().unwrap();
            match ext {
                "wasm" => {
                    builder.add_layer_with_media_type(&path, WASM_LAYER_MEDIA_TYPE.to_string());
                }
                _ => println!(
                    "Skipping Unknown file type: {:?} with extension {:?}",
                    path,
                    path.extension().unwrap()
                ),
            }
        }
    }

    let config = spec::ConfigBuilder::default()
        .entrypoint(vec![entry_point])
        .build()
        .unwrap();

    let img = spec::ImageConfigurationBuilder::default()
        .config(config)
        .os("wasip1")
        .architecture(Arch::Wasm)
        .rootfs(
            spec::RootFsBuilder::default()
                .diff_ids(vec![])
                .build()
                .unwrap(),
        )
        .build()
        .context("failed to build image configuration")
        .unwrap();

    builder.add_config(img, args.repo + "/" + &args.name + ":" + &args.tag);

    println!("Creating oci tar file {}", out_dir.clone().display());
    let f = File::create(out_dir.clone()).unwrap();
    match builder.build(f) {
        Ok(_) => println!("Successfully created oci tar file {}", out_dir.display()),
        Err(e) => {
            print!(
                "Building oci tar file {} failed: {:?}",
                out_dir.display(),
                e
            );
            fs::remove_file(out_dir).unwrap_or(print!("Failed to remove temporary file"));
        }
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    out_path: Option<String>,

    #[arg(short, long)]
    name: String,

    #[arg(short, long)]
    tag: String,

    #[arg(short, long)]
    repo: String,

    #[arg(short, long)]
    module: Vec<String>,

    #[arg(short, long)]
    layer: Vec<String>,

    #[arg(short, long)]
    components: Option<String>,
}
