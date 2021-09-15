fn main() {
	let file = "protos/ops.proto";

	println!("Compiling {}...", file);
    tonic_build::compile_protos(file)
        .unwrap_or_else(|e| panic!("Failed to compile proto {:?}", e));
	println!("Completed");
	
	println!("cargo:rerun-if-changed={}", file);
}
