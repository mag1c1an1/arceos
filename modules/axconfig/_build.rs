use std::io::{Result, Write};
use std::{convert::AsRef, fs, path::Path};
use toml_edit::{Decor, Document, Item, Table, Value};

fn main() {
    // generate config_*.rs for all platforms
    for fname in fs::read_dir("src/platform").unwrap() {
        let fname = fname.unwrap().path();
        if fname.extension().unwrap() == "toml" {
            let platform = fname.file_stem().unwrap().to_str().unwrap();
            gen_config_rs(platform).unwrap();
            println!("cargo:rerun-if-changed={}", fname.display());
        }
    }
    println!("cargo:rerun-if-changed=src/defconfig.toml");
    // println!("cargo:rerun-if-env-changed=SMP");
}

fn add_config(config: &mut Table, key: &str, item: Item, comments: Option<&str>) {
    config.insert(key, item);
    if let Some(comm) = comments {
        if let Some(dst) = config.key_decor_mut(key) {
            *dst = Decor::new(comm, "");
        }
    }
}

fn parse_config_toml(result: &mut Table, path: impl AsRef<Path>) -> Result<()> {
    println!("Reading config file: {}", path.as_ref().display());
    let config_content = std::fs::read_to_string(path)?;
    let config = config_content
        .parse::<Document>()
        .expect("failed to parse config file");
    for (key, item) in config.iter() {
        add_config(
            result,
            key,
            item.clone(),
            config
                .key_decor(key)
                .and_then(|d| d.prefix())
                .and_then(|s| s.as_str()),
        );
    }
    Ok(())
}

fn is_num(s: &str) -> bool {
    let s = s.replace('_', "");
    if s.parse::<usize>().is_ok() {
        true
    } else if let Some(s) = s.strip_prefix("0x") {
        usize::from_str_radix(s, 16).is_ok()
    } else {
        false
    }
}

fn gen_config_rs(platform: &str) -> Result<()> {
    // Load TOML config file
    let mut config = Table::new();
    parse_config_toml(&mut config, "src/defconfig.toml").unwrap();
    parse_config_toml(&mut config, format!("src/platform/{platform}.toml")).unwrap();

    add_config(
        &mut config,
        "smp",
        toml_edit::value(std::env::var("SMP").unwrap_or("1".into())),
        Some("# Number of CPUs"),
    );

    // println!("{config:#x?}");

    // Generate config.rs
    let mut output = Vec::new();
    writeln!(
        output,
        "//! Platform constants and parameters for {platform}."
    )?;
    writeln!(output, "//! Generated by build.rs, DO NOT edit!\n")?;

    for (key, item) in config.iter() {
        let var_name = key.to_uppercase().replace('-', "_");
        if let Item::Value(value) = item {
            let comments = config
                .key_decor(key)
                .and_then(|d| d.prefix())
                .and_then(|s| s.as_str())
                .unwrap_or("")
                .trim()
                .replace('#', "///");
            match value {
                Value::String(s) => {
                    writeln!(output, "{comments}")?;
                    let s = s.value();
                    if is_num(s) {
                        writeln!(output, "pub const {var_name}: usize = {s};")?;
                    } else {
                        writeln!(output, "pub const {var_name}: &str = \"{s}\";")?;
                    }
                }
                Value::Array(regions) => {
                    if key != "mmio-regions" && key != "virtio-mmio-regions" && key != "pci-ranges"
                    {
                        continue;
                    }
                    writeln!(output, "{comments}")?;
                    writeln!(output, "pub const {var_name}: &[(usize, usize)] = &[")?;
                    for r in regions.iter() {
                        let r = r.as_array().unwrap();
                        writeln!(
                            output,
                            "    ({}, {}),",
                            r.get(0).unwrap().as_str().unwrap(),
                            r.get(1).unwrap().as_str().unwrap()
                        )?;
                    }
                    writeln!(output, "];")?;
                }
                _ => {}
            }
        }
    }

    let out_path = format!("src/config_{}.rs", platform.replace('-', "_"));
    fs::write(out_path, output)?;

    Ok(())
}
