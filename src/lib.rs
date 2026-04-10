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
    type Defintion;
    type Error: Error + 'static;

    fn name() -> &'static str {
        type_name::<Self>()
    }

    async fn load(definition: &Self::Defintion) -> Result<Self, Self::Error>;
}

pub type ResourceRef<T> = CachedOrReloaded<'static, Arc<T>>;

pub struct ResourceCell<T: Resource + 'static> {
    cell: OnceLock<ResourcePointer<T>>,
}

// Store resource definition inline to allow for updates
struct ResourcePointer<T: Resource> {
    definition: T::Defintion,
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
    pub async fn init(&self, definition: T::Defintion) -> Result<(), T::Error> {
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

// #[macro_export]
// macro_rules! resources {
//     ($vis:vis $modname:ident { $($name:ident : $type:path),* }) => {
//         // TODO: $vis?
//         $vis mod $modname {
//             $(pub static $name: $crate::ResourceCell<$type> = $crate::ResourceCell::new();)*

//             pub async fn update_all() -> Result<(), >
//         }
//     };
// }
