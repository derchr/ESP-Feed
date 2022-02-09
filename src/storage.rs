use esp_idf_sys::wl_handle_t;
use std::{ffi::CString, fs::File, io::Read};

pub const BASE_DIR: &str = "/mnt";

pub struct StorageHandle {
    wl_handle: wl_handle_t,
    base_path: CString,
}

impl StorageHandle {
    pub fn new() -> Self {
        let base_path = CString::new(BASE_DIR).expect("Invalid CString.");
        let partition_label = CString::new("storage").expect("Invalid CString.");

        let fat_cfg = esp_idf_sys::esp_vfs_fat_mount_config_t {
            max_files: 4,
            format_if_mount_failed: false,
            ..Default::default()
        };
        let mut wl_handle: esp_idf_sys::wl_handle_t = 0;

        unsafe {
            esp_idf_sys::esp_vfs_fat_spiflash_mount(
                base_path.as_ptr(),
                partition_label.as_ptr(),
                &fat_cfg as *const _,
                &mut wl_handle as *mut _,
            );
        }

        Self {
            wl_handle,
            base_path,
        }
    }
}

impl Default for StorageHandle {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for StorageHandle {
    fn drop(&mut self) {
        unsafe {
            esp_idf_sys::esp_vfs_fat_spiflash_unmount(self.base_path.as_ptr(), self.wl_handle);
        }
    }
}

pub trait ReadFile {
    fn raw_bytes(&mut self) -> Vec<u8>;
}

impl ReadFile for File {
    fn raw_bytes(&mut self) -> Vec<u8> {
        let mut out = Vec::new();
        let mut buf = [0u8; 256];

        while {
            let len = self.read(&mut buf).unwrap();
            out.extend_from_slice(&buf[..len]);
            len != 0
        } {}

        out
    }
}
