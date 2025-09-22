from artiq.experiment import *
from artiq.language.types import TBool, TFloat, TInt32, TList
import threading
import time
from sipyco.pc_rpc import simple_server_loop


class FastUrukulMirnyService(EnvExperiment):
    def build(self):
        ###
        ### Init Hardware
        ###
        self.setattr_device("core")

        # Urukul setup
        self.setattr_device("urukul0_cpld")
        self.urukul_list = []
        for i in range(4):
            self.setattr_device(f"urukul0_ch{i}")
            self.urukul_list.append(self.get_device(f"urukul0_ch{i}"))

        # Mirny setup
        self.setattr_device("mirny0_cpld")
        self.mirny_list = []
        for i in range(4):
            self.setattr_device(f"mirny0_ch{i}")
            self.mirny_list.append(self.get_device(f"mirny0_ch{i}"))

        ###
        ### Optimized data structure
        ###
        self._d = {
            "urukul": {
                "freqs": [100.0] * 4,  # MHz
                "amps": [1.0] * 4,  # 0-1
                "rf_on": [False] * 4,  # RF switches
                "changed": False  # Any change flag
            },
            "mirny": {
                "freqs": [1000.0] * 4,  # MHz (will convert from GHz)
                "powers": [0] * 4,  # Power level 0-3
                "atts": [10.0] * 4,  # Attenuation dB
                "rf_on": [False] * 4,  # RF switches
                "changed": False  # Any change flag
            }
        }

        # Cache for change detection
        self._last = {
            "urukul": {
                "freqs": [100.0] * 4,
                "amps": [1.0] * 4,
                "rf_on": [False] * 4
            },
            "mirny": {
                "freqs": [1000.0] * 4,
                "powers": [0] * 4,
                "atts": [10.0] * 4,
                "rf_on": [False] * 4
            }
        }

        self.lock = threading.Lock()
        self.running = True
        self.update_counter = 0

    @kernel
    def kernel_loop(self):
        """Optimized kernel loop for both Urukul and Mirny"""
        self.core.reset()

        # Initialize Urukul
        self.urukul0_cpld.init()
        for i in range(4):
            self.urukul_list[i].init()
            self.urukul_list[i].set_att(10.0 * dB)
            self.urukul_list[i].sw.on()
            self.urukul_list[i].set(100.0 * MHz, amplitude=1.0)

        # Initialize Mirny
        """
        self.mirny0_cpld.init()
        for i in range(4):
            self.mirny_list[i].init()
            self.mirny_list[i].set_att(10.0 * dB)
            self.mirny_list[i].sw.off()
            self.mirny_list[i].set(1000.0 * MHz)
        """

        self.core.break_realtime()

        # Main loop
        while True:
            # Get all data in one RPC call
            urukul_data = self.get_urukul_data()
            mirny_data = self.get_mirny_data()

            # Update Urukul (all 4 channels)
            if urukul_data[4]:  # Check if changed flag is True
                for i in range(4):
                    # Update frequency and amplitude
                    self.urukul_list[i].set_mu(
                        self.urukul_list[i].frequency_to_ftw(urukul_data[0][i] * MHz),
                        asf=self.urukul_list[i].amplitude_to_asf(urukul_data[1][i])
                    )

                    # Update RF switch
                    if urukul_data[2][i]:
                        self.urukul_list[i].sw.on()
                    else:
                        self.urukul_list[i].sw.off()

                # Single IO_UPDATE for all Urukul channels
                self.urukul0_cpld.io_update.pulse(5 * ms)

            """
            # Update Mirny (all 4 channels)
            if mirny_data[5]:  # Check if changed flag is True
                for i in range(4):
                    # Update frequency
                    self.mirny_list[i].set(mirny_data[0][i] * MHz)

                    # Update power level
                    self.mirny_list[i].set_power(mirny_data[1][i])

                    # Update attenuation
                    self.mirny_list[i].set_att(mirny_data[2][i] * dB)

                    # Update RF switch
                    if mirny_data[3][i]:
                        self.mirny_list[i].sw.on()
                    else:
                        self.mirny_list[i].sw.off()

                # IO_UPDATE for Mirny
                self.mirny0_cpld.io_update.pulse(10 * us)
            """

            # Acknowledge update
            self.ack_update()

            # Fixed delay for stable timing
            delay(1 * ms)  # 1kHz update rate

    def get_urukul_data(self) -> TList(TList(TFloat)):
        """Get all Urukul data in one RPC call"""
        with self.lock:
            # Check for changes
            changed = False
            for i in range(4):
                if (abs(self._d["urukul"]["freqs"][i] - self._last["urukul"]["freqs"][i]) > 0.01 or
                        abs(self._d["urukul"]["amps"][i] - self._last["urukul"]["amps"][i]) > 0.001 or
                        self._d["urukul"]["rf_on"][i] != self._last["urukul"]["rf_on"][i]):
                    changed = True
                    break

            # Update last values if changed
            if changed:
                for i in range(4):
                    self._last["urukul"]["freqs"][i] = self._d["urukul"]["freqs"][i]
                    self._last["urukul"]["amps"][i] = self._d["urukul"]["amps"][i]
                    self._last["urukul"]["rf_on"][i] = self._d["urukul"]["rf_on"][i]

            return [
                list(self._d["urukul"]["freqs"]),
                list(self._d["urukul"]["amps"]),
                list(self._d["urukul"]["rf_on"]),
                [],  # Reserved for future use
                changed
            ]

    def get_mirny_data(self) -> TList(TList(TFloat)):
        """Get all Mirny data in one RPC call"""
        with self.lock:
            # Check for changes
            changed = False
            for i in range(4):
                if (abs(self._d["mirny"]["freqs"][i] - self._last["mirny"]["freqs"][i]) > 0.01 or
                        self._d["mirny"]["powers"][i] != self._last["mirny"]["powers"][i] or
                        abs(self._d["mirny"]["atts"][i] - self._last["mirny"]["atts"][i]) > 0.01 or
                        self._d["mirny"]["rf_on"][i] != self._last["mirny"]["rf_on"][i]):
                    changed = True
                    break

            # Update last values if changed
            if changed:
                for i in range(4):
                    self._last["mirny"]["freqs"][i] = self._d["mirny"]["freqs"][i]
                    self._last["mirny"]["powers"][i] = self._d["mirny"]["powers"][i]
                    self._last["mirny"]["atts"][i] = self._d["mirny"]["atts"][i]
                    self._last["mirny"]["rf_on"][i] = self._d["mirny"]["rf_on"][i]

            return [
                list(self._d["mirny"]["freqs"]),
                [float(p) for p in self._d["mirny"]["powers"]],
                list(self._d["mirny"]["atts"]),
                list(self._d["mirny"]["rf_on"]),
                [],  # Reserved for future use
                changed
            ]

    def ack_update(self):
        """Acknowledge update for debugging"""
        self.update_counter += 1
        if self.update_counter % 1000 == 0:
            print(f"Updates: {self.update_counter}")
            self.update_counter = 0

    # ========== URUKUL RPC METHODS ==========
    def set_frequency(self, channel, freq_mhz):
        """Set Urukul frequency in MHz"""
        with self.lock:
            if 0 <= channel < 4:
                self._d["urukul"]["freqs"][channel] = freq_mhz
                return f"Urukul CH{channel}={freq_mhz:.2f}MHz"
        return "Invalid channel"

    def set_amplitude(self, channel, amplitude):
        """Set Urukul amplitude (0-1)"""
        with self.lock:
            if 0 <= channel < 4:
                self._d["urukul"]["amps"][channel] = min(max(amplitude, 0.0), 1.0)
                return f"Urukul CH{channel}={int(amplitude * 100)}%"
        return "Invalid channel"

    def rf_on(self, channel):
        """Turn on Urukul RF"""
        with self.lock:
            if 0 <= channel < 4:
                self._d["urukul"]["rf_on"][channel] = True
                return f"Urukul CH{channel} ON"
        return "Invalid channel"

    def rf_off(self, channel):
        """Turn off Urukul RF"""
        with self.lock:
            if 0 <= channel < 4:
                self._d["urukul"]["rf_on"][channel] = False
                return f"Urukul CH{channel} OFF"
        return "Invalid channel"

    def get_state(self, channel=None):
        """Get Urukul state"""
        with self.lock:
            if channel is not None and 0 <= channel < 4:
                return {
                    "freq_mhz": self._d["urukul"]["freqs"][channel],
                    "amplitude": self._d["urukul"]["amps"][channel],
                    "on": self._d["urukul"]["rf_on"][channel]
                }
            return {
                i: {
                    "freq_mhz": self._d["urukul"]["freqs"][i],
                    "amplitude": self._d["urukul"]["amps"][i],
                    "on": self._d["urukul"]["rf_on"][i]
                } for i in range(4)
            }

    # ========== MIRNY RPC METHODS ==========
    def mirny_set_frequency(self, channel, freq_ghz):
        """Set Mirny frequency in GHz"""
        with self.lock:
            if 0 <= channel < 4:
                self._d["mirny"]["freqs"][channel] = freq_ghz * 1000  # Convert to MHz
                return f"Mirny CH{channel}={freq_ghz:.3f}GHz"
        return "Invalid channel"

    def mirny_set_power(self, channel, power_level):
        """Set Mirny power level (0-3)"""
        with self.lock:
            if 0 <= channel < 4:
                self._d["mirny"]["powers"][channel] = min(max(int(power_level), 0), 3)
                return f"Mirny CH{channel} Power={power_level}"
        return "Invalid channel"

    def mirny_set_attenuation(self, channel, att_db):
        """Set Mirny attenuation in dB"""
        with self.lock:
            if 0 <= channel < 4:
                self._d["mirny"]["atts"][channel] = min(max(att_db, 0.0), 31.5)
                return f"Mirny CH{channel} Att={att_db:.1f}dB"
        return "Invalid channel"

    def mirny_rf_on(self, channel):
        """Turn on Mirny RF"""
        with self.lock:
            if 0 <= channel < 4:
                self._d["mirny"]["rf_on"][channel] = True
                return f"Mirny CH{channel} ON"
        return "Invalid channel"

    def mirny_rf_off(self, channel):
        """Turn off Mirny RF"""
        with self.lock:
            if 0 <= channel < 4:
                self._d["mirny"]["rf_on"][channel] = False
                return f"Mirny CH{channel} OFF"
        return "Invalid channel"

    def mirny_get_state(self, channel=None):
        """Get Mirny state"""
        with self.lock:
            if channel is not None and 0 <= channel < 4:
                return {
                    "freq_ghz": self._d["mirny"]["freqs"][channel] / 1000,  # Convert back to GHz
                    "power": self._d["mirny"]["powers"][channel],
                    "att": self._d["mirny"]["atts"][channel],
                    "on": self._d["mirny"]["rf_on"][channel]
                }
            return {
                i: {
                    "freq_ghz": self._d["mirny"]["freqs"][i] / 1000,
                    "power": self._d["mirny"]["powers"][i],
                    "att": self._d["mirny"]["atts"][i],
                    "on": self._d["mirny"]["rf_on"][i]
                } for i in range(4)
            }

    def rpc_loop(self):
        """RPC server loop"""
        print("=" * 50)
        print("Urukul & Mirny Fast Service")
        print("=" * 50)
        print("Port: 3250")
        print("=" * 50)
        simple_server_loop({"urukul": self}, "0.0.0.0", 3250)

    def run(self):
        """Main run method"""
        print("Starting kernel...")
        thread_kernel = threading.Thread(target=self.kernel_loop)
        thread_kernel.daemon = True
        thread_kernel.start()

        print("Starting RPC server...")
        self.rpc_loop()