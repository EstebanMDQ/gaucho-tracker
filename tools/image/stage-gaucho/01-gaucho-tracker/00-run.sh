#!/bin/bash
set -e

echo "Installing Gaucho Tracker..."

# Enable SPI + TFT
echo "dtparam=spi=on" >> /boot/config.txt
echo "dtoverlay=pitft35-resistive,rotate=90,speed=20000000,fps=25" >> /boot/config.txt

# Enable SSH
touch /boot/ssh

# Setup Wi-Fi
cat > /boot/wpa_supplicant.conf <<EOF
country=US
ctrl_interface=DIR=/var/run/wpa_supplicant GROUP=netdev
update_config=1

network={
    ssid=\"YourSSID\"
    psk=\"YourPassword\"
    key_mgmt=WPA-PSK
}
EOF

# Install gaucho binary
install -m 755 /build/gaucho-tracker /usr/local/bin/gaucho-tracker

# Create systemd unit
cat > /etc/systemd/system/gaucho.service <<EOF
[Unit]
Description=Gaucho Tracker
After=network.target

[Service]
ExecStart=/usr/local/bin/gaucho-tracker
Restart=always
User=pi

[Install]
WantedBy=multi-user.target
EOF

systemctl enable gaucho.service
