fn main() {
    println!("cargo:rustc-link-search=native=/users/Kosumi/tree-fuzzer/target/release");
    println!("cargo:rustc-link-lib=static=tree_fuzzer_rust_splicer");
}
