#![no_main]

// use libfuzzer_sys::fuzz_target;
use serde_json::{from_slice, Value};

#[no_mangle]
pub extern "C" fn test_json(data: *const u8, len: usize) {
    let data = unsafe {
        core::slice::from_raw_parts(data, len)
    };
    let v = from_slice::<Value>(data);
    let mut k = [1, 23];
    let p = k.as_mut_ptr();
    unsafe {
        p.add(3).write(5);
    }
    println!("FCUKDFSDFSSD");
    _ = v.map(|js| println!("{}", js.to_string()));
}

// fuzz_target!(|data: &[u8]| {
//     let v = from_slice::<Value>(data);
//     println!("FCUKDFSDFSSD");
//     _ = v.map(|js| println!("{}", js.to_string()));
//     // fuzzed code goes here
// });
