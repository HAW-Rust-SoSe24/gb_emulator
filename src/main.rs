#[cfg(test)]
pub mod test_helpers;

pub mod cpu;
pub mod rendering;
pub mod mmu;

use std::{fs::File, io::Write, time};

use cpu::CPU;
use macroquad::{prelude::*, ui::root_ui};
use mmu::MemoryOperations;
use rendering::{
    line_rendering::{self},
    tiles::*,
    views::*,
};
use rfd::FileDialog;
use simple_log::LogConfigBuilder;

extern crate simple_log;

use crate::cpu::registers::{Register16Bit, Register8Bit};

//const TIME_PER_FRAME: f32 = 1000.0 / 60.0;

const DUMP_GAMEBOY_DOCTOR_LOG: bool = false;
#[cfg(target_os = "linux")]
const WINDOWS: bool = false;
#[cfg(target_os = "windows")]
const WINDOWS: bool = true;

#[macroquad::main("GB Emulator")]
async fn main() {
    //Set up logging
    let config = LogConfigBuilder::builder()
        .size(1000)
        .roll_count(10)
        .level("info")
        .output_console()
        .build();
    simple_log::new(config).unwrap();

    const PALETTE: [Color; 4] = [
        Color::new(232.0/255.0, 252.0/255.0, 204.0/255.0, 1.00),
        Color::new(172.0/255.0, 212.0/255.0, 144.0/255.0, 1.00),
        Color::new(084.0/255.0, 140.0/255.0, 112.0/255.0, 1.00),
        Color::new(020.0/255.0, 044.0/255.0, 056.0/255.0, 1.00),
    ];
    const SCALING: f32 = 4.0;

    let mut final_image = Image::gen_image_color(160, 144, GREEN);
    let mut gb_display = GbDisplay {
        offset_x: 5.0,
        offset_y: 5.0,
        scaling: SCALING,
    };
    let gb_display_size = gb_display.size(&final_image);

    let mut background_viewer = BackgroundViewer {
        offset_x: gb_display_size.x + 10.0,
        offset_y: 5.0,
        scaling: SCALING / 2.0,
    };
    let mut background_image = Image::gen_image_color(32 * 8, 32 * 8, PINK);
    let background_viewer_size = background_viewer.size();

    let mut tile_atlas = Image::gen_image_color(8 * 16, 8 * 24, WHITE);
    let mut tile_viewer = TileViewer {
        offset_x: gb_display_size.x + background_viewer_size.x + 15.0,
        offset_y: 5.0,
        scaling: SCALING,
    };
    let tile_viewer_size = tile_viewer.size();

    request_new_screen_size(
        background_viewer_size.x + tile_viewer_size.x + gb_display_size.x + 20.0,
        tile_viewer_size.y + 10.0,
    );

    let filedialog = FileDialog::new()
        .add_filter("gb", &["gb"])
        .set_title("Select a Gameboy ROM")
        // Set directory to the current directory
        .set_directory(std::env::current_dir().unwrap())
        .pick_file()
        .unwrap();

    let filepath = filedialog.as_path().to_str();

    let rom = std::fs::read(filepath.expect("No file was found")).expect("Unable to read file");

    let mut cpu = cpu::CPU::new(rom);
    let mut ppu = line_rendering::Ppu::new();

    // Get start time
    let mut last_frame_time = time::Instant::now();
    let mut ppu_time = time::Instant::now();
    let mut dump_time = time::Instant::now();
    let mut frame = 0;

    // Open "registers.txt" file for Gameboy Doctor
    let mut gb_doctor_file = std::fs::File::create("gameboy_doctor_log.txt").unwrap();
    if DUMP_GAMEBOY_DOCTOR_LOG {
        cpu.skip_boot_rom();
    }

    cpu.set_ppu_mode(cpu::interrupts::PpuMode::OamScan);

    loop {
        // Check whether PC is at the end of the bootrom
        if cpu.get_16bit_register(Register16Bit::PC) == 0x0100 {
            log::info!("🚀 Bootrom finished");
            cpu.skip_boot_rom();
        }

        cpu.increment_div();

        if DUMP_GAMEBOY_DOCTOR_LOG {
            dump_cpu_info(&cpu, &mut gb_doctor_file);
        }

        let instruction = cpu.prepare_and_decode_next_instruction();
        log::debug!("🔠 Instruction: {:?}", instruction);
        let is_bootrom_enabled = cpu.is_boot_rom_enabled();
        let result = cpu.step();
        match result {
            Ok(_) => {}
            Err(e) => {
                log::error!("❌ Error: {:?} | Info: {}", e, info_to_string(&cpu));
                break;
            }
        }
        log::debug!("➡️ Result: {:?} | Bootrom: {:?}", result, is_bootrom_enabled);
        let cpu_cycles_taken = result.unwrap().cycles;

        let pc_following_word = cpu
            .mmu
            .read_word(cpu.get_16bit_register(Register16Bit::PC) + 1);
        log::debug!("🔢 Following Word (PC): {:#06X}", pc_following_word);

        for _ in 0..=cpu_cycles_taken {
            ppu.step(&mut cpu, &mut final_image, &PALETTE);

            
            // Alternatively Redraw UI at 30 frames per second: (ppu_time.elapsed().as_millis() as f32) >= TIME_PER_FRAME
            // Draw when a frame is done
            if ppu.get_frame_cycles() == 0 {
                // Poll inputs
                cpu.poll_inputs();
                cpu.blarg_print();
                ppu_time = time::Instant::now();

                // Inform about the time it took to render the frame
                root_ui().label(
                None,
                format!(
                    "Dots: {:?} | Frame time: {:?} | CPU Cycle: {:?} | Frame: {:?} | Frame Cycle: {:?}",
                    ppu.get_dot(),
                    last_frame_time.elapsed(),
                    cpu.get_cycles(),
                    frame,
                    ppu.get_frame_cycles()
                )
                .as_str(),
                );
                last_frame_time = time::Instant::now();

                // Update Debugging Views
                update_atlas_from_memory(&cpu, 16 * 24, &mut tile_atlas, &PALETTE);
                update_background_from_memory(&cpu, &mut background_image, &PALETTE, false, true);
                background_viewer.draw(&background_image);
                tile_viewer.draw(&tile_atlas);

                gb_display.draw(&final_image);
                next_frame().await;
                frame += 1;
            }
        }
    }
}

