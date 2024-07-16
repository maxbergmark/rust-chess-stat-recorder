use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoveType {
    KingCheckmate {
        was_played: bool,
        is_capture: bool,
    },
    DoubleDisambiguationCheckmate {
        was_played: bool,
        is_capture: bool,
        checkmate_type: CheckType,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckType {
    Normal,
    Discovered,
    Double,
}

impl MoveType {
    fn format_king_checkmate(is_capture: bool, was_played: bool) -> String {
        format!(
            "K {} {}",
            if is_capture { 'x' } else { ' ' },
            if was_played { ' ' } else { '?' },
        )
    }

    fn format_double_disambiguation_checkmate(
        is_capture: bool,
        was_played: bool,
        checkmate_type: &CheckType,
    ) -> String {
        format!(
            "DD{}{}{}",
            if is_capture { 'x' } else { ' ' },
            match checkmate_type {
                CheckType::Normal => ' ',
                CheckType::Discovered => 'D',
                CheckType::Double => '2',
            },
            if was_played { ' ' } else { '?' },
        )
    }
}

impl Display for MoveType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::KingCheckmate {
                is_capture,
                was_played,
            } => Self::format_king_checkmate(*is_capture, *was_played),
            Self::DoubleDisambiguationCheckmate {
                is_capture,
                was_played,
                checkmate_type,
            } => Self::format_double_disambiguation_checkmate(
                *is_capture,
                *was_played,
                checkmate_type,
            ),
        };
        write!(f, "{s}")
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Default)]
#[repr(u8)]
pub enum GameResult {
    WhiteWin = 1,
    BlackWin = 2,
    Draw = 3,
    #[default]
    Unfinished = 4,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum Termination {
    Normal = 1,
    TimeForfeit = 2,
    Abandoned = 3,
    #[default]
    Unterminated = 4,
    RulesInfraction = 5,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum TimeControl {
    CorrespondenceGame = 1,
    ClassicalGame = 2,
    #[default]
    StandardGame = 3,
    RapidGame = 4,
    BlitzGame = 5,
    BulletGame = 6,
    UltraBulletGame = 7,
    CorrespondenceTournament = 10,
    ClassicalTournament = 11,
    StandardTournament = 12,
    RapidTournament = 13,
    BlitzTournament = 14,
    BulletTournament = 15,
    UltraBulletTournament = 16,
}
