mod utils;

extern crate fixedbitset;
extern crate js_sys;
extern crate web_sys;

use fixedbitset::FixedBitSet;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    };
}

pub struct Timer<'a> {
    name: &'a str,
}

impl<'a> Timer<'a> {
    pub fn new(name: &'a str) -> Timer<'a> {
        web_sys::console::time_with_label(name);
        Timer {name }
    }
}

impl <'a> Drop for Timer<'a> {
    fn drop(&mut self) {
        web_sys::console::time_end_with_label(self.name);
    }
}

#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    cells: FixedBitSet,
}

#[wasm_bindgen]
impl Universe {
    pub fn new(w: u32, h: u32) -> Universe {
        utils::set_panic_hook();

        let width = w;
        let height = h;

        let size = (width * height) as usize;
        let mut cells = FixedBitSet::with_capacity(size);

        for _i in 0..size {
            cells.set(_i, js_sys::Math::random() < 0.5);
        }

        Universe {
            width,
            height,
            cells,
        }
    }

    pub fn reset(&mut self) {
        let size = (self.width * self.height) as usize;

        for _i in 0..size {
            self.cells.set(_i, js_sys::Math::random() < 0.5);
        }
    }

    pub fn kill_universe(&mut self) {
        self.cells.set_range(.., false);
    }

    pub fn create_glider(&mut self, row: u32, column: u32) {
        self.set_cells(&[
            (row - 1, column),
            (row, column + 1),
            (row + 1, column - 1),
            (row + 1, column),
            (row + 1, column + 1),
        ]);
        self.unset_cells(&[
            (row, column),
            (row - 1, column - 1),
            (row - 1, column + 1),
            (row, column - 1),
        ]);
    }

    pub fn create_pulsar(&mut self, row: u32, column: u32) {
        // Top left section
        self.set_cells(&[
            (row - 1, column - 2),
            (row - 1, column - 3),
            (row - 1, column - 4),
            (row - 2, column - 1),
            (row - 3, column - 1),
            (row - 4, column - 1),
            (row - 6, column - 2),
            (row - 6, column - 3),
            (row - 6, column - 4),
            (row - 2, column - 6),
            (row - 3, column - 6),
            (row - 4, column - 6),
        ]);
        // Top right section
        self.set_cells(&[
            (row - 1, column + 2),
            (row - 1, column + 3),
            (row - 1, column + 4),
            (row - 2, column + 1),
            (row - 3, column + 1),
            (row - 4, column + 1),
            (row - 6, column + 2),
            (row - 6, column + 3),
            (row - 6, column + 4),
            (row - 2, column + 6),
            (row - 3, column + 6),
            (row - 4, column + 6),
        ]);
        // Bottom left section
        self.set_cells(&[
            (row + 1, column - 2),
            (row + 1, column - 3),
            (row + 1, column - 4),
            (row + 2, column - 1),
            (row + 3, column - 1),
            (row + 4, column - 1),
            (row + 6, column - 2),
            (row + 6, column - 3),
            (row + 6, column - 4),
            (row + 2, column - 6),
            (row + 3, column - 6),
            (row + 4, column - 6),
        ]);
        // Bottom right section
        self.set_cells(&[
            (row + 1, column + 2),
            (row + 1, column + 3),
            (row + 1, column + 4),
            (row + 2, column + 1),
            (row + 3, column + 1),
            (row + 4, column + 1),
            (row + 6, column + 2),
            (row + 6, column + 3),
            (row + 6, column + 4),
            (row + 2, column + 6),
            (row + 3, column + 6),
            (row + 4, column + 6),
        ])
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn set_width(&mut self, width: u32) {
        self.width = width;
        let size = (width * self.height) as usize;
        self.cells = FixedBitSet::with_capacity(size);
        for _i in 0..size {
            self.cells.set(_i, false);
        }
    }

    pub fn set_height(&mut self, height: u32) {
        self.height = height;
        let size = (self.width * height) as usize;
        self.cells = FixedBitSet::with_capacity(size);
        for _i in 0..size {
            self.cells.set(_i, false);
        }
    }

    pub fn cells(&self) -> *const u32 {
        self.cells.as_slice().as_ptr()
    }

    pub fn toggle_cell(&mut self, row: u32, column: u32) {
        let idx = self.get_index(row, column);
        self.cells.toggle(idx);
    }

    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }

    fn live_neighbor_count(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;
        for delta_row in [self.height - 1, 0, 1].iter().cloned() {
            for delta_col in [self.width - 1, 0, 1].iter().cloned() {
                if delta_row == 0 && delta_col == 0 {
                    continue;
                }

                let neighbor_row = (row + delta_row) % self.height;
                let neighbor_col = (column + delta_col) % self.width;
                let idx = self.get_index(neighbor_row, neighbor_col);
                count += self.cells[idx] as u8;
            }
        }
        count
    }

    pub fn tick(&mut self) {
        let _timer = Timer::new("Universe::tick");
        let mut next = self.cells.clone();

        for row in 0..self.height {
            for col in 0..self.height {
                let idx = self.get_index(row, col);
                let cell = self.cells[idx];
                let live_neighbors = self.live_neighbor_count(row, col);

                // Debug printing
                /*log!(
                    "Cell [{}, {}] is {:?} and has {} live neighbors",
                    row,
                    col,
                    cell,
                    live_neighbors
                );*/

                next.set(
                    idx,
                    match (cell, live_neighbors) {
                        (true, x) if x < 2 => false,
                        (true, 2) | (true, 3) => true,
                        (true, x) if x > 3 => false,
                        (false, 3) => true,
                        (otherwise, _) => otherwise,
                    },
                );
            }
        }

        self.cells = next;
    }
}

impl Universe {
    pub fn get_cells(&self) -> &FixedBitSet {
        &self.cells
    }

    pub fn set_cells(&mut self, cells: &[(u32, u32)]) {
        for (row, col) in cells.iter().cloned() {
            let idx = self.get_index(row, col);
            self.cells.set(idx, true);
        }
    }

    pub fn unset_cells(&mut self, cells: &[(u32, u32)]) {
        for (row, col) in cells.iter().cloned() {
            let idx = self.get_index(row, col);
            self.cells.set(idx, false);
        }
    }
}
