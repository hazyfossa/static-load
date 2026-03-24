use std::{any::type_name, cell::OnceCell, sync::Arc};

use arc_swap::{ArcSwap, Guard};

pub struct ResourceCell<T: Resource + 'static> {
    cell: OnceCell<ResourceRef<T>>,
}

pub struct ResourceRef<T: Resource> {
    definition: T::Defintion,
    ptr: ArcSwap<T>,
}

#[allow(async_fn_in_trait)]
pub trait Resource: Sized {
    type Defintion;
    type LoadError;

    fn name() -> &'static str {
        type_name::<Self>()
    }

    async fn load(definition: &Self::Defintion) -> Result<Self, Self::LoadError>;
}

impl<T: Resource> ResourceCell<T> {
    pub const fn define() -> Self {
        Self {
            cell: OnceCell::new(),
        }
    }

    // Init can only be called once per ResourceCell
    // It is recommended to call it from `main`
    pub async fn init(&self, definition: T::Defintion) -> Result<(), T::LoadError> {
        let instance = T::load(&definition).await?;

        // Store the defintion alongside pointer to allow for updates
        let resource_ref = ResourceRef {
            definition,
            ptr: ArcSwap::from(Arc::new(instance)),
        };

        let ret = self.cell.set(resource_ref);
        if ret.is_err() {
            panic!("Resource {} is initialized twice", T::name())
        }

        Ok(())
    }

    fn get_cell(&self) -> &ResourceRef<T> {
        match self.cell.get() {
            Some(resource_ref) => resource_ref,
            None => panic!("Resource {} not initialized", T::name()),
        }
    }

    /// This function is very cheap to call
    pub fn read(&self) -> Guard<Arc<T>> {
        self.get_cell().ptr.load()
    }

    pub async fn reload(&self) -> Result<(), T::LoadError> {
        let this = self.get_cell();

        let new_instance = T::load(&this.definition).await?;
        this.ptr.store(Arc::new(new_instance));

        Ok(())
    }
}
