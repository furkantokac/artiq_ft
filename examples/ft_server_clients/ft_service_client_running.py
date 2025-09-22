# ft_service_client.py
# Simple Urukul DDS Client with Sliders

import sys
from PyQt5.QtWidgets import *
from PyQt5.QtCore import *
from sipyco.pc_rpc import Client


class UrukulClient(QMainWindow):
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
        """Initialize UI with sliders"""
        self.setWindowTitle('Urukul DDS Control')

        # Main widget
        central = QWidget()
        self.setCentralWidget(central)
        main_layout = QVBoxLayout()

        # Connection status
        self.status_label = QLabel("Status: " + ("Connected" if self.client else "Disconnected"))
        main_layout.addWidget(self.status_label)

        # All channels control buttons
        all_controls = QHBoxLayout()

        all_on_btn = QPushButton("All Channels ON")
        all_on_btn.clicked.connect(self.all_channels_on)
        all_controls.addWidget(all_on_btn)

        all_off_btn = QPushButton("All Channels OFF")
        all_off_btn.clicked.connect(self.all_channels_off)
        all_controls.addWidget(all_off_btn)

        refresh_btn = QPushButton("Refresh State")
        refresh_btn.clicked.connect(self.refresh_state)
        all_controls.addWidget(refresh_btn)

        main_layout.addLayout(all_controls)

        # Separator
        line = QFrame()
        line.setFrameShape(QFrame.HLine)
        main_layout.addWidget(line)

        # Create 4 channel controls
        self.channels = []
        for ch in range(4):
            # Channel group
            group = QGroupBox(f"Channel {ch}")
            ch_layout = QGridLayout()

            # Frequency slider
            ch_layout.addWidget(QLabel("Frequency:"), 0, 0)

            freq_slider = QSlider(Qt.Horizontal)
            freq_slider.setRange(1, 400)
            freq_slider.setValue(100)
            freq_slider.setTickPosition(QSlider.TicksBelow)
            freq_slider.setTickInterval(50)

            freq_label = QLabel("100 MHz")
            freq_label.setMinimumWidth(80)

            # Connect slider to update
            freq_slider.valueChanged.connect(
                lambda v, c=ch, l=freq_label: l.setText(f"{v} MHz")
            )

            freq_slider.sliderReleased.connect(
                lambda c=ch, s=freq_slider: self.freq_released(c, s.value())
            )

            ch_layout.addWidget(freq_slider, 0, 1)
            ch_layout.addWidget(freq_label, 0, 2)

            # Amplitude slider
            ch_layout.addWidget(QLabel("Amplitude:"), 1, 0)

            amp_slider = QSlider(Qt.Horizontal)
            amp_slider.setRange(0, 100)
            amp_slider.setValue(100)
            amp_slider.setTickPosition(QSlider.TicksBelow)
            amp_slider.setTickInterval(25)

            amp_label = QLabel("100%")
            amp_label.setMinimumWidth(80)

            # Connect slider to update
            amp_slider.valueChanged.connect(
                lambda v, c=ch, l=amp_label: l.setText(f"{v}%")
            )

            amp_slider.sliderReleased.connect(
                lambda c=ch, s=amp_slider: self.amp_released(c, s.value())
            )

            ch_layout.addWidget(amp_slider, 1, 1)
            ch_layout.addWidget(amp_label, 1, 2)

            # RF on/off button
            rf_btn = QPushButton("RF OFF")
            rf_btn.setCheckable(True)
            rf_btn.clicked.connect(lambda checked, c=ch, b=rf_btn: self.toggle_rf(c, checked, b))

            ch_layout.addWidget(rf_btn, 2, 0, 1, 3)

            # Status label
            status_label = QLabel("Ready")
            ch_layout.addWidget(status_label, 3, 0, 1, 3)

            group.setLayout(ch_layout)
            main_layout.addWidget(group)

            # Store references
            self.channels.append({
                'freq_slider': freq_slider,
                'freq_label': freq_label,
                'amp_slider': amp_slider,
                'amp_label': amp_label,
                'rf_btn': rf_btn,
                'status': status_label
            })

        # Debug output area
        main_layout.addWidget(QLabel("Server Messages:"))
        self.debug_text = QTextEdit()
        self.debug_text.setMaximumHeight(100)
        self.debug_text.setReadOnly(True)
        main_layout.addWidget(self.debug_text)

        central.setLayout(main_layout)

        # Initial state refresh
        if self.client:
            self.refresh_state()

    def freq_released(self, channel, value):
        """Slider bırakıldığında çağrılır"""
        if not self.client:
            return
        try:
            result = self.client.set_frequency(channel, value)
            self.channels[channel]['status'].setText(result)
        except Exception as e:
            self.channels[channel]['status'].setText(f"Error: {str(e)}")

    def amp_released(self, channel, value):
        """Slider bırakıldığında çağrılır"""
        if not self.client:
            return
        try:
            result = self.client.set_amplitude(channel, value / 100.0)
            self.channels[channel]['status'].setText(result)
        except Exception as e:
            self.channels[channel]['status'].setText(f"Error: {str(e)}")

    def freq_changed(self, channel, value, label):
        """Handle frequency slider change"""
        label.setText(f"{value} MHz")

        if not self.client:
            self.channels[channel]['status'].setText("Not connected")
            return

        try:
            result = self.client.set_frequency(channel, value)
            self.channels[channel]['status'].setText(result)
            self.debug_text.append(f"Set freq CH{channel}: {value} MHz -> {result}")
        except Exception as e:
            self.channels[channel]['status'].setText(f"Error: {str(e)}")
            self.debug_text.append(f"Error CH{channel}: {str(e)}")

    def amp_changed(self, channel, value, label):
        """Handle amplitude slider change"""
        label.setText(f"{value}%")

        if not self.client:
            self.channels[channel]['status'].setText("Not connected")
            return

        try:
            amplitude = value / 100.0
            result = self.client.set_amplitude(channel, amplitude)
            self.channels[channel]['status'].setText(result)
            self.debug_text.append(f"Set amp CH{channel}: {value}% -> {result}")
        except Exception as e:
            self.channels[channel]['status'].setText(f"Error: {str(e)}")
            self.debug_text.append(f"Error CH{channel}: {str(e)}")

    def toggle_rf(self, channel, checked, button):
        """Handle RF on/off toggle"""
        if not self.client:
            self.channels[channel]['status'].setText("Not connected")
            return

        try:
            if checked:
                result = self.client.rf_on(channel)
                button.setText("RF ON")
            else:
                result = self.client.rf_off(channel)
                button.setText("RF OFF")

            self.channels[channel]['status'].setText(result)
            self.debug_text.append(f"RF CH{channel}: {'ON' if checked else 'OFF'} -> {result}")
        except Exception as e:
            self.channels[channel]['status'].setText(f"Error: {str(e)}")
            self.debug_text.append(f"Error CH{channel}: {str(e)}")

    def all_channels_on(self):
        """Turn all channels ON"""
        if not self.client:
            self.debug_text.append("Not connected to server")
            return

        self.debug_text.append("Turning all channels ON...")
        for ch in range(4):
            try:
                result = self.client.rf_on(ch)
                self.channels[ch]['rf_btn'].setChecked(True)
                self.channels[ch]['rf_btn'].setText("RF ON")
                self.channels[ch]['status'].setText(result)
            except Exception as e:
                self.debug_text.append(f"Error CH{ch}: {str(e)}")

    def all_channels_off(self):
        """Turn all channels OFF"""
        if not self.client:
            self.debug_text.append("Not connected to server")
            return

        self.debug_text.append("Turning all channels OFF...")
        for ch in range(4):
            try:
                result = self.client.rf_off(ch)
                self.channels[ch]['rf_btn'].setChecked(False)
                self.channels[ch]['rf_btn'].setText("RF OFF")
                self.channels[ch]['status'].setText(result)
            except Exception as e:
                self.debug_text.append(f"Error CH{ch}: {str(e)}")

    def refresh_state(self):
        """Get current state from server and update UI"""
        if not self.client:
            self.debug_text.append("Not connected to server")
            return

        try:
            state = self.client.get_state()
            self.debug_text.append("Refreshing state from server...")

            for ch in range(4):
                if ch in state:
                    ch_state = state[ch]

                    # Update frequency
                    freq_mhz = int(ch_state['freq_mhz'])
                    self.channels[ch]['freq_slider'].blockSignals(True)
                    self.channels[ch]['freq_slider'].setValue(freq_mhz)
                    self.channels[ch]['freq_label'].setText(f"{freq_mhz} MHz")
                    self.channels[ch]['freq_slider'].blockSignals(False)

                    # Update amplitude
                    amp_percent = int(ch_state['amplitude'] * 100)
                    self.channels[ch]['amp_slider'].blockSignals(True)
                    self.channels[ch]['amp_slider'].setValue(amp_percent)
                    self.channels[ch]['amp_label'].setText(f"{amp_percent}%")
                    self.channels[ch]['amp_slider'].blockSignals(False)

                    # Update RF state
                    self.channels[ch]['rf_btn'].blockSignals(True)
                    self.channels[ch]['rf_btn'].setChecked(ch_state['on'])
                    self.channels[ch]['rf_btn'].setText("RF ON" if ch_state['on'] else "RF OFF")
                    self.channels[ch]['rf_btn'].blockSignals(False)

                    self.channels[ch]['status'].setText("State refreshed")

            self.debug_text.append(f"State: {state}")

        except Exception as e:
            self.debug_text.append(f"Error refreshing state: {str(e)}")

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
    window = UrukulClient()
    window.show()
    sys.exit(app.exec_())