fn info_to_string(cpu: &CPU) -> String {
    format!(
        "A:{:02X} F:{:02X} B:{:02X} C:{:02X} D:{:02X} E:{:02X} H:{:02X} L:{:02X} SP:{:04X} PC:{:04X} PCMEM:{:02X},{:02X},{:02X},{:02X}\n",
        cpu.get_8bit_register(Register8Bit::A),
        cpu.flags_to_u8(),
        cpu.get_8bit_register(Register8Bit::B),
        cpu.get_8bit_register(Register8Bit::C),
        cpu.get_8bit_register(Register8Bit::D),
        cpu.get_8bit_register(Register8Bit::E),
        cpu.get_8bit_register(Register8Bit::H),
        cpu.get_8bit_register(Register8Bit::L),
        cpu.get_16bit_register(Register16Bit::SP),
        cpu.get_16bit_register(Register16Bit::PC),
        cpu.mmu.read_byte(cpu.get_16bit_register(Register16Bit::PC)),
        cpu.mmu.read_byte(cpu.get_16bit_register(Register16Bit::PC) + 1),
        cpu.mmu.read_byte(cpu.get_16bit_register(Register16Bit::PC) + 2),
        cpu.mmu.read_byte(cpu.get_16bit_register(Register16Bit::PC) + 3),
    )
}

fn dump_cpu_info(cpu: &CPU, destination: &mut File) {
    // Dump registers to file for Gameboy Doctor like this
    // A:00 F:11 B:22 C:33 D:44 E:55 H:66 L:77 SP:8888 PC:9999 PCMEM:AA,BB,CC,DD
    let _ = destination.write_all(
                    info_to_string(cpu)
                    .as_bytes(),
                );
}
