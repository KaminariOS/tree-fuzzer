// #![feature(linkage)]

use test_serde::main_fuzz;
use tree_fuzzer::fuzz_target;

// #[no_mangle]
// pub extern "C" fn LLVMFuzzerTestOneInput(data: *const u8, len: usize) 


fuzz_target!(|data| {
    main_fuzz(data);
});



#[link(name = "tree_fuzzer_json_splicer")]
extern "C" {
    fn libafl_main();
}

