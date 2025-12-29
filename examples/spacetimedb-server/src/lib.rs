use spacetimedb::{ReducerContext, Table, reducer};

// Re-export types from some-lib (Player struct has all derives there)
pub use some_lib::structs::{Guild, PlayerStatus, SpacetimedbPlayer, player};

/// Register a new player
#[reducer]
pub fn register_player(ctx: &ReducerContext, username: String) -> Result<(), String> {
    let identity = ctx.sender;

    // Check if player already exists
    if ctx.db.player().identity().find(identity).is_some() {
        return Err("Player already registered".into());
    }

    // Check if username is taken
    if ctx.db.player().username().find(&username).is_some() {
        return Err("Username already taken".into());
    }

    ctx.db.player().insert(SpacetimedbPlayer {
        id: 0, // auto_inc
        identity,
        username,
        level: 1,
        experience: 0,
        guild: Guild::None,
        status: PlayerStatus::Online,
        score: 0,
        games_played: 0,
        win_rate: 0.0,
        created_at: ctx.timestamp,
        last_seen: ctx.timestamp,
    });

    Ok(())
}

/// Update player status
#[reducer]
pub fn set_status(ctx: &ReducerContext, status: PlayerStatus) -> Result<(), String> {
    let player = ctx
        .db
        .player()
        .identity()
        .find(ctx.sender)
        .ok_or("Player not found")?;

    ctx.db.player().id().update(SpacetimedbPlayer {
        status,
        last_seen: ctx.timestamp,
        ..player
    });

    Ok(())
}

/// Join a guild
#[reducer]
pub fn join_guild(ctx: &ReducerContext, guild: Guild) -> Result<(), String> {
    let player = ctx
        .db
        .player()
        .identity()
        .find(ctx.sender)
        .ok_or("Player not found")?;

    ctx.db
        .player()
        .id()
        .update(SpacetimedbPlayer { guild, ..player });

    Ok(())
}

/// Update player stats after a game
#[reducer]
pub fn record_game(ctx: &ReducerContext, won: bool, score_earned: u32) -> Result<(), String> {
    let player = ctx
        .db
        .player()
        .identity()
        .find(ctx.sender)
        .ok_or("Player not found")?;

    let new_games_played = player.games_played + 1;
    let wins = if won {
        (player.win_rate * player.games_played as f32) as u32 + 1
    } else {
        (player.win_rate * player.games_played as f32) as u32
    };
    let new_win_rate = wins as f32 / new_games_played as f32;

    // Level up every 1000 XP
    let new_experience = player.experience + score_earned as u64;
    let new_level = (new_experience / 1000) as u32 + 1;

    ctx.db.player().id().update(SpacetimedbPlayer {
        score: player.score + score_earned,
        games_played: new_games_played,
        win_rate: new_win_rate,
        experience: new_experience,
        level: new_level,
        last_seen: ctx.timestamp,
        ..player
    });

    Ok(())
}

/// Admin reducer to seed test data
#[reducer]
pub fn seed_test_players(ctx: &ReducerContext, count: u32) -> Result<(), String> {
    let names = [
        "ShadowKnight",
        "DragonMage",
        "StormHunter",
        "FireWarrior",
        "IceSlayer",
        "ThunderMaster",
        "DarkLord",
        "LightKing",
        "SwiftBlade",
        "IronHeart",
    ];
    let guilds = [
        Guild::None,
        Guild::Warriors,
        Guild::Mages,
        Guild::Defenders,
        Guild::Rogues,
        Guild::Healers,
    ];
    let statuses = [
        PlayerStatus::Online,
        PlayerStatus::Away,
        PlayerStatus::Offline,
        PlayerStatus::InGame,
    ];

    for i in 0..count {
        let name_idx = (i as usize) % names.len();
        let username = format!("{}{}", names[name_idx], i);

        // Skip if username exists
        if ctx.db.player().username().find(&username).is_some() {
            continue;
        }

        ctx.db.player().insert(SpacetimedbPlayer {
            id: 0,
            identity: ctx.sender, // All test players owned by admin
            username,
            level: ((i % 100) + 1),
            experience: (i as u64 * 100) % 100000,
            guild: guilds[(i as usize) % guilds.len()],
            status: statuses[(i as usize) % statuses.len()],
            score: (i * 50) % 10000,
            games_played: (i % 500),
            win_rate: ((i % 100) as f32) / 100.0,
            created_at: ctx.timestamp,
            last_seen: ctx.timestamp,
        });
    }

    Ok(())
}
