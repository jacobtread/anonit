use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

type AnyMap = HashMap<TypeId, Box<dyn Any>>;

#[derive(Default)]
pub struct ProducerCtx {
    data: AnyMap,
}

impl ProducerCtx {
    pub fn insert<T: 'static>(&mut self, val: T) -> Option<T> {
        self.data
            .insert(TypeId::of::<T>(), Box::new(val))
            .and_then(|boxed| {
                (boxed as Box<dyn Any + 'static>)
                    .downcast()
                    .ok()
                    .map(|boxed| *boxed)
            })
    }

    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.data
            .get(&TypeId::of::<T>())
            .and_then(|boxed| (&**boxed as &(dyn Any + 'static)).downcast_ref())
    }
}
