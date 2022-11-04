#![feature(test)]

extern crate test;

mod game_bench {
    use stereo::game::Game;

    #[bench]
    fn bench_game_update(b: &mut test::Bencher) {
        let mut game = Game::new();
        b.iter(|| game.update());
    }
}
