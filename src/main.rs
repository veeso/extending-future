mod ext_fut;
mod runtime;
mod task;

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use ext_fut::FutureAdapter;

use self::ext_fut::FutureExt;
use self::runtime::SimpleRuntime;
use self::task::{count, permute};

fn main() {
    let abort = Arc::new(AtomicBool::new(false));
    let runtime = SimpleRuntime::new(&abort);

    // setup ctrlc to abort
    let abort_clone = Arc::clone(&abort);
    ctrlc::set_handler(move || {
        abort_clone.store(true, std::sync::atomic::Ordering::Relaxed);
    })
    .expect("Error setting Ctrl-C handler");

    runtime.block_on(async_main().adapt());
    let permutations = runtime.block_on(permute(&[1, 6, 4, 3, 2, 5], &[1, 2, 3, 4, 5, 6]));
    println!("permutations: {permutations}");
}

async fn async_main() {
    let res = count(10).await;
    println!("async_main {res}");
}
