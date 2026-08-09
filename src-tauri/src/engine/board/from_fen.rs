use crate::engine::{bitboard::BitBoard, types::PieceKind};

impl super::Board {
    /// Parses a full FEN string and overwrites this board's position state.
    //impk Board {
    /// Parses a full FEN string and overwrites this board's position state.
    /// Panics on malformed FEN — intended for test fixtures, not untrusted input.
    pub fn from_fen(&mut self, fen: &str) {
        let mut parts = fen.split_whitespace();
        let placement = parts.next().expect("FEN missing piece placement field");
        let active_color = parts.next().unwrap_or("w");
        let castling = parts.next().unwrap_or("-");
        let en_passant = parts.next().unwrap_or("-");
        let halfmove = parts.next().unwrap_or("0");
        let fullmove = parts.next().unwrap_or("1");

        // --- piece placement ---
        let mut pieces = [[BitBoard(0); 6]; 2];
        let ranks: Vec<&str> = placement.split('/').collect();
        assert_eq!(
            ranks.len(),
            8,
            "FEN placement must have 8 ranks, got {}",
            ranks.len()
        );

        for (rank_from_top, rank_str) in ranks.iter().enumerate() {
            let rank = 7 - rank_from_top;
            let mut file = 0usize;

            for ch in rank_str.chars() {
                if let Some(empty_count) = ch.to_digit(10) {
                    file += empty_count as usize;
                    continue;
                }
                assert!(file < 8, "FEN rank '{}' overflows 8 files", rank_str);

                let color = if ch.is_ascii_uppercase() { 0 } else { 1 };
                let kind = match ch.to_ascii_lowercase() {
                    'p' => PieceKind::Pawn,
                    'n' => PieceKind::Knight,
                    'b' => PieceKind::Bishop,
                    'r' => PieceKind::Rook,
                    'q' => PieceKind::Queen,
                    'k' => PieceKind::King,
                    other => panic!("unrecognized FEN piece char '{}'", other),
                };

                let sq = rank * 8 + file;
                pieces[color][kind as usize] |= BitBoard(1u64 << sq);
                file += 1;
            }
            assert_eq!(
                file, 8,
                "FEN rank '{}' does not fill 8 files (got {})",
                rank_str, file
            );
        }

        let color_occupency = [
            pieces[0].iter().fold(BitBoard(0), |acc, &bb| acc | bb),
            pieces[1].iter().fold(BitBoard(0), |acc, &bb| acc | bb),
        ];
        let total_occupency = color_occupency[0] | color_occupency[1];

        // --- active color ---
        let player_turn: u8 = match active_color {
            "w" => 0,
            "b" => 1,
            other => panic!("unrecognized active color '{}'", other),
        };

        // --- castling rights, packed as bits: K=1, Q=2, k=4, q=8 ---
        let mut castling_rights = 0u8;
        if castling != "-" {
            for ch in castling.chars() {
                castling_rights |= match ch {
                    'K' => 0b0001,
                    'Q' => 0b0010,
                    'k' => 0b0100,
                    'q' => 0b1000,
                    other => panic!("unrecognized castling char '{}'", other),
                };
            }
        }

        // --- en passant target square ---
        let en_passant_square: u8 = if en_passant == "-" {
            64 // sentinel for "no en passant available" — adjust if you use a different sentinel elsewhere
        } else {
            let bytes = en_passant.as_bytes();
            assert_eq!(
                bytes.len(),
                2,
                "en passant square '{}' malformed",
                en_passant
            );
            let file = (bytes[0] - b'a') as u8;
            let rank = (bytes[1] - b'1') as u8;
            rank * 8 + file
        };

        let halfmove_clock: u8 = halfmove.parse().expect("invalid halfmove clock in FEN");
        let fullmove_clock: u16 = fullmove.parse().expect("invalid fullmove clock in FEN");

        // --- commit to self ---
        self.pieces = pieces;
        self.color_occupency = color_occupency;
        self.total_occupency = total_occupency;
        self.player_turn = player_turn;
        self.castling_rights = castling_rights;
        self.en_passant_square = en_passant_square;
        self.halfmove_clock = halfmove_clock;
        self.fullmove_clock = fullmove_clock;

        // enemy_attack_mask depends on the position, so it needs recomputing here —
        // replace this with whatever your actual attack-mask generation method is called
        self.update_enemy_attack_mask();

        // zobrist_hash left as-is per your TODO; set to 0 explicitly if you want a clean slate
        self.zobrist_hash = 0;
    }
}

//Panics on malformed FEN — intended for test fixtures, not untrusted input.
