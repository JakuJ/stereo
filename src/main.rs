mod constants;
mod game;
mod heightmap;
mod rendering;
mod run;

use run::run;

fn main() {
    pollster::block_on(run());
}
