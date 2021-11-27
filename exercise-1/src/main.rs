use libafl::feedback_or;
use libafl::feedbacks::map::{MapFeedbackState, MaxMapFeedback};
use libafl::feedbacks::TimeFeedback;
use libafl::bolts::shmem::StdShMemProvider;
use libafl::bolts::shmem::ShMemProvider;
use libafl::bolts::shmem::ShMem;
use libafl::corpus::{InMemoryCorpus, OnDiskCorpus};
use libafl::inputs::bytes::BytesInput;
use libafl::observers::map::{ConstMapObserver, HitcountsMapObserver};
use libafl::observers::TimeObserver;
use std::path::PathBuf;

fn main() {
    let corpus_dirs = vec![PathBuf::from("./corpus")];
    let input_corpus = InMemoryCorpus::<BytesInput>::new();

    let timeouts_corpus =
        OnDiskCorpus::new(PathBuf::from("./timeouts")).expect("Could not create timeouts corpus");
    let time_observer = TimeObserver::new("time");
    const MAP_SIZE: usize = 65536;

    let mut shmem = StdShMemProvider::new().unwrap().new_map(MAP_SIZE).unwrap();

    shmem
        .write_to_env("__AFL_SHM_ID")
        .expect("couldn't write shared memory ID");
    let mut shmem_map = shmem.map_mut();

    let edges_observer = HitcountsMapObserver::new(ConstMapObserver::<_, MAP_SIZE>::new(
        "shared_mem",
        &mut shmem_map,
    ));

    let feedback_state = MapFeedbackState::with_observer(&edges_observer);
    let feedback = feedback_or!(
            MaxMapFeedback::new_tracking(&feedback_state, &edges_observer, true, false),
                TimeFeedback::new_with_observer(&time_observer)
    );

    let objective_state = MapFeedbackState::new("timeout_edges", MAP_SIZE);
}
