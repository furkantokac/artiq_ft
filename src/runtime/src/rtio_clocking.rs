use log::{info, warn};
use libboard_zynq::timer::GlobalTimer;
use embedded_hal::blocking::delay::DelayMs;
use libconfig::Config;
use libboard_artiq::pl;
#[cfg(has_si5324)]
use libboard_zynq::i2c::I2c;
#[cfg(has_si5324)]
use crate::i2c;
#[cfg(has_si5324)]
use libboard_artiq::si5324;

#[derive(Debug, PartialEq, Copy, Clone)]
#[allow(non_camel_case_types)]
pub enum RtioClock {
    Default,
    Int_125,
    Int_100,
    Int_150,
    Ext0_Bypass,
    Ext0_Synth0_10to125,
    Ext0_Synth0_100to125,
    Ext0_Synth0_125to125,
}

#[allow(unreachable_code)]
fn get_rtio_clock_cfg(cfg: &Config) -> RtioClock {
    let mut res = RtioClock::Default;
    if let Ok(clk) = cfg.read_str("rtio_clock") {
        res = match clk.as_ref() {
            "int_125" => RtioClock::Int_125,
            "int_100" => RtioClock::Int_100,
            "int_150" => RtioClock::Int_150,
            "ext0_bypass" => RtioClock::Ext0_Bypass,
            "ext0_bypass_125" => RtioClock::Ext0_Bypass,
            "ext0_bypass_100" => RtioClock::Ext0_Bypass,
            "ext0_synth0_10to125" => RtioClock::Ext0_Synth0_10to125,
            "ext0_synth0_100to125" => RtioClock::Ext0_Synth0_100to125,
            "ext0_synth0_125to125" => RtioClock::Ext0_Synth0_125to125,
            _ => { 
                warn!("Unrecognised rtio_clock setting. Falling back to default.");
                RtioClock::Default 
            }
        };
    }
    else {
        warn!("error reading configuration. Falling back to default.");
    }
    if res == RtioClock::Default {
        #[cfg(rtio_frequency="100.0")]
        {
            warn!("Using default configuration - internal 100MHz RTIO clock.");
            return RtioClock::Int_100;
        }
        #[cfg(rtio_frequency="125.0")]
        {
            warn!("Using default configuration - internal 125MHz RTIO clock.");
            return RtioClock::Int_125;
        }
        // anything else
        {
            warn!("Using default configuration - internal 125MHz RTIO clock.");
            return RtioClock::Int_125;
        }
    }
    res
}


fn init_rtio(timer: &mut GlobalTimer, _clk: RtioClock) {
    #[cfg(has_rtio_crg_clock_sel)]
    let clock_sel = match _clk {
        RtioClock::Ext0_Bypass => { 
            info!("Using bypassed external clock");
            1
        },
        RtioClock::Int_125 => {
            info!("Using internal RTIO clock");
            0
        },
        _ => {
            warn!("rtio_clock setting '{:?}' is not supported. Using default internal RTIO clock instead", _clk);
            0
        }
    };

    loop {
        unsafe {
            pl::csr::rtio_crg::pll_reset_write(1);
            #[cfg(has_rtio_crg_clock_sel)]
            pl::csr::rtio_crg::clock_sel_write(clock_sel);
            pl::csr::rtio_crg::pll_reset_write(0);
        }
        timer.delay_ms(1);
        let locked = unsafe { pl::csr::rtio_crg::pll_locked_read() != 0 };
        if locked {
            info!("RTIO PLL locked");
            break;
        } else {
            warn!("RTIO PLL failed to lock, retrying...");
            timer.delay_ms(500);
        }
    }

    unsafe {
        pl::csr::rtio_core::reset_phy_write(1);
    }
}

#[cfg(has_drtio)]
fn init_drtio(timer: &mut GlobalTimer)
{
    unsafe {
        pl::csr::drtio_transceiver::stable_clkin_write(1);
    }
    timer.delay_ms(2); // wait for CPLL/QPLL lock
    unsafe {
        pl::csr::drtio_transceiver::txenable_write(0xffffffffu32 as _);
    }
}

