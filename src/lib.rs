mod constants;
pub mod game;
mod heightmap;
mod rendering;
mod run;

use run::run;

pub fn run_game() {
    pollster::block_on(run());
}
