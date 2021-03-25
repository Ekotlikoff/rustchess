fn main() -> Result<(), Box<dyn std::error::Error>> {
    //tonic_build::configure()
    //    .build_server(true)
    //    .out_dir("proto/")
    //    .compile(&["proto/rustchess.proto"], &["proto/"])
    //    .unwrap();
    tonic_build::compile_protos("proto/rustchess.proto")?;
    Ok(())
}