#[cfg(has_si5324)]
fn setup_si5324(i2c: &mut I2c, timer: &mut GlobalTimer, clk: RtioClock) {
    let si5324_settings = match clk {
        RtioClock::Ext0_Synth0_10to125 => { // 125 MHz output from 10 MHz CLKINx reference, 504 Hz BW
            info!("using 10MHz reference to make 125MHz RTIO clock with PLL");
            si5324::FrequencySettings {
                n1_hs  : 10,
                nc1_ls : 4,
                n2_hs  : 10,
                n2_ls  : 300,
                n31    : 6,
                n32    : 6,
                bwsel  : 4,
                crystal_ref: false
            }
        },
        RtioClock::Ext0_Synth0_100to125 => { // 125MHz output, from 100MHz CLKINx reference, 586 Hz loop bandwidth
            info!("using 100MHz reference to make 125MHz RTIO clock with PLL");
            si5324::FrequencySettings {
                n1_hs  : 10,
                nc1_ls : 4,
                n2_hs  : 10,
                n2_ls  : 260,
                n31    : 52,
                n32    : 52,
                bwsel  : 4,
                crystal_ref: false
            }
        },
        RtioClock::Ext0_Synth0_125to125 => { // 125MHz output, from 125MHz CLKINx reference, 606 Hz loop bandwidth
            info!("using 125MHz reference to make 125MHz RTIO clock with PLL");
            si5324::FrequencySettings {
                n1_hs  : 5,
                nc1_ls : 8,
                n2_hs  : 7,
                n2_ls  : 360,
                n31    : 63,
                n32    : 63,
                bwsel  : 4,
                crystal_ref: false
            }
        },
        RtioClock::Int_150 => { // 150MHz output, from crystal
            info!("using internal 150MHz RTIO clock");
            si5324::FrequencySettings {
                n1_hs  : 9,
                nc1_ls : 4,
                n2_hs  : 10,
                n2_ls  : 33732,
                n31    : 7139,
                n32    : 7139,
                bwsel  : 3,
                crystal_ref: true
            }
        },
        RtioClock::Int_100 => { // 100MHz output, from crystal.
            info!("using internal 100MHz RTIO clock");
            si5324::FrequencySettings {
                n1_hs  : 9,
                nc1_ls : 6,
                n2_hs  : 10,
                n2_ls  : 33732,
                n31    : 7139,
                n32    : 7139,
                bwsel  : 3,
                crystal_ref: true
            }
        },
        RtioClock::Int_125 => { // 125MHz output, from crystal, 7 Hz
            info!("using internal 125MHz RTIO clock");
            si5324::FrequencySettings {
                n1_hs  : 10,
                nc1_ls : 4,
                n2_hs  : 10,
                n2_ls  : 19972,
                n31    : 4565,
                n32    : 4565,
                bwsel  : 4,
                crystal_ref: true
            }
        }
        _ => { // same setting  as Int_125, but fallback to default
            warn!("rtio_clock setting '{:?}' is unsupported. Falling back to default internal 125MHz RTIO clock.", clk);
            si5324::FrequencySettings {
                n1_hs  : 10,
                nc1_ls : 4,
                n2_hs  : 10,
                n2_ls  : 19972,
                n31    : 4565,
                n32    : 4565,
                bwsel  : 4,
                crystal_ref: true
            }
        }
    };
    let si5324_ref_input = si5324::Input::Ckin2;
    si5324::setup(i2c, &si5324_settings, si5324_ref_input, timer).expect("cannot initialize Si5324");
}

pub fn init(timer: &mut GlobalTimer, cfg: &Config) {

    let clk = get_rtio_clock_cfg(cfg);
    #[cfg(has_si5324)]
    {
        let i2c = unsafe { (&mut i2c::I2C_BUS).as_mut().unwrap() };
        let si5324_ext_input = si5324::Input::Ckin2;
        match clk {
            RtioClock::Ext0_Bypass => si5324::bypass(i2c, si5324_ext_input, timer).expect("cannot bypass Si5324"),
            _ => setup_si5324(i2c, timer, clk),
        }
    }
    #[cfg(has_drtio)]
    init_drtio(timer);

    init_rtio(timer, clk);

}