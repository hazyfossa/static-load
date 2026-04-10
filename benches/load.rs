use std::{hint::black_box, sync::Arc};

use hazarc::atomic::CachedOrReloaded;
use static_load::{Resource, ResourceCell};

const THREADS: &[usize] = &[0, 1, 4, 16];

struct Test;
impl Resource for Test {
    type Defintion = ();
    type Error = std::convert::Infallible;
    async fn load(_: &Self::Defintion) -> Result<Self, Self::Error> {
        Ok(Self.into())
    }
}

static RESOURCE: ResourceCell<Test> = ResourceCell::new();

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // TODO: we should really init resource inside bench
    // otherwise runs influence each other
    RESOURCE.init(()).await;
    divan::main();
}

#[divan::bench(threads = THREADS)]
fn load_resouce() -> CachedOrReloaded<'static, Arc<Test>> {
    black_box(&RESOURCE).read()
}
