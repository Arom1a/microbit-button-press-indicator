#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_futures::select::select;
use embassy_nrf::gpio::{Input, Level, Output, OutputDrive, Pull};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, signal::Signal};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

static LEFT_ARR: [[u8; 5]; 5] = [
    [0, 0, 0, 0, 0],
    [0, 1, 0, 0, 0],
    [1, 1, 1, 1, 1],
    [0, 1, 0, 0, 0],
    [0, 0, 0, 0, 0],
];

static RIGHT_ARR: [[u8; 5]; 5] = [
    [0, 0, 0, 0, 0],
    [0, 0, 0, 1, 0],
    [1, 1, 1, 1, 1],
    [0, 0, 0, 1, 0],
    [0, 0, 0, 0, 0],
];

static DOUBLE_ARR: [[u8; 5]; 5] = [
    [0, 0, 0, 0, 0],
    [0, 1, 0, 1, 0],
    [1, 1, 1, 1, 1],
    [0, 1, 0, 1, 0],
    [0, 0, 0, 0, 0],
];

static EMPTY: [[u8; 5]; 5] = [
    [0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0],
];

#[derive(defmt::Format)]
enum ButtonState {
    Both,
    Left,
    Right,
    None,
}

static SIGNAL: Signal<ThreadModeRawMutex, ButtonState> = Signal::new();

async fn draw_matrix(
    rows: &mut [Output<'static>; 5],
    cols: &mut [Output<'static>; 5],
    pattern: &[[u8; 5]; 5],
    frequency: Duration,
) {
    for (r, row) in rows.iter_mut().enumerate() {
        row.set_high();

        for (c, col) in cols.iter_mut().enumerate() {
            if pattern[r][c] == 1 {
                col.set_low();
            } else {
                col.set_high();
            }
        }

        // Timer::after_micros(2000).await;
        Timer::after(frequency).await;
        row.set_low();
    }
}

#[embassy_executor::task]
async fn button_task(mut button_a: Input<'static>, mut button_b: Input<'static>) {
    loop {
        let left_pressed = button_a.is_low();
        let right_pressed = button_b.is_low();

        let state = match (left_pressed, right_pressed) {
            (true, true) => {
                info!("both pressed!");
                ButtonState::Both
            }
            (true, false) => {
                info!("left pressed!");
                ButtonState::Left
            }
            (false, true) => {
                info!("right pressed!");
                ButtonState::Right
            }
            (false, false) => {
                info!("none pressed!");
                ButtonState::None
            }
        };

        SIGNAL.signal(state);

        // Timer::after_millis(50).await;
        select(button_a.wait_for_any_edge(), button_b.wait_for_any_edge()).await;
    }
}

#[embassy_executor::task]
async fn display_task(mut rows: [Output<'static>; 5], mut cols: [Output<'static>; 5]) {
    let mut curr_state = ButtonState::None;
    loop {
        if let Some(state) = SIGNAL.try_take() {
            curr_state = state;
            info!("curr_state: {:?}", curr_state);
        }

        let pattern = match curr_state {
            ButtonState::Both => DOUBLE_ARR,
            ButtonState::Left => LEFT_ARR,
            ButtonState::Right => RIGHT_ARR,
            ButtonState::None => EMPTY,
        };
        draw_matrix(&mut rows, &mut cols, &pattern, Duration::from_millis(2)).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    let button_a = Input::new(p.P0_14, Pull::None);
    let button_b = Input::new(p.P0_23, Pull::None);

    let rows = [
        Output::new(p.P0_21, Level::Low, OutputDrive::Standard),
        Output::new(p.P0_22, Level::Low, OutputDrive::Standard),
        Output::new(p.P0_15, Level::Low, OutputDrive::Standard),
        Output::new(p.P0_24, Level::Low, OutputDrive::Standard),
        Output::new(p.P0_19, Level::Low, OutputDrive::Standard),
    ];
    let cols = [
        Output::new(p.P0_28, Level::Low, OutputDrive::Standard),
        Output::new(p.P0_11, Level::Low, OutputDrive::Standard),
        Output::new(p.P0_31, Level::Low, OutputDrive::Standard),
        Output::new(p.P1_05, Level::Low, OutputDrive::Standard),
        Output::new(p.P0_30, Level::Low, OutputDrive::Standard),
    ];

    spawner.spawn(button_task(button_a, button_b)).unwrap();
    spawner.spawn(display_task(rows, cols)).unwrap();
}
