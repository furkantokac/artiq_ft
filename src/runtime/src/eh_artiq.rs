// From Current artiq firmware ksupport implementation.
// Modified to suit the case of artiq-zynq port, for ARM EHABI.
// Portions of the code in this file are derived from code by:
//
// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
#![allow(non_camel_case_types)]

use core::mem;
use cslice::CSlice;
use unwind as uw;
use libc::{c_int, uintptr_t};
use log::debug;

use dwarf::eh::{self, EHAction, EHContext};


const EXCEPTION_CLASS: uw::_Unwind_Exception_Class = 0x4d_4c_42_53_41_52_54_51; /* 'MLBSARTQ' */

#[cfg(target_arch = "arm")]
const UNWIND_DATA_REG: (i32, i32) = (0, 1); // R0, R1

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Exception<'a> {
    pub name:     CSlice<'a, u8>,
    pub file:     CSlice<'a, u8>,
    pub line:     u32,
    pub column:   u32,
    pub function: CSlice<'a, u8>,
    pub message:  CSlice<'a, u8>,
    pub param:    [i64; 3]
}

fn str_err(_: core::str::Utf8Error) -> core::fmt::Error {
    core::fmt::Error
}

impl<'a> core::fmt::Debug for Exception<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Exception {} from {} in {}:{}:{}, message: {}",
            core::str::from_utf8(self.name.as_ref()).map_err(str_err)?,
            core::str::from_utf8(self.function.as_ref()).map_err(str_err)?,
            core::str::from_utf8(self.file.as_ref()).map_err(str_err)?,
            self.line, self.column,
            core::str::from_utf8(self.message.as_ref()).map_err(str_err)?)
    }
}

const MAX_BACKTRACE_SIZE: usize = 128;

#[repr(C)]
struct ExceptionInfo {
    uw_exception:   uw::_Unwind_Exception,
    exception:      Option<Exception<'static>>,
    handled:        bool,
    backtrace:      [usize; MAX_BACKTRACE_SIZE],
    backtrace_size: usize
}

unsafe fn find_eh_action(
    context: *mut uw::_Unwind_Context,
    foreign_exception: bool,
) -> Result<EHAction, ()> {
    let lsda = uw::_Unwind_GetLanguageSpecificData(context) as *const u8;
    let mut ip_before_instr: c_int = 0;
    let ip = uw::_Unwind_GetIPInfo(context, &mut ip_before_instr);
    let eh_context = EHContext {
        // The return address points 1 byte past the call instruction,
        // which could be in the next IP range in LSDA range table.
        ip: if ip_before_instr != 0 { ip } else { ip - 1 },
        func_start: uw::_Unwind_GetRegionStart(context),
        get_text_start: &|| uw::_Unwind_GetTextRelBase(context),
        get_data_start: &|| uw::_Unwind_GetDataRelBase(context),
    };
    eh::find_eh_action(lsda, &eh_context, foreign_exception)
}

pub unsafe fn artiq_personality(state: uw::_Unwind_State,
                                         exception_object: *mut uw::_Unwind_Exception,
                                         context: *mut uw::_Unwind_Context)
                                         -> uw::_Unwind_Reason_Code {
    let state = state as c_int;
    let action = state & uw::_US_ACTION_MASK as c_int;
    let search_phase = if action == uw::_US_VIRTUAL_UNWIND_FRAME as c_int {
        // Backtraces on ARM will call the personality routine with
        // state == _US_VIRTUAL_UNWIND_FRAME | _US_FORCE_UNWIND. In those cases
        // we want to continue unwinding the stack, otherwise all our backtraces
        // would end at __rust_try
        if state & uw::_US_FORCE_UNWIND as c_int != 0 {
            return continue_unwind(exception_object, context);
        }
        true
    } else if action == uw::_US_UNWIND_FRAME_STARTING as c_int {
        false
    } else if action == uw::_US_UNWIND_FRAME_RESUME as c_int {
        return continue_unwind(exception_object, context);
    } else {
        return uw::_URC_FAILURE;
    };

    // The DWARF unwinder assumes that _Unwind_Context holds things like the function
    // and LSDA pointers, however ARM EHABI places them into the exception object.
    // To preserve signatures of functions like _Unwind_GetLanguageSpecificData(), which
    // take only the context pointer, GCC personality routines stash a pointer to
    // exception_object in the context, using location reserved for ARM's
    // "scratch register" (r12).
    uw::_Unwind_SetGR(context,
                      uw::UNWIND_POINTER_REG,
                      exception_object as uw::_Unwind_Ptr);
    // ...A more principled approach would be to provide the full definition of ARM's
    // _Unwind_Context in our libunwind bindings and fetch the required data from there
    // directly, bypassing DWARF compatibility functions.

    let exception_class = (*exception_object).exception_class;
    let foreign_exception = exception_class != EXCEPTION_CLASS;
    let eh_action = match find_eh_action(context, foreign_exception) {
        Ok(action) => action,
        Err(_) => return uw::_URC_FAILURE,
    };
    let exception_info = &mut *(exception_object as *mut ExceptionInfo);
    let exception = &exception_info.exception.unwrap();
    if search_phase {
        match eh_action {
            EHAction::None |
            EHAction::Cleanup(_) => return continue_unwind(exception_object, context),
            EHAction::Catch(_) => {
                // EHABI requires the personality routine to update the
                // SP value in the barrier cache of the exception object.
                (*exception_object).private[5] =
                    uw::_Unwind_GetGR(context, uw::UNWIND_SP_REG);
                return uw::_URC_HANDLER_FOUND;
            }
            EHAction::Terminate => return uw::_URC_FAILURE,
        }
    } else {
        match eh_action {
            EHAction::None => return continue_unwind(exception_object, context),
            EHAction::Cleanup(lpad) |
            EHAction::Catch(lpad) => {
                uw::_Unwind_SetGR(context, UNWIND_DATA_REG.0,
                                  exception_object as uintptr_t);
                uw::_Unwind_SetGR(context, UNWIND_DATA_REG.1, exception as *const _ as uw::_Unwind_Word);
                uw::_Unwind_SetIP(context, lpad);
                return uw::_URC_INSTALL_CONTEXT;
            }
            EHAction::Terminate => return uw::_URC_FAILURE,
        }
    }

    // On ARM EHABI the personality routine is responsible for actually
    // unwinding a single stack frame before returning (ARM EHABI Sec. 6.1).
    unsafe fn continue_unwind(exception_object: *mut uw::_Unwind_Exception,
                              context: *mut uw::_Unwind_Context)
                              -> uw::_Unwind_Reason_Code {
        if __gnu_unwind_frame(exception_object, context) == uw::_URC_NO_REASON {
            uw::_URC_CONTINUE_UNWIND
        } else {
            uw::_URC_FAILURE
        }
    }
    // defined in libgcc
    extern "C" {
        fn __gnu_unwind_frame(exception_object: *mut uw::_Unwind_Exception,
                              context: *mut uw::_Unwind_Context)
                              -> uw::_Unwind_Reason_Code;
    }
}

