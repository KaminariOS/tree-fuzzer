fn main() {
    println!("cargo:rustc-link-search=native=/home/kosumi/Rusty/tree-fuzzer/target/release");
    println!("cargo:rustc-link-lib=static=tree_fuzzer_json");
}
