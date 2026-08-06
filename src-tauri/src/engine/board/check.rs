impl super::Board {
    pub fn check_for_check(&self) -> CheckInfo {
        let mut info = CheckInfo::new();

        info
    }

    pub fn validate_move_with_check(&self, from: u8, to: u8) -> bool {
        true
    }
}

struct CheckInfo {
    is_check: bool,
    check_square: [u8; 2],
    piece_idx: [u8; 2],
}

impl CheckInfo {
    fn new() -> CheckInfo {
        CheckInfo {
            is_check: false,
            check_square: [0; 2],
            piece_idx: [0; 2],
        }
    }
}
