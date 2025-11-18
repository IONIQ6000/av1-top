================================================================================
  AV1 JANITOR - DEAD SIMPLE INSTALLATION
================================================================================

STEP 1: Run the installer
--------------------------
sudo ./install.sh


STEP 2: Edit config
-------------------
sudo nano /etc/av1janitor/config.toml

Set your media directories:
  watched_directories = ["/media/movies", "/media/tv"]


STEP 3: Start it
----------------
sudo systemctl start av1janitor


STEP 4: Monitor it
------------------
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

