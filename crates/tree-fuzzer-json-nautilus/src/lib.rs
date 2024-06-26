// pub extern "C" fn main() {
//     println!("New main");
// }

// use mimalloc::MiMalloc;
// #[global_allocator]
// static GLOBAL: MiMalloc = MiMalloc;

use std::{env, path::PathBuf};

use libafl::{
    corpus::{Corpus, InMemoryCorpus, OnDiskCorpus},
    events::{setup_restarting_mgr_std, EventConfig},
    executors::{inprocess::InProcessExecutor, ExitKind, ShadowExecutor},
    feedback_or,

    feedbacks::{CrashFeedback, MaxMapFeedback, TimeFeedback, NautilusChunksMetadata, NautilusFeedback},
    fuzzer::{Fuzzer, StdFuzzer},
    generators::{NautilusContext, NautilusGenerator},
    mutators::{
        NautilusRandomMutator, NautilusRecursionMutator, NautilusSpliceMutator, StdScheduledMutator,
    },

    inputs::{NautilusInput, NautilusToBytesInputConverter},
    monitors::MultiMonitor,
    mutators::{
        scheduled::{havoc_mutations},
        token_mutations::I2SRandReplace,
    },
    observers::TimeObserver,
    schedulers::{IndexesLenTimeMinimizerScheduler, QueueScheduler},
    stages::{ShadowTracingStage, StdMutationalStage},
    state::{HasMetadata, HasCorpus, StdState},
    Error,
};
use libafl_bolts::{current_nanos, rands::StdRand, tuples::tuple_list, AsSlice};
use libafl_targets::{
    libfuzzer_initialize, libfuzzer_test_one_input, std_edges_map_observer, CmpLogObserver,
};

#[no_mangle]
pub extern "C" fn libafl_main() {
    // Registry the metadata types used in this fuzzer
    // Needed only on no_std
    // unsafe { RegistryBuilder::register::<Tokens>(); }

    println!(
        "Workdir: {:?}",
        env::current_dir().unwrap().to_string_lossy().to_string()
    );
    env_logger::init();
    fuzz(
        &[PathBuf::from("./corpus")],
        PathBuf::from("./crashes"),
        1337,
    )
    .expect("An error occurred while fuzzing");
}

/// The actual fuzzer
fn fuzz(corpus_dirs: &[PathBuf], objective_dir: PathBuf, broker_port: u16) -> Result<(), Error> {
    // 'While the stats are state, they are usually used in the broker - which is likely never restarted
    let monitor = MultiMonitor::new(|s| println!("Monitor: {s}"));

    let context = NautilusContext::from_file(15, "grammar1.json");

    // The restarting state will spawn the same process again as child, then restarted it each time it crashes.
    let (state, mut restarting_mgr) =
        match setup_restarting_mgr_std(monitor, broker_port, EventConfig::from_name("default")) {
            Ok(res) => res,
            Err(err) => match err {
                Error::ShuttingDown => {
                    return Ok(());
                }
                _ => {
                    panic!("Failed to setup the restarter: {err}");
                }
            },
        };
    println!("state restart");
    // Create an observation channel using the coverage map
    // We don't use the hitcounts (see the Cargo.toml, we use pcguard_edges)
    let edges_observer = unsafe { std_edges_map_observer("edges") };

    // Create an observation channel to keep track of the execution time
    let time_observer = TimeObserver::new("time");

    let cmplog_observer = CmpLogObserver::new("cmplog", true);

    // Feedback to rate the interestingness of an input
    // This one is composed by two Feedbacks in OR
    let mut feedback = feedback_or!(
        // New maximization map feedback linked to the edges observer and the feedback state
        MaxMapFeedback::tracking(&edges_observer, true, false),
        NautilusFeedback::new(&context),
        // Time feedback, this one does not need a feedback state
        TimeFeedback::with_observer(&time_observer)
    );

    // A feedback to choose if an input is a solution or not
    let mut objective = CrashFeedback::new();

    // If not restarting, create a State from scratch
    let mut state = state.unwrap_or_else(|| {
        StdState::new(
            // RNG
            StdRand::with_seed(current_nanos()),
            // Corpus that will be evolved, we keep it in memory for performance
            InMemoryCorpus::new(),
            // Corpus in which we store solutions (crashes in this example),
            // on disk so the user can get them after stopping the fuzzer
            OnDiskCorpus::new(objective_dir).unwrap(),
            // States of the feedbacks.
            // The feedbacks can report the data that should persist in the State.
            &mut feedback,
            // Same for objective feedbacks
            &mut objective,
        )
        .unwrap()
    });

    println!("We're a client, let's fuzz :)");

    // A minimization+queue policy to get testcasess from the corpus
    let scheduler = IndexesLenTimeMinimizerScheduler::new(QueueScheduler::new());

    // A fuzzer with feedbacks and a corpus scheduler
    let mut fuzzer = StdFuzzer::new(scheduler, feedback, objective);

    let mut bytes = vec![];
    // The wrapped harness function, calling out to the LLVM-style harness
    let mut harness = |input: &NautilusInput| {
        input.unparse(&context, &mut bytes);
        // let target = input.target_bytes();
        // let buf = target.as_slice();
        libfuzzer_test_one_input(&bytes);
        ExitKind::Ok
    };
    if state
            .metadata_map()
            .get::<NautilusChunksMetadata>()
            .is_none()
        {
            state.add_metadata(NautilusChunksMetadata::new("/tmp/".into()));
        }

    // Create the executor for an in-process function with just one observer for edge coverage
    let mut executor = 
        // ShadowExecutor::new(
        InProcessExecutor::new(
            &mut harness,
            tuple_list!(edges_observer, time_observer),
            &mut fuzzer,
            &mut state,
            &mut restarting_mgr,
        )?
        // ,
        // tuple_list!(cmplog_observer),
    // )
    ;


    // The actual target run starts here.
    // Call LLVMFUzzerInitialize() if present.
    let args: Vec<String> = env::args().collect();
    if libfuzzer_initialize(&args) == -1 {
        println!("Warning: LLVMFuzzerInitialize failed with -1");
    }

    let mut generator = NautilusGenerator::new(&context);
    // In case the corpus is empty (on first run), reset
    state
            .generate_initial_inputs_forced(&mut fuzzer, &mut executor, &mut generator, &mut restarting_mgr, 8)
            .expect("Failed to generate the initial corpus");


    // Setup a tracing stage in which we log comparisons
    // let tracing = ShadowTracingStage::new(&mut executor);

    // Setup a randomic Input2State stage
    // let i2s = StdMutationalStage::new(StdScheduledMutator::new(tuple_list!(I2SRandReplace::new())));

    // Setup a basic mutator
    let mutator = StdScheduledMutator::with_max_stack_pow(
            tuple_list!(
                NautilusRandomMutator::new(&context),
                NautilusRandomMutator::new(&context),
                NautilusRandomMutator::new(&context),
                NautilusRandomMutator::new(&context),
                NautilusRandomMutator::new(&context),
                NautilusRandomMutator::new(&context),
                NautilusRecursionMutator::new(&context),
                NautilusSpliceMutator::new(&context),
                NautilusSpliceMutator::new(&context),
                NautilusSpliceMutator::new(&context),
            ),
            2,
        );
    let mutational = StdMutationalStage::new(mutator);

    // The order of the stages matter!
    let mut stages = tuple_list!(mutational);
    println!("To fuzz_loop");

    fuzzer.fuzz_loop(&mut stages, &mut executor, &mut state, &mut restarting_mgr)?;

    // Never reached
    Ok(())
}
