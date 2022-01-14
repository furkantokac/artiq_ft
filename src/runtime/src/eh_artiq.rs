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
use libc::{c_int, c_void, uintptr_t};
use log::trace;

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
    backtrace:      [usize; MAX_BACKTRACE_SIZE],
    backtrace_size: usize
}

type _Unwind_Stop_Fn = extern "C" fn(version: c_int,
                                     actions: i32,
                                     exception_class: uw::_Unwind_Exception_Class,
                                     exception_object: *mut uw::_Unwind_Exception,
                                     context: *mut uw::_Unwind_Context,
                                     stop_parameter: *mut c_void)
                                    -> uw::_Unwind_Reason_Code;

extern {
    // not defined in EHABI, but LLVM added it and is useful to us
    fn _Unwind_ForcedUnwind(exception: *mut uw::_Unwind_Exception,
                            stop_fn: _Unwind_Stop_Fn,
                            stop_parameter: *mut c_void) -> uw::_Unwind_Reason_Code;
}

unsafe fn find_eh_action(
    context: *mut uw::_Unwind_Context,
    foreign_exception: bool,
    name: *const u8,
    len: usize,
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
    eh::find_eh_action(lsda, &eh_context, foreign_exception, name, len)
}

pub unsafe fn artiq_personality(_state: uw::_Unwind_State,
                                exception_object: *mut uw::_Unwind_Exception,
                                context: *mut uw::_Unwind_Context)
                                -> uw::_Unwind_Reason_Code {
    // we will only do phase 2 forced unwinding now
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
    assert!(!foreign_exception, "we do not expect foreign exceptions");
    let exception_info = &mut *(exception_object as *mut ExceptionInfo);

    let (name_ptr, len) = if foreign_exception || exception_info.exception.is_none() {
        (core::ptr::null(), 0)
    } else {
        let name = (exception_info.exception.unwrap()).name;
        (name.as_ptr(), name.len())
    };
    let eh_action = match find_eh_action(context, foreign_exception, name_ptr, len) {
        Ok(action) => action,
        Err(_) => return uw::_URC_FAILURE,
    };
    let exception = &exception_info.exception.unwrap();
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

    // On ARM EHABI the personality routine is responsible for actually
    // unwinding a single stack frame before returning (ARM EHABI Sec. 6.1).
    unsafe fn continue_unwind(exception_object: *mut uw::_Unwind_Exception,
                              context: *mut uw::_Unwind_Context)
                              -> uw::_Unwind_Reason_Code {
        let reason = __gnu_unwind_frame(exception_object, context);
        if reason == uw::_URC_NO_REASON {
            uw::_URC_CONTINUE_UNWIND
        } else {
            reason
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
    backtrace:      [0; MAX_BACKTRACE_SIZE],
    backtrace_size: 0
};

pub unsafe extern fn raise(exception: *const Exception) -> ! {
    // FIXME: unsound transmute
    // This would cause stack memory corruption.
    trace!("raising exception");
    INFLIGHT.backtrace_size = 0;
    INFLIGHT.exception = Some(mem::transmute::<Exception, Exception<'static>>(*exception));

    let _result = _Unwind_ForcedUnwind(&mut INFLIGHT.uw_exception,
                                   uncaught_exception, core::ptr::null_mut());
    unreachable!()
}

extern fn uncaught_exception(_version: c_int,
                             actions: i32,
                             _uw_exception_class: uw::_Unwind_Exception_Class,
                             uw_exception: *mut uw::_Unwind_Exception,
                             context: *mut uw::_Unwind_Context,
                             _stop_parameter: *mut c_void)
                            -> uw::_Unwind_Reason_Code {
    unsafe {
        trace!("uncaught exception");
        let exception_info = &mut *(uw_exception as *mut ExceptionInfo);

        if exception_info.backtrace_size < exception_info.backtrace.len() {
            let ip = uw::_Unwind_GetIP(context);
            trace!("SP: {:X}, backtrace_size: {}", uw::_Unwind_GetGR(context, uw::UNWIND_SP_REG), exception_info.backtrace_size);
            exception_info.backtrace[exception_info.backtrace_size] = ip;
            exception_info.backtrace_size += 1;
        }

        if actions as u32 & uw::_US_END_OF_STACK as u32 != 0 {
            crate::kernel::core1::terminate(exception_info.exception.as_ref().unwrap(),
                        exception_info.backtrace[..exception_info.backtrace_size].as_mut())
        } else {
            uw::_URC_NO_REASON
        }
    }
}

pub unsafe extern fn reraise() -> ! {
    use cslice::AsCSlice;

    // Reraise is basically cxa_rethrow, which calls _Unwind_Resume_or_Rethrow,
    // which for EHABI would always call _Unwind_RaiseException.
    match INFLIGHT.exception {
        Some(ex) => {
            // we cannot call raise directly as that would corrupt the backtrace
            INFLIGHT.exception = Some(mem::transmute::<Exception, Exception<'static>>(ex));
            let _result = _Unwind_ForcedUnwind(&mut INFLIGHT.uw_exception,
                                           uncaught_exception, core::ptr::null_mut());
            unreachable!()
        },
        None => {
            raise(&Exception {
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
        unsafe {
            $crate::eh_artiq::raise(&exn)
        }
    });
    ($name:expr, $message:expr) => ({
        artiq_raise!($name, $message, 0, 0, 0)
    });
}
