use std::hint::black_box;

use static_load::{Resource, ResourceCell, ResourceRef};

const THREADS: &[usize] = &[0, 1, 4, 16];

pub(crate) struct Noop;

impl Resource for Noop {
    type Definition = ();
    type Error = std::convert::Infallible;

    fn load(_: &Self::Definition) -> impl Future<Output = Result<Self, Self::Error>> {
        std::future::ready(Ok(Self))
    }
}

static RESOURCE: ResourceCell<Noop> = ResourceCell::new();
static RELOADED_RESOURCE: ResourceCell<Noop> = ResourceCell::new();

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // TODO: currently THREADS are mostly meaningless
    // since we init outside
    RESOURCE.init(()).await;

    RELOADED_RESOURCE.init(()).await;
    RELOADED_RESOURCE.reload().await;

    divan::main();
}

#[divan::bench(threads = THREADS)]
fn load_resource() -> ResourceRef<Noop> {
    black_box(&RESOURCE).read()
}

#[divan::bench(threads = THREADS)]
fn load_reloaded_resource() -> ResourceRef<Noop> {
    black_box(&RELOADED_RESOURCE).read()
}
