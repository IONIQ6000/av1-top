================================================================================
  AV1 JANITOR - DEAD SIMPLE INSTALLATION
================================================================================

METHOD 1: Install Script (Recommended)
---------------------------------------
sudo ./install.sh

This builds from source and installs everything automatically.


METHOD 2: Debian Package (.deb file)
--------------------------------------
⚠️  IMPORTANT: .deb files are NOT executable scripts!

Install with:
  sudo dpkg -i av1janitor_0.1.0_amd64.deb
  sudo apt-get install -f  # Install dependencies if needed

DO NOT try to run: ./av1janitor_0.1.0_amd64.deb  ❌


AFTER INSTALLATION (Both Methods):
-----------------------------------
STEP 1: Edit config
  sudo nano /etc/av1janitor/config.toml

  Set your media directories:
    watched_directories = ["/media/movies", "/media/tv"]

STEP 2: Start it
  sudo systemctl start av1janitor

STEP 3: Monitor it
  av1top


DONE! Files are being transcoded to AV1!
========================================

Commands:
  Start:   sudo systemctl start av1janitor
  Stop:    sudo systemctl stop av1janitor
  Status:  sudo systemctl status av1janitor
  Logs:    sudo journalctl -u av1janitor -f
  Monitor: av1top

================================================================================

