// #![feature(linkage)]

use serde_json::{from_slice, ser, Value};
use tree_fuzzer::fuzz_target;

// #[no_mangle]
// pub extern "C" fn LLVMFuzzerTestOneInput(data: *const u8, len: usize) 

fn recursive_walk(v: &Value) {
    if v.is_i64() {
        eprintln!("{:?}", v.as_i64());
    }
    else if v.is_u64() {
        eprintln!("{:?}", v.as_u64());
    }

    else if v.is_object() {
        let obj = v.as_object().unwrap();
        for (_, v) in obj {
            recursive_walk(v);
        }
    }

    else if v.is_array() {
        let arr = v.as_array().unwrap();
        for v in arr {
            recursive_walk(v);
        }
    } else if v.is_null() {
        let _ = v.as_null();
    } else if v.is_number() {
        
    } else if v.is_string() {
        let st = v.as_str().unwrap();
        eprintln!("{st}");
    }
}

fuzz_target!(|data| {
    let v = from_slice::<Value>(data);
    // let mut k = [1, 23];
    // let p = k.as_mut_ptr();
    // unsafe {
    //     // p.add(3).write(5);
    // }
    // println!("FCUKDFSDFSSD");

    _ = v.map(|js| {
        recursive_walk(&js);
        format!("{}", js.to_string())
    });
});



#[link(name = "tree_fuzzer_json_splicer")]
extern "C" {
    fn libafl_main();
}

