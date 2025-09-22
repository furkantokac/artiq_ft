# ft_service.py
# Optimized Urukul DDS Server with Fast Response

from artiq.experiment import *
from artiq.language.units import MHz, ms, us
from sipyco.pc_rpc import simple_server_loop


class UrukulServer(EnvExperiment):
    """
    High-performance Urukul DDS Server
    Provides RPC interface for real-time DDS control
    """

    def build(self):
        """Build the experiment with required devices"""
        self.setattr_device("core")
        self.setattr_device("urukul0_cpld")
        self.setattr_device("urukul0_ch0")
        self.setattr_device("urukul0_ch1")
        self.setattr_device("urukul0_ch2")
        self.setattr_device("urukul0_ch3")

    def prepare(self):
        """Prepare experiment and initialize hardware"""
        # Initialize hardware on core device
        self.init_hardware()

        # Current state tracking (in Hz for frequency)
        self.current_freq = [100e6, 100e6, 100e6, 100e6]
        self.current_amp = [1.0, 1.0, 1.0, 1.0]
        self.current_sw = [False, False, False, False]

        # Channel objects for fast access
        self.channels = [
            self.urukul0_ch0,
            self.urukul0_ch1,
            self.urukul0_ch2,
            self.urukul0_ch3
        ]

    @kernel
    def init_hardware(self):
        """Initialize all hardware components"""
        self.core.reset()

        # Initialize CPLD
        self.urukul0_cpld.init()
        delay(10 * ms)

        # Initialize all DDS channels
        self.urukul0_ch0.init()
        self.urukul0_ch1.init()
        self.urukul0_ch2.init()
        self.urukul0_ch3.init()

        # Set default parameters (100 MHz, full amplitude, off)
        self.urukul0_ch0.set(100 * MHz, amplitude=1.0)
        self.urukul0_ch0.sw.off()

        self.urukul0_ch1.set(100 * MHz, amplitude=1.0)
        self.urukul0_ch1.sw.off()

        self.urukul0_ch2.set(100 * MHz, amplitude=1.0)
        self.urukul0_ch2.sw.off()

        self.urukul0_ch3.set(100 * MHz, amplitude=1.0)
        self.urukul0_ch3.sw.off()

    @kernel
    def fast_update_ch0(self, freq_hz, amp):
        """Fast update for channel 0 - optimized for speed"""
        self.core.break_realtime()
        self.urukul0_ch0.set(freq_hz, amplitude=amp)

    @kernel
    def fast_update_ch1(self, freq_hz, amp):
        """Fast update for channel 1 - optimized for speed"""
        self.core.break_realtime()
        self.urukul0_ch1.set(freq_hz, amplitude=amp)

    @kernel
    def fast_update_ch2(self, freq_hz, amp):
        """Fast update for channel 2 - optimized for speed"""
        self.core.break_realtime()
        self.urukul0_ch2.set(freq_hz, amplitude=amp)

    @kernel
    def fast_update_ch3(self, freq_hz, amp):
        """Fast update for channel 3 - optimized for speed"""
        self.core.break_realtime()
        self.urukul0_ch3.set(freq_hz, amplitude=amp)

    @kernel
    def batch_update_all(self, freq0, freq1, freq2, freq3,
                         amp0, amp1, amp2, amp3):
        """Update all channels at once for maximum efficiency"""
        self.core.break_realtime()

        # Update all parameters in one kernel execution
        self.urukul0_ch0.set(freq0, amplitude=amp0)
        self.urukul0_ch1.set(freq1, amplitude=amp1)
        self.urukul0_ch2.set(freq2, amplitude=amp2)
        self.urukul0_ch3.set(freq3, amplitude=amp3)

    @kernel
    def switch_ch0_on(self):
        """Turn on channel 0 RF switch"""
        self.core.break_realtime()
        self.urukul0_ch0.sw.on()

    @kernel
    def switch_ch0_off(self):
        """Turn off channel 0 RF switch"""
        self.core.break_realtime()
        self.urukul0_ch0.sw.off()

    @kernel
    def switch_ch1_on(self):
        """Turn on channel 1 RF switch"""
        self.core.break_realtime()
        self.urukul0_ch1.sw.on()

    @kernel
    def switch_ch1_off(self):
        """Turn off channel 1 RF switch"""
        self.core.break_realtime()
        self.urukul0_ch1.sw.off()

    @kernel
    def switch_ch2_on(self):
        """Turn on channel 2 RF switch"""
        self.core.break_realtime()
        self.urukul0_ch2.sw.on()

    @kernel
    def switch_ch2_off(self):
        """Turn off channel 2 RF switch"""
        self.core.break_realtime()
        self.urukul0_ch2.sw.off()

    @kernel
    def switch_ch3_on(self):
        """Turn on channel 3 RF switch"""
        self.core.break_realtime()
        self.urukul0_ch3.sw.on()

    @kernel
    def switch_ch3_off(self):
        """Turn off channel 3 RF switch"""
        self.core.break_realtime()
        self.urukul0_ch3.sw.off()

    @kernel
    def set_switch_states(self, sw0, sw1, sw2, sw3):
        """Set all switch states at once"""
        self.core.break_realtime()

        if sw0:
            self.urukul0_ch0.sw.on()
        else:
            self.urukul0_ch0.sw.off()

        if sw1:
            self.urukul0_ch1.sw.on()
        else:
            self.urukul0_ch1.sw.off()

        if sw2:
            self.urukul0_ch2.sw.on()
        else:
            self.urukul0_ch2.sw.off()

        if sw3:
            self.urukul0_ch3.sw.on()
        else:
            self.urukul0_ch3.sw.off()

    # ==================== RPC Methods ====================

    def set_frequency(self, channel, freq_mhz):
        """
        Set frequency for specified channel
        Args:
            channel: Channel number (0-3)
            freq_mhz: Frequency in MHz
        Returns:
            Status string
        """
        if channel < 0 or channel > 3:
            return f"Invalid channel: {channel}"

        freq_hz = freq_mhz * 1e6
        self.current_freq[channel] = freq_hz

        # Use fast update methods
        if channel == 0:
            self.fast_update_ch0(freq_hz, self.current_amp[0])
        elif channel == 1:
            self.fast_update_ch1(freq_hz, self.current_amp[1])
        elif channel == 2:
            self.fast_update_ch2(freq_hz, self.current_amp[2])
        else:
            self.fast_update_ch3(freq_hz, self.current_amp[3])

        return f"CH{channel}: {freq_mhz:.2f} MHz"

    def set_amplitude(self, channel, amplitude):
        """
        Set amplitude for specified channel
        Args:
            channel: Channel number (0-3)
            amplitude: Amplitude (0.0 to 1.0)
        Returns:
            Status string
        """
        if channel < 0 or channel > 3:
            return f"Invalid channel: {channel}"

        # Clamp amplitude to valid range
        amplitude = max(0.0, min(1.0, amplitude))
        self.current_amp[channel] = amplitude

        # Use fast update methods
        if channel == 0:
            self.fast_update_ch0(self.current_freq[0], amplitude)
        elif channel == 1:
            self.fast_update_ch1(self.current_freq[1], amplitude)
        elif channel == 2:
            self.fast_update_ch2(self.current_freq[2], amplitude)
        else:
            self.fast_update_ch3(self.current_freq[3], amplitude)

        return f"CH{channel}: {int(amplitude * 100)}%"

    def rf_on(self, channel):
        """
        Turn on RF output for specified channel
        Args:
            channel: Channel number (0-3)
        Returns:
            Status string
        """
        if channel < 0 or channel > 3:
            return f"Invalid channel: {channel}"

        self.current_sw[channel] = True

        if channel == 0:
            self.switch_ch0_on()
        elif channel == 1:
            self.switch_ch1_on()
        elif channel == 2:
            self.switch_ch2_on()
        else:
            self.switch_ch3_on()

        return f"CH{channel}: ON"

    def rf_off(self, channel):
        """
        Turn off RF output for specified channel
        Args:
            channel: Channel number (0-3)
        Returns:
            Status string
        """
        if channel < 0 or channel > 3:
            return f"Invalid channel: {channel}"

        self.current_sw[channel] = False

        if channel == 0:
            self.switch_ch0_off()
        elif channel == 1:
            self.switch_ch1_off()
        elif channel == 2:
            self.switch_ch2_off()
        else:
            self.switch_ch3_off()

        return f"CH{channel}: OFF"

    def set_channel_params(self, channel, freq_mhz, amplitude, rf_on):
        """
        Set all parameters for a channel at once
        Args:
            channel: Channel number (0-3)
            freq_mhz: Frequency in MHz
            amplitude: Amplitude (0.0 to 1.0)
            rf_on: RF switch state (True/False)
        Returns:
            Status dictionary
        """
        if channel < 0 or channel > 3:
            return {"error": f"Invalid channel: {channel}"}

        freq_hz = freq_mhz * 1e6
        amplitude = max(0.0, min(1.0, amplitude))

        # Update stored values
        self.current_freq[channel] = freq_hz
        self.current_amp[channel] = amplitude
        self.current_sw[channel] = rf_on

        # Apply updates
        if channel == 0:
            self.fast_update_ch0(freq_hz, amplitude)
            if rf_on:
                self.switch_ch0_on()
            else:
                self.switch_ch0_off()
        elif channel == 1:
            self.fast_update_ch1(freq_hz, amplitude)
            if rf_on:
                self.switch_ch1_on()
            else:
                self.switch_ch1_off()
        elif channel == 2:
            self.fast_update_ch2(freq_hz, amplitude)
            if rf_on:
                self.switch_ch2_on()
            else:
                self.switch_ch2_off()
        else:
            self.fast_update_ch3(freq_hz, amplitude)
            if rf_on:
                self.switch_ch3_on()
            else:
                self.switch_ch3_off()

        return {
            "channel": channel,
            "freq_mhz": freq_mhz,
            "amplitude": amplitude,
            "rf_on": rf_on,
            "status": "OK"
        }

    def update_all_channels(self, params):
        """
        Update all channels at once for maximum efficiency
        Args:
            params: Dictionary with channel parameters
                   {0: {"freq_mhz": f, "amplitude": a, "rf_on": b}, ...}
        Returns:
            Status dictionary
        """
        # Prepare values for batch update
        for ch in range(4):
            if ch in params:
                p = params[ch]
                if "freq_mhz" in p:
                    self.current_freq[ch] = p["freq_mhz"] * 1e6
                if "amplitude" in p:
                    self.current_amp[ch] = max(0.0, min(1.0, p["amplitude"]))
                if "rf_on" in p:
                    self.current_sw[ch] = p["rf_on"]

        # Batch update frequencies and amplitudes
        self.batch_update_all(
            self.current_freq[0], self.current_freq[1],
            self.current_freq[2], self.current_freq[3],
            self.current_amp[0], self.current_amp[1],
            self.current_amp[2], self.current_amp[3]
        )

        # Update switch states
        self.set_switch_states(
            self.current_sw[0], self.current_sw[1],
            self.current_sw[2], self.current_sw[3]
        )

        return {"status": "All channels updated", "channels": self.get_state()}

    def get_state(self, channel=None):
        """
        Get current state of one or all channels
        Args:
            channel: Channel number (0-3) or None for all
        Returns:
            State dictionary
        """
        if channel is not None:
            if channel < 0 or channel > 3:
                return {"error": f"Invalid channel: {channel}"}

            return {
                "freq_mhz": self.current_freq[channel] / 1e6,
                "amplitude": self.current_amp[channel],
                "on": self.current_sw[channel]
            }

        # Return all channels
        return {
            i: {
                "freq_mhz": self.current_freq[i] / 1e6,
                "amplitude": self.current_amp[i],
                "on": self.current_sw[i]
            } for i in range(4)
        }

    def ping(self):
        """Simple ping method for connection testing"""
        return "pong"

    def get_info(self):
        """Get server information"""
        return {
            "name": "Urukul DDS Server",
            "version": "2.0",
            "channels": 4,
            "features": ["fast_update", "batch_update", "individual_control"],
            "state": self.get_state()
        }

    def run(self):
        """Run the RPC server"""
        print("=" * 60)
        print("Urukul DDS Server v2.0 - Optimized for Speed")
        print("=" * 60)
        print("Server Address: 0.0.0.0:3250")
        print("RPC Target: urukul")
        print("")
        print("Features:")
        print("  - Fast single-channel updates")
        print("  - Batch update for all channels")
        print("  - Minimal kernel overhead")
        print("  - State tracking and query")
        print("")
        print("Channels: 4 (0-3)")
        print("Frequency Range: 1-400 MHz")
        print("Amplitude Range: 0-100%")
        print("=" * 60)
        print("Server started. Waiting for connections...")
        print("")

        try:
            simple_server_loop({"urukul": self}, "0.0.0.0", 3250)
        except KeyboardInterrupt:
            print("\nServer stopped by user")
        except Exception as e:
            print(f"\nServer error: {e}")
