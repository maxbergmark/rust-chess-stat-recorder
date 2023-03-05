

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub (crate) enum GameResult {
    WhiteWin = 1,
    BlackWin = 2,
    Draw = 3,
    Unfinished = 4,
}

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub (crate) enum Termination {
    Normal = 1,
    TimeForfeit = 2,
    Abandoned = 3,
    Unterminated = 4,
    RulesInfraction = 5,
}

#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub (crate) enum TimeControl {
    RatedCorrespondenceGame = 1,
    RatedClassicalGame = 2,
    RatedStandardGame = 3,
    RatedRapidGame = 4,
    RatedBlitzGame = 5,
    RatedBulletGame = 6,
    RatedUltraBulletGame = 7,
    RatedCorrespondenceTournament = 10,
    RatedClassicalTournament = 11,
    RatedStandardTournament = 12,
    RatedRapidTournament = 13,
    RatedBlitzTournament = 14,
    RatedBulletTournament = 15,
    RatedUltraBulletTournament = 16,
}