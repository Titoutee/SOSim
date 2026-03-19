use crate::mem::{
    Memory, PHYS_TOTAL, Stack,
    config::MEM_CTXT,
    paging::{Flag, FrameAllocator, PageTableEntry},
};
use std::fmt;

impl fmt::Display for Stack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "├────────────────────────── Stack ──────────────────────────┤"
        )?;
        writeln!(f, "│ Base:           0x{:08x}", self.base)?;
        writeln!(f, "│ Size:           0x{:08x} ({} bytes)", self.sz, self.sz)?;
        writeln!(
            f,
            "│ Capacity:       0x{:08x} ({} bytes)",
            self.cap, self.cap
        )?;
        writeln!(f, "│ Stack Pointer:  0x{:08x}", self.sp)?;
        writeln!(
            f,
            "│ Used:     {} / {} bytes",
            self.sp - self.base,
            self.cap
        )?;

        let usage_percent = if self.cap > 0 {
            ((self.sp - self.base) as f64 / self.cap as f64) * 100.0
        } else {
            0.0
        };
        writeln!(f, "│ Usage:    {:.1}%", usage_percent)?;
        Ok(())
    }
}

impl fmt::Debug for Stack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Stack")
            .field("base", &format!("0x{:08x}", self.base))
            .field("size", &format!("0x{:08x}", self.sz))
            .field("capacity", &format!("0x{:08x}", self.cap))
            .field("pointer", &format!("0x{:08x}", self.sp))
            .finish()
    }
}

impl fmt::Display for Memory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // let free = self.alloc.free_list.lock().unwrap();
        writeln!(f)?;
        writeln!(
            f,
            "┌───────────── Memory (Memory Management Unit) ─────────────┐"
        )?;

        let free_bytes = self.free_bytes();
        let used_bytes = self.alloc.used_list.len() * MEM_CTXT.page_size;
        let percent_used = if PHYS_TOTAL > 0 {
            (used_bytes as f64 / PHYS_TOTAL as f64) * 100.0
        } else {
            0.0
        };
        let percent_free = if PHYS_TOTAL > 0 {
            (free_bytes as f64 / PHYS_TOTAL as f64) * 100.0
        } else {
            0.0
        };

        writeln!(
            f,
            "│ Free Pages:      {} / {}",
            self.alloc.free_list.len(),
            self.alloc.free_list.len() + self.alloc.used_list.len()
        )?;
        writeln!(
            f,
            "│ Used Pages:      {} / {}",
            self.alloc.used_list.len(),
            self.alloc.free_list.len() + self.alloc.used_list.len()
        )?;
        writeln!(
            f,
            "├───────────────────────────────────────────────────────────┤"
        )?;
        writeln!(
            f,
            "│ Free Memory:     {:>8} bytes ({:>5.1}%)",
            free_bytes, percent_free
        )?;
        writeln!(
            f,
            "│ Used Memory:     {:>8} bytes ({:>5.1}%)",
            used_bytes, percent_used
        )?;
        writeln!(f, "│ Total Memory:    {:>8} bytes", PHYS_TOTAL)?;
        writeln!(
            f,
            "├───────────────────────────────────────────────────────────┤"
        )?;
        writeln!(f, "│ Active Allocations: {}", self.alloc_var.len())?;

        if !self.alloc_var.is_empty() {
            writeln!(f, "│")?;
            writeln!(f, "│ Allocation Details:")?;
            for (addr, size) in self.alloc_var.iter() {
                writeln!(f, "│   ├─ 0x{:08x}: {} bytes", addr, size)?;
            }
        }
        writeln!(f, "│ Page Size:       {} bytes", MEM_CTXT.page_size)?;
        writeln!(
            f,
            "│ Virtual Address: {} bits",
            MEM_CTXT.v_addr_lvl_len + MEM_CTXT.v_addr_off_len,
        )?;
        writeln!(f, "{}", self._ram.stack)?;
        writeln!(f, "{}", self.alloc)?;
        Ok(())
    }
}

impl fmt::Display for FrameAllocator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f)?;
        writeln!(f, "├───────────── Frame Allocator ─────────────┤")?;
        writeln!(f, "│ Total Frames:   {}", self.total_frames())?;
        writeln!(f, "│ Free Frames:    {}", self.free_frames())?;
        writeln!(f, "│ Used Frames:    {}", self.used_frames())?;
        writeln!(f, "├───────────────────────────────────────────┤")?;
        //writeln!(f, "│ Free Frame List: {:?}", self.free_list)?;
        //writeln!(f, "│ Used Frame List: {:?}", self.used_list)?;
        writeln!(f, "└───────────────────────────────────────────┘")?;
        Ok(())
    }
}

impl fmt::Debug for PageTableEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PageTableEntry")
            .field("raw", &format!("0x{:08x}", self.0))
            .field("present", &self.get_flag(Flag::Present))
            .field("writable", &self.get_flag(Flag::Writable))
            .field("readable", &self.get_flag(Flag::Read))
            .field("ppn", &format!("0x{:05x}", self.get_ppn()))
            .finish()
    }
}

impl fmt::Display for PageTableEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PTE(0x{:08x}) [", self.0)?;
        if self.get_flag(Flag::Present) {
            write!(f, "P")?;
        }
        if self.get_flag(Flag::Writable) {
            write!(f, "W")?;
        }
        if self.get_flag(Flag::Read) {
            write!(f, "R")?;
        }
        write!(f, "] -> 0x{:05x}", self.get_ppn())
    }
}
