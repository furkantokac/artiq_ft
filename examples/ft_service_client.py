# ft_service_client.py
import sys
from PyQt5.QtWidgets import *
from PyQt5.QtCore import *
from sipyco.pc_rpc import Client


class DDSControlGUI(QMainWindow):
    def __init__(self):
        super().__init__()
        self.client = None
        self.connect_to_server()
        self.init_ui()

    def connect_to_server(self):
        """Connect to RPC server"""
        try:
            self.client = Client("localhost", 3250, "urukul", timeout=5)
            print("Connected to server successfully")
        except Exception as e:
            print(f"Connection failed: {e}")
            self.client = None

    def init_ui(self):
        """Initialize UI with tabs for Urukul and Mirny"""
        self.setWindowTitle('DDS Control - Urukul & Mirny')

        # Main widget with tabs
        central = QWidget(self)
        self.setCentralWidget(central)
        main_layout = QVBoxLayout()

        # Connection status
        self.status_label = QLabel("Status: " + ("Connected" if self.client else "Disconnected"))
        main_layout.addWidget(self.status_label)

        # Tab widget
        tabs = QTabWidget(self)

        # Urukul tab
        urukul_tab = QWidget(self)
        urukul_layout = QVBoxLayout()
        self.setup_urukul_tab(urukul_layout)
        urukul_tab.setLayout(urukul_layout)
        tabs.addTab(urukul_tab, "Urukul (MHz)")

        # Mirny tab
        mirny_tab = QWidget(self)
        mirny_layout = QVBoxLayout()
        self.setup_mirny_tab(mirny_layout)
        mirny_tab.setLayout(mirny_layout)
        tabs.addTab(mirny_tab, "Mirny (GHz)")

        main_layout.addWidget(tabs)

        # Debug output area
        main_layout.addWidget(QLabel("Server Messages:"))
        self.debug_text = QTextEdit()
        self.debug_text.setMaximumHeight(100)
        self.debug_text.setReadOnly(True)
        main_layout.addWidget(self.debug_text)

        central.setLayout(main_layout)

        # Initial state refresh
        if self.client:
            self.refresh_urukul_state()
            self.refresh_mirny_state()

    def setup_urukul_tab(self, layout):
        """Setup Urukul controls"""
        # All channels control buttons
        all_controls = QHBoxLayout(self)

        all_on_btn = QPushButton("All Urukul ON")
        all_on_btn.clicked.connect(self.all_urukul_on)
        all_controls.addWidget(all_on_btn)

        all_off_btn = QPushButton("All Urukul OFF")
        all_off_btn.clicked.connect(self.all_urukul_off)
        all_controls.addWidget(all_off_btn)

        refresh_btn = QPushButton("Refresh")
        refresh_btn.clicked.connect(self.refresh_urukul_state)
        all_controls.addWidget(refresh_btn)

        layout.addLayout(all_controls)

        # Create 4 channel controls
        self.urukul_channels = []
        for ch in range(4):
            group = QGroupBox(f"Urukul Channel {ch}")
            ch_layout = QGridLayout()

            # Frequency slider (1-400 MHz)
            ch_layout.addWidget(QLabel("Frequency:"), 0, 0)

            freq_slider = QSlider(Qt.Horizontal)
            freq_slider.setRange(1, 400)
            freq_slider.setValue(100)
            freq_slider.setTickPosition(QSlider.TicksBelow)
            freq_slider.setTickInterval(50)

            freq_label = QLabel("100 MHz")
            freq_label.setMinimumWidth(80)

            #freq_slider.valueChanged.connect(
            #    lambda v, l=freq_label: l.setText(f"{v} MHz")
            #)
            freq_slider.valueChanged.connect(
                lambda c=ch, s=freq_slider: self.urukul_changed(c, s.value())
            )
            freq_slider.sliderReleased.connect(
                lambda c=ch, s=freq_slider: self.urukul_freq_released(c, s.value())
            )

            ch_layout.addWidget(freq_slider, 0, 1)
            ch_layout.addWidget(freq_label, 0, 2)

            # Amplitude slider (0-100%)
            ch_layout.addWidget(QLabel("Amplitude:"), 1, 0)

            amp_slider = QSlider(Qt.Horizontal)
            amp_slider.setRange(0, 100)
            amp_slider.setValue(100)
            amp_slider.setTickPosition(QSlider.TicksBelow)
            amp_slider.setTickInterval(25)

            amp_label = QLabel("100%")
            amp_label.setMinimumWidth(80)

            amp_slider.valueChanged.connect(
                lambda v, l=amp_label: l.setText(f"{v}%")
            )
            amp_slider.sliderReleased.connect(
                lambda c=ch, s=amp_slider: self.urukul_amp_released(c, s.value())
            )

            ch_layout.addWidget(amp_slider, 1, 1)
            ch_layout.addWidget(amp_label, 1, 2)

            # RF on/off button
            rf_btn = QPushButton("RF OFF")
            rf_btn.setCheckable(True)
            rf_btn.clicked.connect(
                lambda checked, c=ch, b=rf_btn: self.toggle_urukul_rf(c, checked, b)
            )

            ch_layout.addWidget(rf_btn, 2, 0, 1, 3)

            # Status label
            status_label = QLabel("Ready")
            ch_layout.addWidget(status_label, 3, 0, 1, 3)

            group.setLayout(ch_layout)
            layout.addWidget(group)

            self.urukul_channels.append({
                'freq_slider': freq_slider,
                'freq_label': freq_label,
                'amp_slider': amp_slider,
                'amp_label': amp_label,
                'rf_btn': rf_btn,
                'status': status_label
            })

    def urukul_changed(self, channel, value):
        self.client.set_frequency(0, value)
        return
        if not self.client:
            return
        try:
            result = self.client.set_frequency(channel, value)
            self.urukul_channels[channel]['status'].setText(result)
            self.debug_text.append(result)
        except Exception as e:
            self.urukul_channels[channel]['status'].setText(f"Error: {str(e)}")

    def setup_mirny_tab(self, layout):
        """Setup Mirny controls"""
        # All channels control buttons
        all_controls = QHBoxLayout()

        all_on_btn = QPushButton("All Mirny ON")
        all_on_btn.clicked.connect(self.all_mirny_on)
        all_controls.addWidget(all_on_btn)

        all_off_btn = QPushButton("All Mirny OFF")
        all_off_btn.clicked.connect(self.all_mirny_off)
        all_controls.addWidget(all_off_btn)

        refresh_btn = QPushButton("Refresh")
        refresh_btn.clicked.connect(self.refresh_mirny_state)
        all_controls.addWidget(refresh_btn)

        layout.addLayout(all_controls)

        # Create 4 channel controls
        self.mirny_channels = []
        for ch in range(4):
            group = QGroupBox(f"Mirny Channel {ch}")
            ch_layout = QGridLayout()

            # Frequency input (0.053-13.6 GHz)
            ch_layout.addWidget(QLabel("Frequency (GHz):"), 0, 0)

            freq_spin = QDoubleSpinBox()
            freq_spin.setRange(0.053, 13.6)
            freq_spin.setValue(1.0)
            freq_spin.setDecimals(3)
            freq_spin.setSingleStep(0.1)
            freq_spin.setSuffix(" GHz")

            freq_spin.valueChanged.connect(
                lambda v, c=ch: self.mirny_freq_changed(c, v)
            )

            ch_layout.addWidget(freq_spin, 0, 1, 1, 2)

            # Power selection (0-3)
            ch_layout.addWidget(QLabel("Power Level:"), 1, 0)

            power_combo = QComboBox()
            power_combo.addItems(["0 (Min)", "1", "2", "3 (Max)"])
            power_combo.setCurrentIndex(0)

            power_combo.currentIndexChanged.connect(
                lambda idx, c=ch: self.mirny_power_changed(c, idx)
            )

            ch_layout.addWidget(power_combo, 1, 1, 1, 2)

            # Attenuation slider (0-31.5 dB)
            ch_layout.addWidget(QLabel("Attenuation:"), 2, 0)

            att_slider = QSlider(Qt.Horizontal)
            att_slider.setRange(0, 315)  # 0-31.5 dB in 0.1 dB steps
            att_slider.setValue(100)  # 10 dB
            att_slider.setTickPosition(QSlider.TicksBelow)
            att_slider.setTickInterval(50)

            att_label = QLabel("10.0 dB")
            att_label.setMinimumWidth(80)

            att_slider.valueChanged.connect(
                lambda v, l=att_label: l.setText(f"{v / 10:.1f} dB")
            )
            att_slider.sliderReleased.connect(
                lambda c=ch, s=att_slider: self.mirny_att_released(c, s.value() / 10)
            )

            ch_layout.addWidget(att_slider, 2, 1)
            ch_layout.addWidget(att_label, 2, 2)

            # RF on/off button
            rf_btn = QPushButton("RF OFF")
            rf_btn.setCheckable(True)
            rf_btn.clicked.connect(
                lambda checked, c=ch, b=rf_btn: self.toggle_mirny_rf(c, checked, b)
            )

            ch_layout.addWidget(rf_btn, 3, 0, 1, 3)

            # Status label
            status_label = QLabel("Ready")
            ch_layout.addWidget(status_label, 4, 0, 1, 3)

            group.setLayout(ch_layout)
            layout.addWidget(group)

            self.mirny_channels.append({
                'freq_spin': freq_spin,
                'power_combo': power_combo,
                'att_slider': att_slider,
                'att_label': att_label,
                'rf_btn': rf_btn,
                'status': status_label
            })

    # ========== URUKUL METHODS ==========
    def urukul_freq_released(self, channel, value):
        if not self.client:
            return
        try:
            result = self.client.set_frequency(channel, value)
            self.urukul_channels[channel]['status'].setText(result)
            self.debug_text.append(result)
        except Exception as e:
            self.urukul_channels[channel]['status'].setText(f"Error: {str(e)}")

    def urukul_amp_released(self, channel, value):
        if not self.client:
            return
        try:
            result = self.client.set_amplitude(channel, value / 100.0)
            self.urukul_channels[channel]['status'].setText(result)
            self.debug_text.append(result)
        except Exception as e:
            self.urukul_channels[channel]['status'].setText(f"Error: {str(e)}")

    def toggle_urukul_rf(self, channel, checked, button):
        if not self.client:
            return
        try:
            if checked:
                result = self.client.rf_on(channel)
                button.setText("RF ON")
            else:
                result = self.client.rf_off(channel)
                button.setText("RF OFF")
            self.urukul_channels[channel]['status'].setText(result)
            self.debug_text.append(result)
        except Exception as e:
            self.urukul_channels[channel]['status'].setText(f"Error: {str(e)}")

    def all_urukul_on(self):
        for ch in range(4):
            try:
                result = self.client.rf_on(ch)
                self.urukul_channels[ch]['rf_btn'].setChecked(True)
                self.urukul_channels[ch]['rf_btn'].setText("RF ON")
                self.urukul_channels[ch]['status'].setText(result)
            except Exception as e:
                self.debug_text.append(f"Error CH{ch}: {str(e)}")

    def all_urukul_off(self):
        for ch in range(4):
            try:
                result = self.client.rf_off(ch)
                self.urukul_channels[ch]['rf_btn'].setChecked(False)
                self.urukul_channels[ch]['rf_btn'].setText("RF OFF")
                self.urukul_channels[ch]['status'].setText(result)
            except Exception as e:
                self.debug_text.append(f"Error CH{ch}: {str(e)}")

    def refresh_urukul_state(self):
        if not self.client:
            return
        try:
            state = self.client.get_state()
            for ch in range(4):
                if ch in state:
                    ch_state = state[ch]
                    # Update UI
                    self.urukul_channels[ch]['freq_slider'].setValue(int(ch_state['freq_mhz']))
                    self.urukul_channels[ch]['freq_label'].setText(f"{int(ch_state['freq_mhz'])} MHz")
                    self.urukul_channels[ch]['amp_slider'].setValue(int(ch_state['amplitude'] * 100))
                    self.urukul_channels[ch]['amp_label'].setText(f"{int(ch_state['amplitude'] * 100)}%")
                    self.urukul_channels[ch]['rf_btn'].setChecked(ch_state['on'])
                    self.urukul_channels[ch]['rf_btn'].setText("RF ON" if ch_state['on'] else "RF OFF")
        except Exception as e:
            self.debug_text.append(f"Error refreshing Urukul state: {str(e)}")

    # ========== MIRNY METHODS ==========
    def mirny_freq_changed(self, channel, value):
        if not self.client:
            return
        try:
            result = self.client.mirny_set_frequency(channel, value)
            self.mirny_channels[channel]['status'].setText(result)
            self.debug_text.append(result)
        except Exception as e:
            self.mirny_channels[channel]['status'].setText(f"Error: {str(e)}")

    def mirny_power_changed(self, channel, value):
        if not self.client:
            return
        try:
            result = self.client.mirny_set_power(channel, value)
            self.mirny_channels[channel]['status'].setText(result)
            self.debug_text.append(result)
        except Exception as e:
            self.mirny_channels[channel]['status'].setText(f"Error: {str(e)}")

    def mirny_att_released(self, channel, value):
        if not self.client:
            return
        try:
            result = self.client.mirny_set_attenuation(channel, value)
            self.mirny_channels[channel]['status'].setText(result)
            self.debug_text.append(result)
        except Exception as e:
            self.mirny_channels[channel]['status'].setText(f"Error: {str(e)}")

    def toggle_mirny_rf(self, channel, checked, button):
        if not self.client:
            return
        try:
            if checked:
                result = self.client.mirny_rf_on(channel)
                button.setText("RF ON")
            else:
                result = self.client.mirny_rf_off(channel)
                button.setText("RF OFF")
            self.mirny_channels[channel]['status'].setText(result)
            self.debug_text.append(result)
        except Exception as e:
            self.mirny_channels[channel]['status'].setText(f"Error: {str(e)}")

    def all_mirny_on(self):
        for ch in range(4):
            try:
                result = self.client.mirny_rf_on(ch)
                self.mirny_channels[ch]['rf_btn'].setChecked(True)
                self.mirny_channels[ch]['rf_btn'].setText("RF ON")
                self.mirny_channels[ch]['status'].setText(result)
            except Exception as e:
                self.debug_text.append(f"Error CH{ch}: {str(e)}")

    def all_mirny_off(self):
        for ch in range(4):
            try:
                result = self.client.mirny_rf_off(ch)
                self.mirny_channels[ch]['rf_btn'].setChecked(False)
                self.mirny_channels[ch]['rf_btn'].setText("RF OFF")
                self.mirny_channels[ch]['status'].setText(result)
            except Exception as e:
                self.debug_text.append(f"Error CH{ch}: {str(e)}")

    def refresh_mirny_state(self):
        if not self.client:
            return
        try:
            state = self.client.mirny_get_state()
            for ch in range(4):
                if ch in state:
                    ch_state = state[ch]
                    # Update UI
                    self.mirny_channels[ch]['freq_spin'].setValue(ch_state['freq_ghz'])
                    self.mirny_channels[ch]['power_combo'].setCurrentIndex(ch_state['power'])
                    self.mirny_channels[ch]['att_slider'].setValue(int(ch_state['att'] * 10))
                    self.mirny_channels[ch]['att_label'].setText(f"{ch_state['att']:.1f} dB")
                    self.mirny_channels[ch]['rf_btn'].setChecked(ch_state['on'])
                    self.mirny_channels[ch]['rf_btn'].setText("RF ON" if ch_state['on'] else "RF OFF")
        except Exception as e:
            self.debug_text.append(f"Error refreshing Mirny state: {str(e)}")

    def closeEvent(self, event):
        """Clean up on close"""
        if self.client:
            try:
                self.client.close_rpc()
            except:
                pass
        event.accept()


if __name__ == '__main__':
    app = QApplication(sys.argv)
    window = DDSControlGUI()
    window.show()
    sys.exit(app.exec_())