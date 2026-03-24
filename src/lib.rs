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
    type Error;

    fn name() -> &'static str {
        type_name::<Self>()
    }

    async fn load(definition: &Self::Defintion) -> Result<Self, Self::Error>;
}

impl<T: Resource> ResourceRef<T> {
    async fn init(definition: T::Defintion) -> Result<Self, T::Error> {
        let instance = T::load(&definition).await?;

        Ok(Self {
            definition,
            ptr: ArcSwap::from(Arc::new(instance)),
        })
    }

    async fn update(&self) -> Result<(), T::Error> {
        // TODO: raw load: return Arc
        let new_instance = T::load(&self.definition).await?;
        self.ptr.store(Arc::new(new_instance));
        Ok(())
    }
}

impl<T: Resource> ResourceCell<T> {
    pub const fn define() -> Self {
        Self {
            cell: OnceCell::new(),
        }
    }

    pub async fn init(&self, definition: T::Defintion) -> Result<(), T::Error> {
        let resource_ref = ResourceRef::init(definition).await?;

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

    pub async fn update(&self) -> Result<(), T::Error> {
        self.get_cell().update().await
    }
}
