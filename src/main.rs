mod run;

use crate::run::run;

fn main() {
    pollster::block_on(run());
}
