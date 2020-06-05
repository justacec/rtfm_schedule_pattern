#![no_std]
#![no_main]

extern crate cortex_m_rt as rt;
extern crate panic_itm;

use cortex_m::asm;
use rtfm::app;
use rtfm::cyccnt::{U32Ext};
use stm32f4xx_hal::{prelude::*, stm32};

#[derive(PartialEq)]
enum State {
    Stop,
    SomethingElse
}

pub struct my_object {
    state: State 
}

#[app(device = stm32f4xx_hal::stm32, peripherals = true, monotonic = rtfm::cyccnt::CYCCNT)]
const APP: () = {
    struct Resources {
        object: my_object,
        #[init(false)]
        object_update_state: bool,
        #[init(false)]
        object_schedule: bool,
        #[init(0)]
        object_run_count: u32,
    }

    #[init()]
    fn init(cx: init::Context) -> init::LateResources {

        // Cortex-M peripherals
        let mut _core: rtfm::Peripherals = cx.core;

        // Device specific peripherals
        let mut _device: stm32::Peripherals = cx.device;

        _device.RCC.apb2enr.write(|w| w.syscfgen().enabled()); 

        // Specify and fix the clock speeds
        let rcc = _device.RCC.constrain();

        let _clocks = rcc.cfgr
            .use_hse(25.mhz())
            .sysclk(100.mhz())
            .freeze();


        // Initialize (enable) the monotonic timer (CYCCNT)
        _core.DCB.enable_trace();
        _core.DWT.enable_cycle_counter();

        init::LateResources {
            object: my_object{ state: State::Stop }
        }
    }

    #[idle()]
    fn idle(_cx: idle::Context) -> ! {
        loop {
            asm::nop();
        }
    }

    // This functions will be responsible for directly controlling the motor.
    // It will look at the current status and the target status and then configure the
    // motor accordingly
    #[task(schedule = [object_update_scheduler], spawn = [object_update], resources = [object, object_update_state, object_schedule])]
    fn object_update_scheduler(cx: object_update_scheduler::Context) {
        let frequency: f32 = 60.0;

        *cx.resources.object_schedule = true;

       cx.spawn.object_update().unwrap();

       if cx.resources.object.state != State::Stop {
            let PERIOD = ((100_000_000 as f32) / frequency) as u32;
            cx.schedule.object_update_scheduler(cx.scheduled + PERIOD.cycles()).unwrap();
        } else {
            *cx.resources.object_schedule = false;
        }
    }

    #[task(spawn = [object_update_scheduler, object_worker], capacity=10, resources = [object_update_state, object_run_count, object_schedule])]
    fn object_update(cx: object_update::Context) {
        *cx.resources.object_run_count = *cx.resources.object_run_count + 1;

        // If the scheduler is running, just run once
        if *cx.resources.object_schedule == false {
            cx.spawn.object_update_scheduler().unwrap();
        } else {
            if *cx.resources.object_update_state == false {
                cx.spawn.object_worker().unwrap();
            }    
        }
    }

    #[task(capacity=10, resources = [object_update_state, object_run_count])]
    fn object_worker(cx: object_worker::Context) {
        if *cx.resources.object_update_state == true {
            return
        }

        *cx.resources.object_update_state = true;

        while *cx.resources.object_run_count > 0 {
            for i in 0..1000 {
                asm::nop();
            }
        }

        *cx.resources.object_update_state = false;
    }

    extern "C" {
        fn USART1();
    }

};
