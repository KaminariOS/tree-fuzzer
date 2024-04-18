// #![feature(linkage)]

use serde_json::{from_slice, Value};
use tree_fuzzer::fuzz_target;

// #[no_mangle]
// pub extern "C" fn LLVMFuzzerTestOneInput(data: *const u8, len: usize) 
//
//
fn show_val(val: &Value) {
    match val {
        Value::Null => {
            println!("NULL");
        },
        Value::Bool(b) => {
            println!("bool {b}");
        },
        Value::Array(ve) => {
            ve.iter().for_each(|e| show_val(e));
        },
        Value::Number(n) => {

            println!("Number {n}");
        },
        Value::String(s) => {

            println!("String {s}");
        },
        Value::Object(m) => {
            m.iter().for_each(|(k, v)| {
                print!("Key: {k} Val: ");
                show_val(v);
            })
        },
        
    }
}

fuzz_target!(|data| {
    let v = from_slice::<Value>(data);
        // p.add(3).write(5);
    // println!("FCUKDFSDFSSD");
    if let Ok(val) = v {

        println!("{}", val.to_string())
    } else {
        println!("INVALID===================================");
    }
});



#[link(name = "tree_fuzzer_json_nautilus")]
extern "C" {
    fn libafl_main();
}

