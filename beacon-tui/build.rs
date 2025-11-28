use cargo_metadata::{MetadataCommand, Package};

fn main() {
    let metadata = MetadataCommand::new()
        .exec()
        .expect("failed to read cargo metadata");

    let mut max_len = 0;
    for package in metadata.packages {
        // only consider workspace members
        if metadata.workspace_members.contains(&package.id) && has_tracing_dep(&package) {
            let len = package.name.trim_start_matches("beacon-").len();
            if len > max_len {
                max_len = len;
                println!(
                    "cargo:warning=New max target length: {} (from package {})",
                    max_len, package.name
                );
            }
        }
    }

    println!("cargo:rustc-env=MAX_TARGET_LEN={}", max_len);
}

fn has_tracing_dep(pkg: &Package) -> bool {
    pkg.dependencies.iter().any(|dep| dep.name == "tracing")
}
