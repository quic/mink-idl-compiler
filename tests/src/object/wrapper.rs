// Copyright (c) 2024, Qualcomm Innovation Center, Inc. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

#![cfg(feature = "std")]

use core::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

/// Wrapper over an interface `T` which is an interface, and the the concrete implementation of
/// some type `I` must be `dyn T`.
///
/// All fields in the [`Wrapper`] is protected by mutual exclusivity except `type_id` this is
/// because this is read only and never retagged to write into after creation.
pub struct Wrapper<T: ?Sized + 'static> {
    pub refs: AtomicUsize,
    pub type_id: std::any::TypeId,
    pub inner: Mutex<Box<T>>,
}

impl<T: ?Sized + 'static> Wrapper<T> {
    #[inline]
    /// Creates a new `Wrapper` from `Box<T>`
    ///
    /// # Safety
    ///
    /// This function exhibits safety as long as `I` and `T` can be _as-cast_.
    ///
    /// # Example
    /// an example of this would be a trait `T` which is implemented for `I`. Hence `I` can be
    /// _as-cast_ to `dyn T`.
    pub unsafe fn new<I: 'static>(inner: Box<T>) -> Self {
        Self {
            refs: AtomicUsize::new(1),
            type_id: std::any::TypeId::of::<I>(),
            inner: Mutex::new(inner),
        }
    }
}

#[inline]
/// Increases ref count for the wrapper
///
/// # Panics
///
/// Panics if refcounter exceeds [`usize::MAX`]
///
/// # Safety
///
/// This function is safe as long as `wrapper` is a valid pointer to `Wrapper<T>`.
pub unsafe fn retain<T: ?Sized + 'static>(wrapper: *mut Wrapper<T>) -> i32 {
    if (*wrapper).refs.fetch_add(1, Ordering::Relaxed) != usize::MAX {
        super::OK
    } else {
        unreachable!()
    }
}

#[inline]
/// Decreases ref count for the wrapper
///
/// # Panics
///
/// Panics if refcounter underflows [`usize::MAX`], this should never happen and even if it does it
/// isn't detectable easily since it might point to freed memory. This indicates a transport layer
/// bug, or some memory corruption
///
/// # Safety
///
/// Assuming the pointer passed in is a valid un-freed pointer to a `Wrapper<T>` this function is
/// safe. This function exhibits undefined behavior if a transport layer bug can causes
/// double-frees
pub unsafe fn release<T: ?Sized + 'static>(wrapper: *mut Wrapper<T>) -> i32 {
    match (*wrapper).refs.fetch_sub(1, Ordering::SeqCst) {
        1 => std::mem::drop(Box::from_raw(wrapper)),
        0 => {
            unreachable!()
        }
        _ => {}
    };

    super::OK
}

#[inline]
/// Downcasts to a concrete type `I` if the `Wrapper<T>` contains the [`std::any::TypeId`] of `I`.
///
/// This function also ensures that the calling function does match the [`super::Object::invoke`] field
/// using the `marker` variable
///
/// Returns None if [`super::Object`] doesn't match `marker` or if [`std::mem::TypeId`] doesn't
/// match.
pub fn downcast_concrete<R, I: 'static, T: ?Sized + 'static>(
    object: &super::Object,
    marker: super::Invoke,
    mut f: impl FnMut(&I) -> R,
) -> Option<R> {
    let wrapper = object
        .get_raw_context(marker)
        .map(<super::Ctx>::cast::<Wrapper<T>>)?;
    if unsafe { (*wrapper).type_id } == std::any::TypeId::of::<I>() {
        let locked = unsafe { (*wrapper).inner.lock().unwrap() };
        let ptr_t = Box::as_ref(&locked) as *const T;
        // Safety: Address is actually an `I` and this retag is exclusive
        let i = unsafe { &*(ptr_t as *const I) };
        let r = Some(f(i));
        // To ensure the computation of `f(i)` still holds the lock
        drop(locked);
        r
    } else {
        None
    }
}
