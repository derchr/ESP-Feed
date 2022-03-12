//! Helper module to make use of a button interrupt.

use esp_idf_hal::interrupt::Mutex;
use std::sync::Arc;

pub fn add_pin_interrupt(pin: i32, handler: impl FnMut() + 'static) {
    unsafe {
        esp_idf_sys::gpio_set_intr_type(pin, esp_idf_sys::gpio_int_type_t_GPIO_INTR_ANYEDGE);
        esp_idf_sys::gpio_install_isr_service(0);
    }

    fn add_isr_handler(pin: i32, f: impl FnMut() + 'static) {
        extern "C" fn isr_handler(arg: *mut std::ffi::c_void) {
            let closure: &mut Box<dyn FnMut()> = unsafe { &mut *(arg as *mut Box<dyn FnMut()>) };
            closure();
        }

        let cb: Box<Box<dyn FnMut()>> = Box::new(Box::new(f));
        unsafe {
            // Note: This leaks the closure, but it's fine as it
            // has to live until the end of the program anyways.
            esp_idf_sys::gpio_isr_handler_add(pin, Some(isr_handler), Box::into_raw(cb) as *mut _);
        }
    }

    add_isr_handler(pin, handler);
}

pub fn button_debounce_handler() -> (impl FnMut() + 'static, Arc<Mutex<u8>>) {
    let button_state = Arc::new(esp_idf_hal::interrupt::Mutex::new(0u8));
    let button_state_cloned = Arc::clone(&button_state);
    let mut last_tick = 0;
    let mut last_pressed = false;

    let closure = move || {
        let mut pressed = button_state_cloned.lock();

        // Assume tick rate is 100 Hz. Wait for at least 5 ticks (50ms).
        // TODO: Get the real tick rate from the config variable.
        let current_tick = unsafe { esp_idf_sys::xTaskGetTickCountFromISR() };
        if current_tick - last_tick < 5 {
            return;
        } else {
            last_tick = current_tick;
        }

        if last_pressed {
            last_pressed = false;
            return;
        } else {
            last_pressed = true;
        }

        *pressed += 1;
    };

    (closure, button_state)
}

pub fn register_button_interrupt(pin: i32) -> Arc<Mutex<u8>> {
    let (button1_handler, button1_state) = button_debounce_handler();
    add_pin_interrupt(pin, button1_handler);
    button1_state
}
