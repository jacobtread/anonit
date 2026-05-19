use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

type AnyMap = HashMap<TypeId, Box<dyn Any>>;

#[derive(Default)]
pub struct ContextData {
    data: AnyMap,
}

impl ContextData {
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

    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.data
            .get_mut(&TypeId::of::<T>())
            .and_then(|boxed| (&mut **boxed as &mut (dyn Any + 'static)).downcast_mut())
    }

    pub fn get_or_default<T: Default + 'static>(&mut self) -> &mut T {
        let boxed = self
            .data
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(T::default()));

        (&mut **boxed as &mut (dyn Any + 'static))
            .downcast_mut()
            .expect("item cannot be any type other than the expected type")
    }
}
