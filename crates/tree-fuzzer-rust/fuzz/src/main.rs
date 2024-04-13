// #![feature(linkage)]

use tree_fuzzer::fuzz_target;

// #[no_mangle]
// pub extern "C" fn LLVMFuzzerTestOneInput(data: *const u8, len: usize) 

fuzz_target!(|data| {
    // if data.len() > 200 {
    //     panic!("Fuck");
    // }
    // let mut k = [1, 23];
    // let p = k.as_mut_ptr();
    unsafe {
        // p.add(3).write(5);
    }
    // println!("FCUKDFSDFSSD");
});



#[link(name = "tree_fuzzer_json")]
extern "C" {
    fn libafl_main();
}

