use super::*;

#[test]
fn legal_test() {
    let game = Game::new();

    {
        let mut board = game.board.lock().unwrap();
        board.move_gen.generate_moves();
        board.init();
    }

    let legal_moves = game.board.lock().unwrap().get_all_legal_moves(0, false);

    // legal_moves.iter().enumerate().for_each(|(i, bb)| {
    //     println!("{}: {}", i, bb);
    // });

    let total: u32 = legal_moves.iter().map(|bb| bb.count()).sum();
    assert_eq!(
        total, 20,
        "starting position should have exactly 20 legal moves for white, got {total}"
    );
}

#[test]
fn check_test() {
    let game = Game::new();

    let mut board = game.board.lock().unwrap();
    board.move_gen.generate_moves();
    board.init();

    board.from_fen("8/8/8/8/8/6n1/8/r6K w - - 0 1");

    let check_info = board.check_for_check();

    println!("{:?}", check_info);
    println!("{}", board.total_occupency);

    assert_eq!(true, check_info.is_check);
}
