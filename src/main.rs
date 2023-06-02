#[macro_use]
extern crate simple_excel_writer;
extern crate plotters;
extern crate ads1x1x;
extern crate i2cdev;

use std::io::{Read, stdin};
use std::{thread, time};
use simple_excel_writer as excel;
use ads1x1x::{channel, Ads1x1x, SlaveAddr, ChannelSelection};
use ads1x1x::Error::{I2C, InvalidInputData};
use ads1x1x::DynamicOneShot;
use linux_embedded_hal::I2cdev;

use plotters::prelude::*;
use plotters::series::*;
use excel::*;
use nb::block;

fn main() {
    println!("Running!");

    let mut buffer = String::new();
    let mut time = 30;
    let mut interval = 0.1;
    let mut multiplier = 1.0;

    loop {
        stdin().read_line(&mut buffer);
        let command: Vec<&str> = buffer.trim_end().split(" ").collect();

        match command.get(0).unwrap() {
            &"time" => {
                time = String::from(*command.get(1).unwrap()).parse::<i16>().unwrap();
                println!("Set recording time to: {}", time) // Sets the total time to record data
            },
            &"interval" => {
                interval = String::from(*command.get(1).unwrap()).parse::<f32>().unwrap();
                println!("Set recording interval to: {}", interval) // Sets the interval to record data at
            },
            &"multiplier" => {
                multiplier = String::from(*command.get(1).unwrap()).parse::<f64>().unwrap();
                println!("Set data multiplier to: {}", multiplier)
            },
            &"start" => record(time, interval, multiplier),
            &"exit" => break,
            &_ => println!("Invalid command!")
        };
        buffer.clear()
    }

    println!("Shutting down!")
}

fn record(time: i16, interval: f32, multiplier: f64) {
    println!("Recording data!");

    let input = read_transducer_input(time, interval);
    let data: Vec<(f64, f64)> = input
        .iter()
        .map(|(b_in, b_out)| {
            (
                voltage_to_pressure(multiplier, *b_in),
                voltage_to_pressure(multiplier, *b_out)
            )
        }).collect();

    plot(time, interval, &data);
    write_to_excel(interval, &data);
    // println!("{:?}", data);
    println!("Finished recording!")
}

fn read_transducer_input(time: i16, interval: f32) -> Vec<(i16, i16)> {
    // Setup the ADS1115
    let dev = I2cdev::new("/dev/i2c-1").unwrap();
    let address = SlaveAddr::default();
    let mut adc = Ads1x1x::new_ads1115(dev, address);

    let mut data = Vec::new();
    let sleep_time = time::Duration::from_secs_f32(interval);
    let mut t = 0.0;

    while t <= time as f32 {
        let v = [
            block!(adc.read(ChannelSelection::SingleA2)).unwrap(),
            block!(adc.read(ChannelSelection::SingleA3)).unwrap()
        ];

        let v_in = v[0];
        let v_out = v[1];

        data.push((v_in, v_out));
        t += interval as f32;
        thread::sleep(sleep_time)
    }

    println!("Finished reading data!");
    return data
}

fn read_test_input(time: i16, interval: f32) -> Vec<(i16, i16)> {
    println!("Reading test data!");
    let mut data = Vec::new();

    let mut t = 0.0;

    while t <= time as f32 {
        let v_fake_in = 4.5;
        let v_fake_out = 0.5;

        data.push(
            ((8000f64*v_fake_in).round() as i16, (8000f64*v_fake_out).round() as i16)
        );
        t += interval
    }

    println!("Finished reading!");
    return data
}

fn voltage_to_pressure(mul: f64, binary: i16) -> f64 {
    // let correction_factor = 68.0/62.0; // Attempt to correct floating point errors
    let v = binary_to_voltage(binary);
    let p_psi: f64 = ((mul * v) - 0.5)*2.5; // Voltage range is 0.5 to 4.5 for 0-10 psi
    let p_kPa: f64 = p_psi * 6.89475729; // Convert

    return p_kPa
}

fn binary_to_voltage(binary: i16) -> f64 {
    return f64::from(binary)/8000.0f64;
}

fn plot(time: i16, interval: f32, pressure_data: &Vec<(f64, f64)>) {
    let root = SVGBackend::new("plot.svg", (640, 480)).into_drawing_area();
    root.fill(&WHITE);
    let root = root.margin(10i32, 10i32, 10i32, 10i32);

    let mut chart = ChartBuilder::on(&root)
        .caption("Pressure data", ("sans-serif", 40).into_font())
        .x_label_area_size(20i32)
        .y_label_area_size(40i32)
        .build_ranged(0f64..f64::from(time), 0f64..70f64);
    let mut chart = chart.unwrap();

    // Then we can draw a mesh
    chart
        .configure_mesh()
        // We can customize the maximum number of labels allowed for each axis
        .x_labels(5)
        .y_labels(5)
        // We can also change the format of the label text
        .y_label_formatter(&|x| format!("{:.3}", x))
        .draw();

    let mut p_in_data = Vec::new();
    let mut p_out_data = Vec::new();

    let mut t = 0f64;
    for (p_in, p_out) in pressure_data {
        p_in_data.push((t, *p_in));
        p_out_data.push((t, *p_out));

        t += interval as f64;
    }

    chart.draw_series(LineSeries::new(
        p_in_data,
        &RED
    )).unwrap();
    chart.draw_series(LineSeries::new(
        p_out_data,
        &BLUE
    )).unwrap();
}

fn write_to_excel(interval: f32, pressure_data: &Vec<(f64, f64)>) {
    let mut wb = Workbook::create("data.xlsx");
    let mut sheet = wb.create_sheet("Trial 1");

    sheet.add_column(Column { width: 30.0 });
    sheet.add_column(Column { width: 30.0 });
    sheet.add_column(Column { width: 30.0 });

    wb.write_sheet(&mut sheet, |sw| {
        let result = Ok(());

        let mut t = 0f64;
        for (p_in, p_out) in pressure_data {
            let result = sw.append_row(row![t, *p_in, *p_out]);
            if result.is_err() { break }
            t += interval as f64;
        };
        return result;
    }).expect("write excel error!");

    wb.close().expect("close excel error!");
}
