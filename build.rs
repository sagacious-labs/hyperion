fn main() {
    let files = ["protos/v1alpha1/api/api.proto"];

    println!("Compiling {:?}...", files);
    tonic_build::configure().compile(&files, &["."]).unwrap();
    println!("Completed");

    // println!("cargo:rerun-if-changed=protos");
}
