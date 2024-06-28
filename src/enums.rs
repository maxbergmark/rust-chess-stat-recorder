#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub(crate) enum GameResult {
    WhiteWin = 1,
    BlackWin = 2,
    Draw = 3,
    Unfinished = 4,
}

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub(crate) enum Termination {
    Normal = 1,
    TimeForfeit = 2,
    Abandoned = 3,
    Unterminated = 4,
    RulesInfraction = 5,
}

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub(crate) enum TimeControl {
    CorrespondenceGame = 1,
    ClassicalGame = 2,
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
