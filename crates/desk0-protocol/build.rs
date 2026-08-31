fn main() -> Result<(), Box<dyn std::error::Error>> {
  if std::env::var("PROTOC").is_err()
    && std::process::Command::new("protoc")
      .arg("--version")
      .output()
      .is_err()
  {
    let protoc_path = protobuf_src::protoc();
    unsafe {
      std::env::set_var("PROTOC", protoc_path);
    }
  }
  tonic_build::compile_protos("proto/desk0/desk0.proto")?;
  Ok(())
}
