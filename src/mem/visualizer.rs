//! Visualize a memory region of a certain zoom around a certain address. This is useful for debugging and understanding the memory layout of the machine,
//! and especially to make obvious the impacts of overflowing buffer overflows and other memory errors.

use std::sync::{Arc, Mutex};

use crate::mem::{MEMORY, Memory};
use crate::process::Signal;
use colored::Colorize;

pub struct ColorMap {
    pub reserved: (u8, u8, u8),
    pub allocated: (u8, u8, u8), // Allocated if a process wrote to it, reserved if the page is allocated but for some reason the process didn't write to it (e.g. it's the zero page or something)
    pub free: (u8, u8, u8),      // Free if the page is not allocated at all
}

impl ColorMap {
    pub fn new() -> Self {
        Self {
            reserved: (255, 0, 0),  // Red for reserved
            allocated: (0, 255, 0), // Green for allocated
            free: (255, 255, 255),  // White for free
        }
    }
}

pub struct Visualizer {
    pub mem: Arc<Mutex<Memory>>,
    pub zoom: usize,
    pub center: usize,
    pub color_map: ColorMap,
}

impl Visualizer {
    pub fn new(zoom: usize, center: usize) -> Self {
        Self {
            mem: MEMORY.clone(),
            zoom,
            center,
            color_map: ColorMap::new(),
        }
    }

    pub fn visualize_text(&self) -> Signal {
        let start = self.center.saturating_sub(self.zoom);
        let end = self.center.saturating_add(self.zoom);
        println!(
            "Memory visualization (centered at address {}):",
            self.center
        );
        for addr in start..=end {
            if let Ok(byte) = self.mem.lock().unwrap()._read_at(addr as u32, 1) {
                let byte = byte[0];
                println!("Address {}: {}", addr, byte);
            } else {
                println!("Address {}: [Invalid]", addr);
            }
        }
        Signal::Debug
    }

    pub fn visualize_art(&self) -> Signal {
        let start = self.center.saturating_sub(self.zoom);
        let end = self.center.saturating_add(self.zoom);
        println!(
            "Memory visualization (centered at address {}):",
            self.center
        );
        // We print a header with the addresses for better readability
        // We print the addresses in a fixed width format, with a vertical bar as a separator
        print!("|"); // Initial padding for the left border
        for addr in start..=end {
            print!(" {:#x} |", addr); // Print the last 3 digits of the address for readability
        }
        println!(); // Newline after the header
        print!("|"); // Left border
        for addr in start..=end {
            if let Ok(_) = self.mem.lock().unwrap()._read_at(addr as u32, 1) {
                // let byte = byte[0]; // We just read one byte
                // For simplicity, we just print the byte as a colored block. In a real implementation, we would use a more sophisticated visualization.
                // We start with checking if the byte is involved in any allocation
                let color = if self.mem.lock().unwrap().is_byte_reserved(addr as u32) {
                    self.color_map.reserved
                } else if self.mem.lock().unwrap().is_byte_allocated(addr as u32) {
                    self.color_map.allocated
                } else {
                    self.color_map.free
                };
                print!("{}", "   █|   ".truecolor(color.0, color.1, color.2));
            } else {
                print!(
                    "{}|",
                    "█".truecolor(
                        self.color_map.reserved.0,
                        self.color_map.reserved.1,
                        self.color_map.reserved.2
                    )
                );
            }
        }
        println!(); // Newline after the visualization
        Signal::Debug
    }
}
