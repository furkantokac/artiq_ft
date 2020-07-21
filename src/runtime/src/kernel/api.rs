use libm;

use crate::eh_artiq;
use crate::rtio;
use super::rpc::{rpc_send, rpc_send_async, rpc_recv};
use super::dma;

macro_rules! api {
    ($i:ident) => ({
        extern { static $i: u8; }
        unsafe { api!($i = &$i as *const _) }
    });
    ($i:ident, $d:item) => ({
        $d
        api!($i = $i)
    });
    ($i:ident = $e:expr) => {
        (stringify!($i), $e as *const ())
    }
}

macro_rules! api_libm_f64f64 {
    ($i:ident) => ({
        extern fn $i(x: f64) -> f64 {
            libm::$i(x)
        }
        api!($i = $i)
    })
}

pub fn resolve(required: &[u8]) -> Option<u32> {
    let api = &[
        // timing
        api!(now_mu = rtio::now_mu),
        api!(at_mu = rtio::at_mu),
        api!(delay_mu = rtio::delay_mu),

        // rpc
        api!(rpc_send = rpc_send),
        api!(rpc_send_async = rpc_send_async),
        api!(rpc_recv = rpc_recv),

        // rtio
        api!(rtio_init = rtio::init),
        api!(rtio_get_destination_status = rtio::get_destination_status),
        api!(rtio_get_counter = rtio::get_counter),
        api!(rtio_output = rtio::output),
        api!(rtio_output_wide = rtio::output_wide),
        api!(rtio_input_timestamp = rtio::input_timestamp),
        api!(rtio_input_data = rtio::input_data),
        api!(rtio_input_timestamped_data = rtio::input_timestamped_data),
        api!(rtio_log = rtio::log),

        // rtio dma
        api!(dma_record_start = dma::dma_record_start),
        api!(dma_record_stop = dma::dma_record_stop),
        api!(dma_erase = dma::dma_erase),
        api!(dma_retrieve = dma::dma_retrieve),
        api!(dma_playback = dma::dma_playback),

        // Double-precision floating-point arithmetic helper functions
        // RTABI chapter 4.1.2, Table 2
        api!(__aeabi_dadd),
        api!(__aeabi_ddiv),
        api!(__aeabi_dmul),
        api!(__aeabi_dsub),
        // Double-precision floating-point comparison helper functions
        // RTABI chapter 4.1.2, Table 3
        api!(__aeabi_dcmpeq),
        api!(__aeabi_dcmpeq),
        api!(__aeabi_dcmplt),
        api!(__aeabi_dcmple),
        api!(__aeabi_dcmpge),
        api!(__aeabi_dcmpgt),
        api!(__aeabi_dcmpun),
        // Single-precision floating-point arithmetic helper functions
        // RTABI chapter 4.1.2, Table 4
        api!(__aeabi_fadd),
        api!(__aeabi_fdiv),
        api!(__aeabi_fmul),
        api!(__aeabi_fsub),
        // Single-precision floating-point comparison helper functions
        // RTABI chapter 4.1.2, Table 5
        api!(__aeabi_fcmpeq),
        api!(__aeabi_fcmpeq),
        api!(__aeabi_fcmplt),
        api!(__aeabi_fcmple),
        api!(__aeabi_fcmpge),
        api!(__aeabi_fcmpgt),
        api!(__aeabi_fcmpun),
        // Floating-point to integer conversions.
        // RTABI chapter 4.1.2, Table 6
        api!(__aeabi_d2iz),
        api!(__aeabi_d2uiz),
        api!(__aeabi_d2lz),
        api!(__aeabi_d2ulz),
        api!(__aeabi_f2iz),
        api!(__aeabi_f2uiz),
        api!(__aeabi_f2lz),
        api!(__aeabi_f2ulz),
        // Conversions between floating types.
        // RTABI chapter 4.1.2, Table 7
        api!(__aeabi_f2d),
        // Integer to floating-point conversions.
        // RTABI chapter 4.1.2, Table 8
        api!(__aeabi_i2d),
        api!(__aeabi_ui2d),
        api!(__aeabi_l2d),
        api!(__aeabi_ul2d),
        api!(__aeabi_i2f),
        api!(__aeabi_ui2f),
        api!(__aeabi_l2f),
        api!(__aeabi_ul2f),
        // Long long helper functions
        // RTABI chapter 4.2, Table 9
        api!(__aeabi_lmul),
        api!(__aeabi_llsl),
        api!(__aeabi_llsr),
        api!(__aeabi_lasr),
        // Integer division functions
        // RTABI chapter 4.3.1
        api!(__aeabi_idiv),
        api!(__aeabi_ldivmod),
        api!(__aeabi_uidiv),
        api!(__aeabi_uldivmod),
        // 4.3.4 Memory copying, clearing, and setting
        api!(__aeabi_memcpy8),
        api!(__aeabi_memcpy4),
        api!(__aeabi_memcpy),
        api!(__aeabi_memmove8),
        api!(__aeabi_memmove4),
        api!(__aeabi_memmove),
        api!(__aeabi_memset8),
        api!(__aeabi_memset4),
        api!(__aeabi_memset),
        api!(__aeabi_memclr8),
        api!(__aeabi_memclr4),
        api!(__aeabi_memclr),

        // libc
        api!(memcmp, extern { fn memcmp(a: *const u8, b: *mut u8, size: usize); }),

        // exceptions
        api!(_Unwind_Resume = unwind::_Unwind_Resume),
        api!(__artiq_personality = eh_artiq::artiq_personality),
        api!(__artiq_raise = eh_artiq::raise),
        api!(__artiq_reraise = eh_artiq::reraise),

        // libm
        api_libm_f64f64!(sqrt),
        api_libm_f64f64!(round),
        api_libm_f64f64!(floor),
        {
            extern fn fmod(x: f64, y: f64) -> f64 {
                libm::fmod(x, y)
            }
            api!(fmod = fmod)
        },
        api_libm_f64f64!(log),
        api_libm_f64f64!(log10),
        api_libm_f64f64!(exp),
        {
            extern fn pow(x: f64, y: f64) -> f64 {
                libm::pow(x, y)
            }
            api!(pow = pow)
        },
        api_libm_f64f64!(sin),
        api_libm_f64f64!(cos),
        api_libm_f64f64!(tan),
        api_libm_f64f64!(asin),
        api_libm_f64f64!(acos),
        api_libm_f64f64!(atan),
    ];
    api.iter()
       .find(|&&(exported, _)| exported.as_bytes() == required)
       .map(|&(_, ptr)| ptr as u32)
}
