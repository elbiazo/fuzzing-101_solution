use libafl::bolts::current_nanos;
use libafl::bolts::rands::StdRand;
use libafl::bolts::tuples::tuple_list;
use libafl::corpus::{
    Corpus, InMemoryCorpus, IndexesLenTimeMinimizerCorpusScheduler, OnDiskCorpus,
    QueueCorpusScheduler,
};
use libafl::events::{setup_restarting_mgr_std, EventConfig, EventRestarter};
use libafl::executors::{ExitKind, InProcessExecutor, TimeoutExecutor};
use libafl::feedbacks::{CrashFeedback, MapFeedbackState, MaxMapFeedback, TimeFeedback};
use libafl::inputs::{BytesInput, HasTargetBytes};
use libafl::monitors::MultiMonitor;
use libafl::mutators::{havoc_mutations, StdScheduledMutator};
use libafl::observers::{HitcountsMapObserver, StdMapObserver, TimeObserver};
use libafl::stages::StdMutationalStage;
use libafl::state::{HasCorpus, StdState};
use libafl::{feedback_and_fast, feedback_or, Error, Fuzzer, StdFuzzer};
use libafl_targets::{libfuzzer_test_one_input, EDGES_MAP, MAX_EDGES_NUM};
use std::path::PathBuf;
use std::time::Duration;

#[no_mangle]
fn libafl_main() -> Result<(), Error> {
    let corpus_dirs = vec![PathBuf::from("./corpus")];

    let input_corpus = InMemoryCorpus::<BytesInput>::new();

    let solutions_corpus = OnDiskCorpus::<BytesInput>::new(PathBuf::from("./solutions")).unwrap();

    let edges = unsafe { &mut EDGES_MAP[0..MAX_EDGES_NUM] };
    let edges_observer = HitcountsMapObserver::new(StdMapObserver::new("edges", edges));

    let time_observer = TimeObserver::new("time");

    let feedback_state = MapFeedbackState::with_observer(&edges_observer);

    let feedback = feedback_or!(
        MaxMapFeedback::new_tracking(&feedback_state, &edges_observer, true, false),
        TimeFeedback::new_with_observer(&time_observer)
    );
    let objective_state = MapFeedbackState::new("obj_edges", unsafe { EDGES_MAP.len() });
    let objective = feedback_and_fast!(
        CrashFeedback::new(),
        MaxMapFeedback::new(&objective_state, &edges_observer)
    );
    let monitor =MultiMonitor::new(|s| {
        println!("{}", s);
    });

    let (state, mut mgr) = match setup_restarting_mgr_std(monitor, 1337, EventConfig::AlwaysUnique) {
        Ok(res) => res,
        Err(err) => match err {
            Error::ShuttingDown => {
                return Ok(());
            }
            _ => {
                panic!("Failed to setup the restarting manager: {}", err);
            }
        }
    };

    let mut state = state.unwrap_or_else(|| {
        StdState::new(
            StdRand::with_seed(current_nanos()),
            input_corpus,
            solutions_corpus,
            tuple_list!(feedback_state, objective_state),
        )
    });

    let scheduler = IndexesLenTimeMinimizerCorpusScheduler::new(QueueCorpusScheduler::new());
    let mut fuzzer = StdFuzzer::new(scheduler, feedback, objective);

    let mut harness = |input: &BytesInput| {
        let target = input.target_bytes();
        let buffer = target.as_slice();
        libfuzzer_test_one_input(buffer);
        ExitKind::Ok
    };

    let in_proc_executor = InProcessExecutor::new(
        &mut harness,
        tuple_list!(edges_observer, time_observer),
        &mut fuzzer,
        &mut state,
        &mut mgr,
    )
    .unwrap();

    let timeout = Duration::from_millis(5000);
    let mut executor = TimeoutExecutor::new(in_proc_executor, timeout);
    // In case the corpus is empty (i.e. on first run), load existing test cases from on-disk
    // corpus
    if state.corpus().count() < 1 {
        state
            .load_initial_inputs(&mut fuzzer, &mut executor, &mut mgr, &corpus_dirs)
            .unwrap_or_else(|err| {
                panic!(
                    "Failed to load initial corpus at {:?}: {:?}",
                    &corpus_dirs, err
                )
            });
        println!("We imported {} inputs from disk.", state.corpus().count());
    }

    let mutator = StdScheduledMutator::new(havoc_mutations());
    let mut stages = tuple_list!(StdMutationalStage::new(mutator));
    fuzzer
        .fuzz_loop_for(&mut stages, &mut executor, &mut state, &mut mgr, 1000)
        .unwrap();
    mgr.on_restart(&mut state).unwrap();

    Ok(())
}
