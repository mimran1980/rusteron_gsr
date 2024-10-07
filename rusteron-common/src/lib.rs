use std::any::type_name;
use std::ptr;

pub type Error = Box<dyn std::error::Error>;
pub type Result<T, E = Error> = std::result::Result<T, E>;

pub struct ManagedCResource<T> {
    resource: *mut T,
    cleanup: Box<dyn FnMut(*mut T) -> i32>,
}

impl<T> ManagedCResource<T> {
    pub fn new(
        init: impl FnOnce(*mut *mut T) -> i32,
        cleanup: impl FnMut(*mut T) -> i32 + 'static,
    ) -> Result<Self, i32> {
        let mut resource: *mut T = ptr::null_mut();
        let result = init(&mut resource);
        if result < 0 {
            return Err(result);
        }

        Ok(Self {
            resource,
            cleanup: Box::new(cleanup),
        })
    }

    pub fn get(&self) -> *mut T {
        self.resource
    }
}

impl<T> Drop for ManagedCResource<T> {
    fn drop(&mut self) {
        let result = (self.cleanup)(self.resource);
        if result < 0 {
            eprintln!(
                "Failed to close resource of type {} with error code {}",
                type_name::<T>(),
                result
            );
        } else {
            println!(
                "Closed resource of type {} with success code {}",
                type_name::<T>(),
                result
            );
        }
    }
}