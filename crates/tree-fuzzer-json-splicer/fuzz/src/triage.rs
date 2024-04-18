use std::env;
use std::fs::read;

use test_serde::main_fuzz;

fn main() {
   let args: Vec<String> = env::args().collect(); 
   let data = read(&args[0]).unwrap();
   main_fuzz(&data);
}
