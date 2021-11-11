fn main() {
    let files = ["protos/v1alpha1/api/api.proto"];
    tonic_build::configure()
        .compile(&files, &[".", "protos"])
        .unwrap();
}
