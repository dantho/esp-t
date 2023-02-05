// reference:
// https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-reference/system/freertos.html
use std::ptr;

use esp_idf_sys::{
    c_types::c_void, esp, gpio_config, gpio_config_t, gpio_install_isr_service,
    gpio_int_type_t_GPIO_INTR_POSEDGE, gpio_isr_handler_add, gpio_mode_t_GPIO_MODE_INPUT,
    xQueueGenericCreate, xQueueGiveFromISR, xQueueReceive, QueueHandle_t,ESP_INTR_FLAG_IRAM, esp_random,
};

// These imports are needed for part 2.
use esp32_c3_dkc02_bsc as bsc;
use bsc::led::{RGB8, WS2812RMT};

// 4. Create a `static mut` that holds the queue handle.
static mut EVENT_QUEUE: Option<QueueHandle_t> = None;

// 6. Define what the interrupt handler does, once the button is pushed. Button_interrupt sends a message into the queue. 
#[link_section = ".iram0.text"]
unsafe extern "C" fn button_interrupt(_: *mut c_void) {
    xQueueGiveFromISR(EVENT_QUEUE.unwrap(), std::ptr::null_mut());
}

fn main() -> anyhow::Result<()> {
    const GPIO_NUM: i32 = 9;

    // 1. Add GPIO configuration c struct
    // let io_conf = gpio_config_t {
    //     ...
    // };
    let io_conf: gpio_config_t = gpio_config_t {
        pin_bit_mask: 1<<GPIO_NUM,
        mode: gpio_mode_t_GPIO_MODE_INPUT,
        pull_up_en: true as u32,
        pull_down_en: false as u32,
        intr_type: gpio_int_type_t_GPIO_INTR_POSEDGE,
    };
    unsafe {

        // 2. write the GPIO configuration into the register
        // esp!(...)?;
        esp!(gpio_config(&io_conf))?;

        // 3. Install the global GPIO interrupt handler
        // esp!(...)?;
        esp!(gpio_install_isr_service(ESP_INTR_FLAG_IRAM as i32))?;

        // Queue configurations
        const QUEUE_TYPE_BASE: u8 = 0;
        const ITEM_SIZE: u32 = 0; 
        const QUEUE_SIZE: u32 = 1;

        // 5. Create an event queue
        // EVENT_QUEUE = Some(...);
        EVENT_QUEUE = Some(xQueueGenericCreate(QUEUE_SIZE, ITEM_SIZE, QUEUE_TYPE_BASE));
                
        // 7. Add the button GPIO and the function to the interrupt handler
        // esp!(...)?;
        esp!(gpio_isr_handler_add(
            GPIO_NUM,
            Some(button_interrupt),
            std::ptr::null_mut()
        ))?;
    }
    let mut led = bsc::led::WS2812RMT::new()?;
    led.set_pixel(RGB8::new(0,0,0))?;

    // The loop in main waits until it gets a message through the rx ("receiver") part of the channel
    loop {
        unsafe {
            // maximum delay
            const QUEUE_WAIT_TICKS: u32 = 1000;;

            // 8. Receive the event from the queue.
            // let res = ...;
            let res = xQueueReceive(EVENT_QUEUE.unwrap(), ptr::null_mut(), QUEUE_WAIT_TICKS);

            // 9. Handle the value of res.
            // ...
            // If the event has the value 0, nothing happens. if it has a different value, the button was pressed. 
            match res {
                1 => {
                    let dimmer = 15;
                    let rgb = RGB8::new(
                        (esp_random() & 0xFF) as u8 / dimmer,
                        (esp_random() & 0xFF) as u8 / dimmer,
                        (esp_random() & 0xFF) as u8 / dimmer,
                    );
                    led.set_pixel(rgb)?;
                    println!("LED color is {:?}", rgb);
                },
                _ => {},
            };
        }
    }
}
