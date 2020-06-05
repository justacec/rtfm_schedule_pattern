#![no_std]
#![no_main]

extern crate cortex_m_rt as rt;
extern crate panic_itm;
extern crate byte;

use cortex_m::asm;
use cortex_m::iprintln;
use core::ops::{Deref, DerefMut};

use rtfm::app;
use rtfm::cyccnt::{U32Ext};
use cortex_m::peripheral::{ITM};
use num_derive::FromPrimitive;
use core::ptr;

#[app(device = stm32f4xx_hal::stm32, peripherals = true, monotonic = rtfm::cyccnt::CYCCNT)]
const APP: () = {
    struct Resources {
        #[init(false)]
        motor_a_update_state: bool,
        #[init(false)]
        motor_a_schedule: bool,
        #[init(0)]
        motor_a_run_count: u32,
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
    #[task(schedule = [motor_a_update_scheduler], spawn = [motor_a_update], resources = [motor_a_update_state, motor_a_schedule])]
    fn motor_a_update_scheduler(cx: motor_a_update_scheduler::Context) {
        let frequency: f32 = 60.0;

        *cx.resources.motor_a_schedule = true;

       cx.spawn.motor_a_update().unwrap();

       if cx.resources.motor_a.state != MotorState::Stop {
            let PERIOD = ((100_000_000 as f32) / frequency) as u32;
            cx.schedule.motor_a_update_scheduler(cx.scheduled + PERIOD.cycles()).unwrap();
        } else {
            *cx.resources.motor_a_schedule = false;
        }
    }

    #[task(schedule = [motor_a_update], spawn = [motor_a_update_scheduler, motor_a_worker], capacity=10, resources = [motor_a_run_count, motor_a_schedule, itm])]
    fn motor_a_update(cx: motor_a_update::Context) {
        *cx.resources.motor_a_run_count = *cx.resources.motor_a_run_count + 1;

        // If the scheduler is running, just run once
        if *cx.resources.motor_a_schedule == false {
            cx.spawn.motor_a_update_scheduler().unwrap();
        } else {
            if cx.resources.motor_a_update_state == false {
                cx.spawn.motor_a_worker().unwrap();
            }    
        }
    }

    #[task(schedule = [motor_a_update], capacity=10, resources = [motor_a, motor_a_update_state, motor_a_run_count, itm])]
    fn motor_a_worker(cx: motor_a_worker::Context) {
        if *cx.resources.motor_a_update_state == true {
            return
        }

        *cx.resources.motor_a_update_state = true;

        while(*cx.resources.motor_a_run_count > 0) {
            for i in 0..1000 {
                asm::nop();
            }
        }

        *cx.resources.motor_a_update_state = false;
    }

    extern "C" {
        fn USART1();
    }

};
