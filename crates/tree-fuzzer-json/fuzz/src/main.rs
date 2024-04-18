// #![feature(linkage)]

use serde_json::{from_slice, Value};
use tree_fuzzer::fuzz_target;

// #[no_mangle]
// pub extern "C" fn LLVMFuzzerTestOneInput(data: *const u8, len: usize) 

fuzz_target!(|data| {
    // if data.len() > 200 {
    //     panic!("Fuck");
    // }
    let v = from_slice::<Value>(data);
    // let mut k = [1, 23];
    // let p = k.as_mut_ptr();
    unsafe {
        // p.add(3).write(5);
    }
    // println!("FCUKDFSDFSSD");
    _ = v.map(|js| {
        if js.is_array() {
            let arr = js.as_array().unwrap();
            
        } 
        if js.is_object() {
            let obj = js.as_object().unwrap();
        }
        format!("{}", js.to_string());
    });
});



#[link(name = "tree_fuzzer_json")]
extern "C" {
    fn libafl_main();
}

