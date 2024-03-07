// use tree_sitter::Language;
//
// #[derive(Clone, Debug, clap::Parser)]
// #[command(author, version, about, long_about = None)]
// pub struct Args {
//     
// }

// pub fn main(language: Language, node_types_json_str: &'static str) {
//     println!("Hello, world!");
// }

/// Define a fuzz target.
///
/// ## Example
///
/// This example takes a `&[u8]` slice and attempts to parse it. The parsing
/// might fail and return an `Err`, but it shouldn't ever panic or segfault.
///
/// ```no_run
/// #![no_main]
///
/// use libfuzzer_sys::fuzz_target;
///
/// // Note: `|input|` is short for `|input: &[u8]|`.
/// fuzz_target!(|input| {
///     let _result: Result<_, _> = my_crate::parse(input);
/// });
/// # mod my_crate { pub fn parse(_: &[u8]) -> Result<(), ()> { unimplemented!() } }
/// ```
///
/// ## Rejecting Inputs
///
/// It may be desirable to reject some inputs, i.e. to not add them to the
/// corpus.
///
/// For example, when fuzzing an API consisting of parsing and other logic,
/// one may want to allow only those inputs into the corpus that parse
/// successfully. To indicate whether an input should be kept in or rejected
/// from the corpus, return either [Corpus::Keep] or [Corpus::Reject] from your
/// fuzz target. The default behavior (e.g. if `()` is returned) is to keep the
/// input in the corpus.
///
/// For example:
///
/// ```no_run
/// #![no_main]
///
/// use libfuzzer_sys::{Corpus, fuzz_target};
///
/// fuzz_target!(|input: String| -> Corpus {
///     let parts: Vec<&str> = input.splitn(2, '=').collect();
///     if parts.len() != 2 {
///         return Corpus::Reject;
///     }
///
///     let key = parts[0];
///     let value = parts[1];
///     let _result: Result<_, _> = my_crate::parse(key, value);
///     Corpus::Keep
/// });
/// # mod my_crate { pub fn parse(_key: &str, _value: &str) -> Result<(), ()> { unimplemented!() } }
/// ```
///
/// ## Arbitrary Input Types
///
/// The input is a `&[u8]` slice by default, but you can take arbitrary input
/// types, as long as the type implements [the `arbitrary` crate's `Arbitrary`
/// trait](https://docs.rs/arbitrary/*/arbitrary/trait.Arbitrary.html) (which is
/// also re-exported as `libfuzzer_sys::arbitrary::Arbitrary` for convenience).
///
/// For example, if you wanted to take an arbitrary RGB color, you could do the
/// following:
///
/// ```no_run
/// #![no_main]
/// # mod foo {
///
/// use libfuzzer_sys::{arbitrary::{Arbitrary, Error, Unstructured}, fuzz_target};
///
/// #[derive(Debug)]
/// pub struct Rgb {
///     r: u8,
///     g: u8,
///     b: u8,
/// }
///
/// impl<'a> Arbitrary<'a> for Rgb {
///     fn arbitrary(raw: &mut Unstructured<'a>) -> Result<Self, Error> {
///         let mut buf = [0; 3];
///         raw.fill_buffer(&mut buf)?;
///         let r = buf[0];
///         let g = buf[1];
///         let b = buf[2];
///         Ok(Rgb { r, g, b })
///     }
/// }
///
/// // Write a fuzz target that works with RGB colors instead of raw bytes.
/// fuzz_target!(|color: Rgb| {
///     my_crate::convert_color(color);
/// });
/// # mod my_crate {
/// #     use super::Rgb;
/// #     pub fn convert_color(_: Rgb) {}
/// # }
/// # }
/// ```
///
/// You can also enable the `arbitrary` crate's custom derive via this crate's
/// `"arbitrary-derive"` cargo feature.
#[macro_export]
macro_rules! fuzz_target {
    (|$bytes:ident| $body:expr) => {
        
        fn main() {
            unsafe {
                libafl_main();
            }
        }
        const _: () = {
            /// Auto-generated function


            #[no_mangle]
            pub extern "C" fn rust_fuzzer_test_input(bytes: &[u8]) -> i32 {
                // When `RUST_LIBFUZZER_DEBUG_PATH` is set, write the debug
                // formatting of the input to that file. This is only intended for
                // `cargo fuzz`'s use!

                // `RUST_LIBFUZZER_DEBUG_PATH` is set in initialization.
                // if let Some(path) = $crate::RUST_LIBFUZZER_DEBUG_PATH.get() {
                //     use std::io::Write;
                //     let mut file = std::fs::File::create(path)
                //         .expect("failed to create `RUST_LIBFUZZER_DEBUG_PATH` file");
                //     writeln!(&mut file, "{:?}", bytes)
                //         .expect("failed to write to `RUST_LIBFUZZER_DEBUG_PATH` file");
                //     return 0;
                // }

                __libfuzzer_sys_run(bytes);
                0
            }

            // Split out the actual fuzzer into a separate function which is
            // tagged as never being inlined. This ensures that if the fuzzer
            // panics there's at least one stack frame which is named uniquely
            // according to this specific fuzzer that this is embedded within.
            //
            // Systems like oss-fuzz try to deduplicate crashes and without this
            // panics in separate fuzzers can accidentally appear the same
            // because each fuzzer will have a function called
            // `rust_fuzzer_test_input`. By using a normal Rust function here
            // it's named something like `the_fuzzer_name::_::__libfuzzer_sys_run` which should
            // ideally help prevent oss-fuzz from deduplicate fuzz bugs across
            // distinct targets accidentally.
            #[inline(never)]
            fn __libfuzzer_sys_run($bytes: &[u8]) {
                $body
            }
        };
    };

    (|$data:ident: &[u8]| $body:expr) => {
        $crate::fuzz_target!(|$data| $body);
    };

    // (|$data:ident: $dty:ty| $body:expr) => {
    //     $crate::fuzz_target!(|$data: $dty| -> () { $body });
    // };

    // (|$data:ident: $dty:ty| -> $rty:ty $body:block) => {
    //     const _: () = {
    //         /// Auto-generated function
    //         #[no_mangle]
    //         pub extern "C" fn rust_fuzzer_test_input(bytes: &[u8]) -> i32 {
    //             use $crate::arbitrary::{Arbitrary, Unstructured};
    //
    //             // Early exit if we don't have enough bytes for the `Arbitrary`
    //             // implementation. This helps the fuzzer avoid exploring all the
    //             // different not-enough-input-bytes paths inside the `Arbitrary`
    //             // implementation. Additionally, it exits faster, letting the fuzzer
    //             // get to longer inputs that actually lead to interesting executions
    //             // quicker.
    //             if bytes.len() < <$dty as Arbitrary>::size_hint(0).0 {
    //                 return -1;
    //             }
    //
    //             let mut u = Unstructured::new(bytes);
    //             let data = <$dty as Arbitrary>::arbitrary_take_rest(u);
    //
    //             // When `RUST_LIBFUZZER_DEBUG_PATH` is set, write the debug
    //             // formatting of the input to that file. This is only intended for
    //             // `cargo fuzz`'s use!
    //
    //             // `RUST_LIBFUZZER_DEBUG_PATH` is set in initialization.
    //             if let Some(path) = $crate::RUST_LIBFUZZER_DEBUG_PATH.get() {
    //                 use std::io::Write;
    //                 let mut file = std::fs::File::create(path)
    //                     .expect("failed to create `RUST_LIBFUZZER_DEBUG_PATH` file");
    //                 (match data {
    //                     Ok(data) => writeln!(&mut file, "{:#?}", data),
    //                     Err(err) => writeln!(&mut file, "Arbitrary Error: {}", err),
    //                 })
    //                 .expect("failed to write to `RUST_LIBFUZZER_DEBUG_PATH` file");
    //                 return -1;
    //             }
    //
    //             let data = match data {
    //                 Ok(d) => d,
    //                 Err(_) => return -1,
    //             };
    //
    //             let result = ::libfuzzer_sys::Corpus::from(__libfuzzer_sys_run(data));
    //             result.to_libfuzzer_code()
    //         }
    //
    //         // See above for why this is split to a separate function.
    //         #[inline(never)]
    //         fn __libfuzzer_sys_run($data: $dty) -> $rty {
    //             $body
    //         }
    //     };
    // };
}

extern "C" {
    // We do not actually cross the FFI bound here.
    #[allow(improper_ctypes)]
    fn rust_fuzzer_test_input(input: &[u8]) -> i32;

    // fn LLVMFuzzerMutate(data: *mut u8, size: usize, max_size: usize) -> usize;
}


#[export_name = "llvmtt1"]
pub unsafe extern "C" fn tt1() {
    println!("Fcuk");
}

/// Do not use; only for LibFuzzer's consumption.
#[doc(hidden)]
#[export_name = "LLVMFuzzerTestOneInput"]
pub unsafe extern "C" fn test_input_wrap(data: *const u8, size: usize) -> i32 {
    let test_input = ::std::panic::catch_unwind(|| {
        let data_slice = ::std::slice::from_raw_parts(data, size);
        rust_fuzzer_test_input(data_slice)
    });

    match test_input {
        Ok(i) => i,
        Err(_) => {
            // hopefully the custom panic hook will be called before and abort the
            // process before the stack frames are unwinded.
            ::std::process::abort();
        }
    }
}

