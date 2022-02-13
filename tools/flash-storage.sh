python3 /home/derek/Git/esp-idf/components/fatfs/wl_fatfsgen.py --output_file storage.img --partition_size 1048576 storage
esptool.py --chip esp32 -p /dev/ttyACM0 -b 115200 --before=default_reset --after=hard_reset write_flash --flash_mode dio --flash_freq 40m --flash_size 4MB 0x10000 storage.img
