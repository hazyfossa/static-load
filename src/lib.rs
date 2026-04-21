use std::{
    any::type_name,
    error::Error,
    sync::{Arc, OnceLock},
};

use hazarc::{AtomicArc, Cache, atomic::CachedOrReloaded};

// TODO: if we abandon Cache, we can return fully owned ArcBorrows from read,
// turning resources from `static` to `const` and never requiring $mut (no this! macro)
//
// while for our very-infrequent-update case the benefits of Cache (probably) outweight
// drawbacks, a proper benchmark would be nice

#[allow(async_fn_in_trait)]
// TODO: allow unsized resources which manage their own Arc layout
pub trait Resource: Sized {
    type Definition;
    type Error: Error + 'static;

    fn name() -> &'static str {
        type_name::<Self>()
    }

    async fn load(definition: &Self::Definition) -> Result<Self, Self::Error>;
}

pub type ResourceRef<T> = CachedOrReloaded<'static, Arc<T>>;

pub struct ResourceCell<T: Resource + 'static> {
    cell: OnceLock<ResourcePointer<T>>,
}

// Store resource definition inline to allow for updates
struct ResourcePointer<T: Resource> {
    definition: T::Definition,
    cached_ptr: hazarc::Cache<AtomicArc<T>>,
}

// Expects the `self` cell to be initialized
macro_rules! this {
    ($self:ident.$method:ident) => {
        $self
            .cell
            .$method()
            .expect(&format!("Resource {} not initialized", T::name()))
    };
}

impl<T: Resource> ResourceCell<T> {
    pub const fn new() -> Self {
        Self {
            cell: OnceLock::new(),
        }
    }

    /// Init can only be called once per ResourceCell
    /// It is recommended to call it from `main`
    pub async fn init(&self, definition: T::Definition) -> Result<(), T::Error> {
        let instance = T::load(&definition).await?;
        let ptr = AtomicArc::from(instance);
        let cached_ptr = Cache::new(ptr);

        // Store the defintion alongside pointer to allow for updates
        let resource_ptr = ResourcePointer {
            definition,
            cached_ptr,
        };

        let ret = self.cell.set(resource_ptr);
        if ret.is_err() {
            panic!("Resource {} is initialized twice", T::name())
        }

        Ok(())
    }

    /// This function is very cheap to call
    ///
    /// For initial data (before a reload), performance should be
    /// comparable to a 'static pointer dereference
    ///
    /// For hot-reloaded data, performance is comparable to
    /// loading from arc-swap (still very fast)
    pub fn read(&self) -> CachedOrReloaded<'_, Arc<T>> {
        let this = this!(self.get);
        this.cached_ptr.load_shared()
    }

    pub fn manual_update(&self, new: T) {
        let this = this!(self.get);
        this.cached_ptr.inner().store(new.into());
    }

    pub async fn reload(&self) -> Result<(), T::Error> {
        let this = this!(self.get);

        let new_instance = T::load(&this.definition).await?.into();
        this.cached_ptr.inner().store(new_instance);

        Ok(())
    }
}

#[cfg(feature = "bundle")]
#[macro_export]
macro_rules! resources {
    ($vis:vis $name:ident {
        $($resource:ident: $type:ty),* $(,)?
    }) => {
        $vis mod $name { paste::paste! {
            use super::*;
            use $crate::{Resource, ResourceCell};

            $(pub static [<$resource:upper>]: ResourceCell<$type> = ResourceCell::new();)*

            pub async fn init($([<$resource:lower>]: <$type as Resource>::Definition),*) -> Result<(), String> {
                $crate::resources!(@parallel "Initializing" ret => {
                    $($resource.init([<$resource:lower>]))* }
                );
                ret
            }


            pub async fn reload_all() -> Result<(), String> {
                $crate::resources!(@parallel "Reloading" ret => { $($resource.reload())* });
                ret
            }
        }}
    };

    (@parallel $action:literal $ret:ident => { $( $resource:ident . $fn:tt($($arg:tt)?) )* }) => { paste::paste! {
        let mut tasks = tokio::task::JoinSet::new();

        $(tasks.spawn(async {
            [<$resource:upper>].$fn($($arg)?).await
            .map_err(|e| format!("{} resource {} failed: {e:?}", $action, stringify!($resource)))
        });)*

        let $ret = tasks.join_all().await.into_iter().collect();
    }};
}
