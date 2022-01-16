esptool.py --chip esp32 elf2image target/xtensa-esp32-espidf/debug/esp-feed
esptool.py --chip esp32 -p /dev/ttyUSB0 -b 931200 --before=default_reset --after=hard_reset write_flash --flash_mode dio --flash_freq 40m --flash_size 4MB 0x110000 target/xtensa-esp32-espidf/debug
