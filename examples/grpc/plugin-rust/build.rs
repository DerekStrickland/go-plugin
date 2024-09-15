use std::io::Result;

fn main() -> Result<()> {
    tonic_build::configure()
        .build_server(true)
        .out_dir("src/proto")
        .compile_well_known_types(true)
        .include_file("mod.rs")
        .type_attribute(".", "#[derive(serde::Deserialize)]")
        .compile(
            &[
                "../proto/kv.proto",
                "../../../internal/plugin/grpc_stdio.proto",
            ],
            &["../proto", "../../../internal/plugin"],
        )
        .unwrap();

    Ok(())
}
