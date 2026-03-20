// src/kernel/sync.rs

use core::cell::UnsafeCell;
use core::hint::spin_loop;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicBool, Ordering};

use crate::arch::aarch64::exception;

pub struct IrqSafeSpinLock<T> {
    locked: AtomicBool,
    value: UnsafeCell<T>,
}

unsafe impl<T: Send> Sync for IrqSafeSpinLock<T> {}
unsafe impl<T: Send> Send for IrqSafeSpinLock<T> {}

pub struct IrqSafeSpinLockGuard<'a, T> {
    lock: &'a IrqSafeSpinLock<T>,
    irq_state: exception::IrqState,
    _not_send: PhantomData<*mut ()>,
}

impl<T> IrqSafeSpinLock<T> {
    pub const fn new(value: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            value: UnsafeCell::new(value),
        }
    }

    #[inline(always)]
    pub fn lock(&self) -> IrqSafeSpinLockGuard<'_, T> {
        let irq_state = exception::save_and_disable_interrupts();

        while self
            .locked
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            while self.locked.load(Ordering::Relaxed) {
                spin_loop();
            }
        }

        IrqSafeSpinLockGuard {
            lock: self,
            irq_state,
            _not_send: PhantomData,
        }
    }

    #[inline(always)]
    pub fn try_lock(&self) -> Option<IrqSafeSpinLockGuard<'_, T>> {
        let irq_state = exception::save_and_disable_interrupts();

        if self
            .locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            Some(IrqSafeSpinLockGuard {
                lock: self,
                irq_state,
                _not_send: PhantomData,
            })
        } else {
            exception::restore_interrupts(irq_state);
            None
        }
    }
}

impl<T> core::ops::Deref for IrqSafeSpinLockGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.value.get() }
    }
}

impl<T> core::ops::DerefMut for IrqSafeSpinLockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.value.get() }
    }
}

impl<T> Drop for IrqSafeSpinLockGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.locked.store(false, Ordering::Release);
        exception::restore_interrupts(self.irq_state);
    }
}