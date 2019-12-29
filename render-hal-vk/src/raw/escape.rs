#![allow(dead_code)]

//! Module provides wrapper for types that cannot be dropped silently.
//! Usually such types are required to be returned to their creator.
//! `Escape` wrapper help the user to do so by sending underlying value to the `Terminal` when it is dropped.
//! Users are encouraged to dispose of the values manually while `Escape` be just a safety net.

use std::{
    iter::repeat,
    mem::{forget, ManuallyDrop},
    ops::{Deref, DerefMut},
    ptr::read,
};

use crossbeam_channel::{unbounded, Receiver, Sender};
use relevant::Relevant;

use crate::raw::object::VulkanObject;

/// Wraps value of any type and send it to the `Terminal` from which the wrapper was created.
/// In case `Terminal` is already dropped then value will be cast into oblivion via `std::mem::forget`.
#[derive(Debug, Clone)]
pub struct Escape<T: Into<VulkanObject>> {
    value: ManuallyDrop<T>,
    sender: Sender<VulkanObject>,
}

impl<T> Escape<T>
where
    T: Into<VulkanObject>,
{
    /// Unwrap the value.
    pub fn into_inner(mut escape: Self) -> T {
        unsafe { read(&mut *escape.value) }
    }
}

impl<T> Deref for Escape<T>
where
    T: Into<VulkanObject>,
{
    type Target = T;
    fn deref(&self) -> &T {
        &*self.value
    }
}

impl<T> DerefMut for Escape<T>
where
    T: Into<VulkanObject>,
{
    fn deref_mut(&mut self) -> &mut T {
        &mut *self.value
    }
}

impl<T> Drop for Escape<T>
where
    T: Into<VulkanObject>,
{
    fn drop(&mut self) {
        let value = unsafe { read(&mut *self.value) };
        self.sender.send(value.into()).unwrap();
    }
}

/// This types allows the user to create `Escape` wrappers.
/// Receives values from dropped `Escape` instances that was created by this `Terminal`.
#[derive(Debug)]
pub struct Terminal {
    sender: ManuallyDrop<Sender<VulkanObject>>,
    receiver: Receiver<VulkanObject>,
}

impl Default for Terminal {
    fn default() -> Self {
        Self::new()
    }
}

impl Terminal {
    /// Create new `Terminal`.
    pub fn new() -> Self {
        let (sender, receiver) = unbounded();
        Terminal {
            sender: ManuallyDrop::new(sender),
            receiver,
        }
    }

    /// Wrap the value. It will be yielded by iterator returned by `Terminal::drain` if `Escape` will be dropped.
    pub fn escape<T>(&self, value: T) -> Escape<T>
    where
        T: Into<VulkanObject>,
    {
        Escape {
            value: ManuallyDrop::new(value),
            sender: Sender::clone(&self.sender),
        }
    }

    // Get iterator over values from dropped `Escape` instances that was created by this `Terminal`.
    /*pub fn drain<'a>(&'a mut self) -> impl Iterator<Item = VulkanObject> + 'a {
        repeat(()).scan(&self.receiver, |receiver, ()| receiver.try_recv())
    }*/
}

impl Drop for Terminal {
    fn drop(&mut self) {
        unsafe {
            drop(read(&mut self.sender));
            trace!("Sender dropped");
            // TODO: assert!(self.receiver.try_recv().is_none());
            trace!("Receiver checked");
        }
    }
}
