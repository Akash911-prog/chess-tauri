use super::*;

#[test]
fn legal_test() {
    let game = Game::new();

    {
        let mut board = game.board.lock().unwrap();
        board.move_gen.generate_moves();
        board.set_occupency();
    }

    let legal_moves = game.board.lock().unwrap().get_all_legal_moves();

    legal_moves.iter().enumerate().for_each(|(i, bb)| {
        println!("{}: {}", i, bb);
    });

    let total: u32 = legal_moves.iter().map(|bb| bb.count()).sum();
    assert_eq!(
        total, 20,
        "starting position should have exactly 20 legal moves for white, got {total}"
    );
}
