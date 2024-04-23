// Copyright (c) 2024, Qualcomm Innovation Center, Inc. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

use super::error;
use core::num::NonZeroI32;

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
/// All errors that are considered `UserDefined` are errors specific to the
/// interface. Please refer to corresponding IDL for the values.
pub struct Error(NonZeroI32);

impl From<NonZeroI32> for Error {
    #[inline]
    fn from(value: NonZeroI32) -> Self {
        Self(value)
    }
}

impl From<i32> for Error {
    #[inline]
    fn from(value: i32) -> Self {
        Self(NonZeroI32::new(value).expect("Error should be a non-zero value"))
    }
}

impl From<Error> for i32 {
    #[inline]
    fn from(value: Error) -> Self {
        value.0.get()
    }
}

impl From<Error> for NonZeroI32 {
    #[inline]
    fn from(value: Error) -> Self {
        value.0
    }
}

impl Error {
    #[inline]
    #[must_use]
    /// Creates an [`Error`] instance if the incoming value is non-zero.
    /// Invoking this with a zero will result in `None` being returned.
    pub const fn new(val: i32) -> Option<Self> {
        // All of this will be optimized away in non-zero const context.
        if let Some(non_zero) = core::num::NonZeroI32::new(val) {
            Some(Self(non_zero))
        } else {
            None
        }
    }

    #[inline]
    #[must_use]
    /// Creates an [`Error`] instance without checking for if the value is
    /// zero or not. During const function usage this function will never accept zero.
    /// During non-const use of the function any invocation with zero is undefined
    /// behavior
    ///
    /// # Safety
    /// Use when you know the return value is not zero, else look at the safer
    /// alternative [`Error::new`]
    pub const unsafe fn new_unchecked(val: i32) -> Self {
        Self(core::num::NonZeroI32::new_unchecked(val))
    }
}

macro_rules! define_err {
        ($(#[$comment:meta] $var: ident = $val: expr;)+) => {
            $(
                #[$comment]
                pub const $var: crate::object::error::Error = unsafe { crate::object::Error::new_unchecked($val) };
            )+
        };
    }

pub mod transport {
    define_err![
        /// Object no longer exists
        DEFUNCT   =  -90;
        /// Calling thread must exit
        ABORT     =  -91;
        /// Invalid object context
        BADOBJ    =  -92;
        /// Caller's object table full
        NOSLOTS   =  -93;
        /// Too many args
        MAXARGS   =  -94;
        /// Buffers too large
        MAXDATA   =  -95;
        /// The request could not be processed
        UNAVAIL   =  -96;
        /// Kernel out of memory
        KMEM      =  -97;
        /// Local method sent to remote object
        REMOTE    =  -98;
        /// Cannot forward invocation, remote process is busy
        BUSY      =  -99;
        /// Cannot authenticate message
        AUTH      =  -100;
        /// Message has been replayed.
        REPLAY    =  -101;
        /// Replay counter cannot be incremented
        MAXREPLAY =  -102;
        /// Target of invocation took too long to respond
        TIMEOUT   =  -103;
    ];
}
pub mod generic {
    define_err![
        /// Non-specific error
        GENERIC         =    1;
        /// Unsupported/unrecognized request
        INVALID         =    2;
        /// Supplied buffer/string too large
        SIZE_IN         =    3;
        /// Supplied output buffer too small
        SIZE_OUT        =    4;
        /// Out of memory
        MEM             =    5;
        /// Reserved for future use
        RESERVED1       =    6;
        /// Reserved for future use
        RESERVED2       =    7;
        /// Reserved for future use
        RESERVED3       =    8;
        /// Reserved for future use
        RESERVED4       =    9;
    ];
}

impl core::fmt::Debug for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use error::{generic::*, transport::*};

        match *self {
            // Transport errors
            DEFUNCT => write!(f, "ObjectDefunct"),
            ABORT => write!(f, "ServiceAbnormallyAborted"),
            BADOBJ => write!(f, "BadObject"),
            NOSLOTS => write!(f, "ObjectTableFull"),
            MAXARGS => write!(f, "CallExceededMaxArguments"),
            MAXDATA => write!(f, "CallExceededMaxData"),
            UNAVAIL => write!(f, "ObjectUnavailable"),
            KMEM => write!(f, "KernelOutOfMemory"),
            REMOTE => write!(f, "LocalMethodSentToRemoteObject"),
            BUSY => write!(f, "RemoteServiceIsBusy"),
            AUTH => write!(f, "CannotAuthenticateMessage"),
            REPLAY => write!(f, "RequestHasBeenReplayed"),
            MAXREPLAY => write!(f, "ReplayCounterCannotBeIncremented"),
            TIMEOUT => write!(f, "InvokeTimeout"),

            // Generic errors
            GENERIC => write!(f, "Generic"),
            INVALID => write!(f, "Invalid"),
            SIZE_IN => write!(f, "BufferTooLarge"),
            SIZE_OUT => write!(f, "BufferTooSmall"),
            MEM => write!(f, "OutOfMemory"),
            RESERVED1 => write!(f, "Reserved1"),
            RESERVED2 => write!(f, "Reserved2"),
            RESERVED3 => write!(f, "Reserved3"),
            RESERVED4 => write!(f, "Reserved4"),

            // Interface specific
            _ => write!(f, "InterfaceSpecificError({})", self.0),
        }
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(&self, f)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

#[test]
fn valid_error_strings() {
    // Ensure we don't need any more overhead than and i32
    assert_eq!(core::mem::size_of::<Error>(), core::mem::size_of::<i32>());

    let custom_error = NonZeroI32::new(17_i32).unwrap();
    let transport_error = error::transport::ABORT;

    // transport errors
    assert_eq!(format!("{transport_error:?}"), "ServiceAbnormallyAborted");

    // custom errors
    assert_eq!(
        format!("{:?}", Error::from(custom_error)),
        format!("InterfaceSpecificError({custom_error})")
    );
    assert_eq!(
        format!("{}", Error::from(custom_error)),
        format!("InterfaceSpecificError({custom_error})")
    );
}