extern fn cleanup(_unwind_code: uw::_Unwind_Reason_Code,
                  uw_exception: *mut uw::_Unwind_Exception) {
    unsafe {
        let exception_info = &mut *(uw_exception as *mut ExceptionInfo);

        exception_info.exception = None;
    }
}

static mut INFLIGHT: ExceptionInfo = ExceptionInfo {
    uw_exception: uw::_Unwind_Exception {
        exception_class:   EXCEPTION_CLASS,
        exception_cleanup: cleanup,
        private:           [0; uw::unwinder_private_data_size],
    },
    exception:      None,
    handled:        true,
    backtrace:      [0; MAX_BACKTRACE_SIZE],
    backtrace_size: 0
};

pub unsafe extern fn raise(exception: *const Exception) -> ! {
    // Zing! The Exception<'a> to Exception<'static> transmute is not really sound in case
    // the exception is ever captured. Fortunately, they currently aren't, and we save
    // on the hassle of having to allocate exceptions somewhere except on stack.
    debug!("Trying to raise exception");
    INFLIGHT.exception = Some(mem::transmute::<Exception, Exception<'static>>(*exception));
    INFLIGHT.handled   = false;

    let result = uw::_Unwind_RaiseException(&mut INFLIGHT.uw_exception);
    assert!(result == uw::_URC_END_OF_STACK);

    INFLIGHT.backtrace_size = 0;
    // read backtrace
    let _ = uw::backtrace(|ip| {
        if INFLIGHT.backtrace_size < MAX_BACKTRACE_SIZE {
            INFLIGHT.backtrace[INFLIGHT.backtrace_size] = ip;
            INFLIGHT.backtrace_size += 1;
        }
    });
    crate::kernel::terminate(INFLIGHT.exception.as_ref().unwrap(), INFLIGHT.backtrace[..INFLIGHT.backtrace_size].as_mut());
}

pub unsafe extern fn reraise() -> ! {
    use cslice::AsCSlice;

    debug!("Re-raise");
    // current implementation uses raise as _Unwind_Resume is not working now
    // would debug that later.
    match INFLIGHT.exception {
        Some(ref exception) => raise(exception),
        None => raise(&Exception {
            name:     "0:artiq.coredevice.exceptions.RuntimeError".as_c_slice(),
            file:     file!().as_c_slice(),
            line:     line!(),
            column:   column!(),
            // https://github.com/rust-lang/rfcs/pull/1719
            function: "__artiq_reraise".as_c_slice(),
            message:  "No active exception to reraise".as_c_slice(),
            param:    [0, 0, 0]
        })
    }
}

#[macro_export]
macro_rules! artiq_raise {
    ($name:expr, $message:expr, $param0:expr, $param1:expr, $param2:expr) => ({
        use cslice::AsCSlice;
        let exn = $crate::eh_artiq::Exception {
            name:     concat!("0:artiq.coredevice.exceptions.", $name).as_c_slice(),
            file:     file!().as_c_slice(),
            line:     line!(),
            column:   column!(),
            // https://github.com/rust-lang/rfcs/pull/1719
            function: "(Rust function)".as_c_slice(),
            message:  $message.as_c_slice(),
            param:    [$param0, $param1, $param2]
        };
        #[allow(unused_unsafe)]
        unsafe { $crate::eh_artiq::raise(&exn) }
    });
    ($name:expr, $message:expr) => ({
        artiq_raise!($name, $message, 0, 0, 0)
    });
}
