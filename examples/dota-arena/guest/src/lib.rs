use serde::{Deserialize, Serialize};

// Shared 1MB exchange buffer for passing inputs/outputs
const BUFFER_SIZE: usize = 1024 * 1024;
static mut EXCHANGE_BUFFER: [u8; BUFFER_SIZE] = [0u8; BUFFER_SIZE];

#[link(wasm_import_module = "env")]
extern "C" {
    // Saves a execution checkpoint
    pub fn checkpoint();
}

// Export the exchange buffer pointer so the host client can read/write data directly
#[no_mangle]
pub extern "C" fn get_exchange_buffer_pointer() -> *mut u8 {
    unsafe { std::ptr::addr_of_mut!(EXCHANGE_BUFFER) as *mut u8 }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Cooldowns {
    storm_hammer: u32,
    warcry: u32,
    dragon_slave: u32,
    laguna_blade: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Hero {
    id: u32,
    name: String,
    x: i32,
    y: i32,
    hp: i32,
    max_hp: i32,
    damage: i32,
    range: i32,
    stunned_turns: u32,
    warcry_turns: u32, // Sven armor buff
    cooldowns: Cooldowns,
    dodge_chance: u32, // Dodge chance percentage
    is_radiant: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct GameState {
    turn: u32,
    status: String, // "active", "won_radiant", "won_dire", "draw"
    radiant: Vec<Hero>,
    dire: Vec<Hero>,
    log: Vec<String>,
    checkpoint_count: u32,
}

#[derive(Debug, Deserialize, Clone)]
struct HeroAction {
    hero_id: u32,
    action: String, // "move_up" | "move_down" | "move_left" | "move_right" | "attack" | "storm_hammer" | "warcry" | "dragon_slave" | "laguna_blade" | "none"
}

#[derive(Debug, Deserialize, Clone)]
struct TurnActionRequest {
    action: String, // "play" | "reset" | "get_state"
    game_mode: String, // "pvp" | "pve_radiant" (user plays Radiant, Dire is AI) | "pve_dire" (user plays Dire, Radiant is AI)
    radiant_actions: Vec<HeroAction>,
    dire_actions: Vec<HeroAction>,
}

static mut GAME_STATE: Option<GameState> = None;

fn deterministic_random(seed: u32) -> u32 {
    seed.wrapping_mul(1103515245).wrapping_add(12345)
}

fn check_dodge(turn: u32, attacker_id: u32, chance: u32) -> bool {
    if chance == 0 {
        return false;
    }
    let seed = turn.wrapping_add(attacker_id);
    let rand_val = deterministic_random(seed) % 100;
    rand_val < chance
}

// Helper to initialize the game state
fn init_default_game() -> GameState {
    let mut radiant = Vec::new();
    for i in 1..=5 {
        radiant.push(Hero {
            id: i,
            name: format!("Sven {}", i),
            x: 0,
            y: i as i32 + 1, // positions 2 to 6 on Y axis
            hp: 200,
            max_hp: 200,
            damage: 20,
            range: 1,
            stunned_turns: 0,
            warcry_turns: 0,
            cooldowns: Cooldowns {
                storm_hammer: 0,
                warcry: 0,
                dragon_slave: 0,
                laguna_blade: 0,
            },
            dodge_chance: 15,
            is_radiant: true,
        });
    }

    let mut dire = Vec::new();
    for i in 1..=5 {
        dire.push(Hero {
            id: i + 5,
            name: format!("Lina {}", i),
            x: 7,
            y: i as i32 + 1, // positions 2 to 6 on Y axis
            hp: 130,
            max_hp: 130,
            damage: 12,
            range: 3,
            stunned_turns: 0,
            warcry_turns: 0,
            cooldowns: Cooldowns {
                storm_hammer: 0,
                warcry: 0,
                dragon_slave: 0,
                laguna_blade: 0,
            },
            dodge_chance: 15,
            is_radiant: false,
        });
    }

    GameState {
        turn: 1,
        status: "active".to_string(),
        radiant,
        dire,
        log: vec!["5v5 Dota Arena initialized! Control your team and conquer.".to_string()],
        checkpoint_count: 0,
    }
}

// Calculate Manhattan distance between two heroes
fn get_distance(h1: &Hero, h2: &Hero) -> i32 {
    (h1.x - h2.x).abs() + (h1.y - h2.y).abs()
}

fn get_nearest_enemy(hero: &Hero, enemies: &[Hero]) -> Option<Hero> {
    let mut nearest: Option<Hero> = None;
    let mut min_dist = i32::MAX;
    for enemy in enemies.iter() {
        if enemy.hp > 0 {
            let d = (hero.x - enemy.x).abs() + (hero.y - enemy.y).abs();
            if d < min_dist {
                min_dist = d;
                nearest = Some(enemy.clone());
            }
        }
    }
    nearest
}

// Deterministic AI logic for Lina
fn get_lina_ai_action(hero: &Hero, enemies: &[Hero]) -> String {
    let nearest = match get_nearest_enemy(hero, enemies) {
        Some(e) => e,
        None => return "none".to_string(),
    };
    let dist = get_distance(hero, &nearest);

    if hero.cooldowns.laguna_blade == 0 && dist <= 3 {
        "laguna_blade".to_string()
    } else if hero.cooldowns.dragon_slave == 0 && dist <= 4 {
        "dragon_slave".to_string()
    } else if dist <= hero.range {
        "attack".to_string()
    } else {
        // Move towards nearest enemy
        let mut best_move = "none".to_string();
        let mut min_d = dist;
        let moves = [
            ("move_up", hero.x, hero.y - 1),
            ("move_down", hero.x, hero.y + 1),
            ("move_left", hero.x - 1, hero.y),
            ("move_right", hero.x + 1, hero.y),
        ];

        for (mv, nx, ny) in moves.iter() {
            if *nx >= 0 && *nx <= 7 && *ny >= 0 && *ny <= 7 {
                let d = (nx - nearest.x).abs() + (ny - nearest.y).abs();
                if d < min_d {
                    min_d = d;
                    best_move = mv.to_string();
                }
            }
        }
        best_move
    }
}

// Deterministic AI logic for Sven
fn get_sven_ai_action(hero: &Hero, enemies: &[Hero]) -> String {
    let nearest = match get_nearest_enemy(hero, enemies) {
        Some(e) => e,
        None => return "none".to_string(),
    };
    let dist = get_distance(hero, &nearest);

    if hero.cooldowns.storm_hammer == 0 && dist <= 3 {
        "storm_hammer".to_string()
    } else if hero.cooldowns.warcry == 0 && hero.warcry_turns == 0 && dist > 1 {
        "warcry".to_string()
    } else if dist <= hero.range {
        "attack".to_string()
    } else {
        // Move towards nearest enemy
        let mut best_move = "none".to_string();
        let mut min_d = dist;
        let moves = [
            ("move_up", hero.x, hero.y - 1),
            ("move_down", hero.x, hero.y + 1),
            ("move_left", hero.x - 1, hero.y),
            ("move_right", hero.x + 1, hero.y),
        ];

        for (mv, nx, ny) in moves.iter() {
            if *nx >= 0 && *nx <= 7 && *ny >= 0 && *ny <= 7 {
                let d = (nx - nearest.x).abs() + (ny - nearest.y).abs();
                if d < min_d {
                    min_d = d;
                    best_move = mv.to_string();
                }
            }
        }
        best_move
    }
}

#[no_mangle]
pub extern "C" fn play_turn(input_len: u32) -> i32 {
    let input_bytes = unsafe { &EXCHANGE_BUFFER[..input_len as usize] };
    let req: TurnActionRequest = match serde_json::from_slice(input_bytes) {
        Ok(r) => r,
        Err(_) => return -1,
    };

    let mut state = unsafe {
        if GAME_STATE.is_none() || req.action == "reset" {
            init_default_game()
        } else {
            GAME_STATE.clone().unwrap()
        }
    };

    if req.action == "get_state" {
        let resp_bytes = serde_json::to_vec(&state).unwrap();
        if resp_bytes.len() > BUFFER_SIZE {
            return -2;
        }
        unsafe {
            EXCHANGE_BUFFER[..resp_bytes.len()].copy_from_slice(&resp_bytes);
            GAME_STATE = Some(state);
        }
        return resp_bytes.len() as i32;
    }

    if req.action == "claim_victory" {
        if req.game_mode == "radiant" {
            state.status = "won_dire".to_string();
            state.log.push("Dire team claimed victory due to Radiant disconnect timeout!".to_string());
        } else {
            state.status = "won_radiant".to_string();
            state.log.push("Radiant team claimed victory due to Dire disconnect timeout!".to_string());
        }
        state.turn += 1;
        state.checkpoint_count += 1;
        unsafe {
            GAME_STATE = Some(state.clone());
            checkpoint();
        }
        let resp_bytes = serde_json::to_vec(&state).unwrap();
        unsafe {
            EXCHANGE_BUFFER[..resp_bytes.len()].copy_from_slice(&resp_bytes);
        }
        return resp_bytes.len() as i32;
    }

    if req.action != "reset" && state.status != "active" {
        let resp_bytes = serde_json::to_vec(&state).unwrap();
        if resp_bytes.len() > BUFFER_SIZE {
            return -2;
        }
        unsafe {
            EXCHANGE_BUFFER[..resp_bytes.len()].copy_from_slice(&resp_bytes);
            GAME_STATE = Some(state);
        }
        return resp_bytes.len() as i32;
    }

    if req.action == "reset" {
        state.checkpoint_count += 1;
        unsafe {
            GAME_STATE = Some(state.clone());
            checkpoint();
        }
        let resp_bytes = serde_json::to_vec(&state).unwrap();
        unsafe {
            EXCHANGE_BUFFER[..resp_bytes.len()].copy_from_slice(&resp_bytes);
        }
        return resp_bytes.len() as i32;
    }

    // Resolve Turn Actions
    state.log.clear();
    state.log.push(format!("--- Turn {} ---", state.turn));

    // 1. Determine Actions (AI mapping)
    let mut radiant_actions = req.radiant_actions.clone();
    let mut dire_actions = req.dire_actions.clone();

    if req.game_mode == "pve_radiant" {
        dire_actions.clear();
        for dire_hero in state.dire.iter() {
            if dire_hero.hp > 0 {
                let act = get_lina_ai_action(dire_hero, &state.radiant);
                dire_actions.push(HeroAction {
                    hero_id: dire_hero.id,
                    action: act,
                });
            }
        }
    } else if req.game_mode == "pve_dire" {
        radiant_actions.clear();
        for rad_hero in state.radiant.iter() {
            if rad_hero.hp > 0 {
                let act = get_sven_ai_action(rad_hero, &state.dire);
                radiant_actions.push(HeroAction {
                    hero_id: rad_hero.id,
                    action: act,
                });
            }
        }
    }

    // Process stuns, cooldowns, and active buffs for ALL heroes
    for h in state.radiant.iter_mut().chain(state.dire.iter_mut()) {
        if h.hp <= 0 {
            continue;
        }
        if h.stunned_turns > 0 {
            h.stunned_turns -= 1;
        }
        if h.warcry_turns > 0 {
            h.warcry_turns -= 1;
        }
        if h.cooldowns.storm_hammer > 0 { h.cooldowns.storm_hammer -= 1; }
        if h.cooldowns.warcry > 0 { h.cooldowns.warcry -= 1; }
        if h.cooldowns.dragon_slave > 0 { h.cooldowns.dragon_slave -= 1; }
        if h.cooldowns.laguna_blade > 0 { h.cooldowns.laguna_blade -= 1; }
    }

    // 2. Resolve Movements (Radiant first, then Dire)
    let is_occupied = |x: i32, y: i32, rad: &[Hero], dir: &[Hero], current_id: u32| -> bool {
        for h in rad.iter().chain(dir.iter()) {
            if h.hp > 0 && h.id != current_id && h.x == x && h.y == y {
                return true;
            }
        }
        false
    };

    // Radiant Movements
    for act in radiant_actions.iter() {
        if let Some(h_idx) = state.radiant.iter().position(|r| r.id == act.hero_id) {
            let h = &state.radiant[h_idx];
            if h.hp <= 0 || h.stunned_turns > 0 {
                continue;
            }
            let mut nx = h.x;
            let mut ny = h.y;
            match act.action.as_str() {
                "move_up" => ny = (ny - 1).max(0),
                "move_down" => ny = (ny + 1).min(7),
                "move_left" => nx = (nx - 1).max(0),
                "move_right" => nx = (nx + 1).min(7),
                _ => {}
            }
            if (nx != h.x || ny != h.y) && !is_occupied(nx, ny, &state.radiant, &state.dire, h.id) {
                state.radiant[h_idx].x = nx;
                state.radiant[h_idx].y = ny;
                state.log.push(format!("{} moves to ({}, {}).", state.radiant[h_idx].name, nx, ny));
            }
        }
    }

    // Dire Movements
    for act in dire_actions.iter() {
        if let Some(h_idx) = state.dire.iter().position(|d| d.id == act.hero_id) {
            let h = &state.dire[h_idx];
            if h.hp <= 0 || h.stunned_turns > 0 {
                continue;
            }
            let mut nx = h.x;
            let mut ny = h.y;
            match act.action.as_str() {
                "move_up" => ny = (ny - 1).max(0),
                "move_down" => ny = (ny + 1).min(7),
                "move_left" => nx = (nx - 1).max(0),
                "move_right" => nx = (nx + 1).min(7),
                _ => {}
            }
            if (nx != h.x || ny != h.y) && !is_occupied(nx, ny, &state.radiant, &state.dire, h.id) {
                state.dire[h_idx].x = nx;
                state.dire[h_idx].y = ny;
                state.log.push(format!("{} moves to ({}, {}).", state.dire[h_idx].name, nx, ny));
            }
        }
    }

    // 3. Resolve Spells & Attacks
    // Sven combat:
    for act in radiant_actions.iter() {
        let r_pos = match state.radiant.iter().position(|r| r.id == act.hero_id) {
            Some(p) => p,
            None => continue,
        };
        let r_hero = state.radiant[r_pos].clone();
        if r_hero.hp <= 0 || r_hero.stunned_turns > 0 {
            continue;
        }

        match act.action.as_str() {
            "attack" => {
                if let Some(target) = get_nearest_enemy(&r_hero, &state.dire) {
                    let d = get_distance(&r_hero, &target);
                    if d <= r_hero.range {
                        let t_pos = state.dire.iter().position(|d| d.id == target.id).unwrap();
                        if check_dodge(state.turn, r_hero.id, state.dire[t_pos].dodge_chance) {
                            state.log.push(format!("[DODGE] {} dodged {}'s attack!", state.dire[t_pos].name, r_hero.name));
                        } else {
                            state.dire[t_pos].hp = (state.dire[t_pos].hp - r_hero.damage).max(0);
                            state.log.push(format!("{} attacks {} for {} damage!", r_hero.name, state.dire[t_pos].name, r_hero.damage));
                        }
                    }
                }
            }
            "storm_hammer" => {
                if let Some(target) = get_nearest_enemy(&r_hero, &state.dire) {
                    let d = get_distance(&r_hero, &target);
                    if d <= 3 {
                        let t_pos = state.dire.iter().position(|d| d.id == target.id).unwrap();
                        state.dire[t_pos].hp = (state.dire[t_pos].hp - 30).max(0);
                        state.dire[t_pos].stunned_turns = 1;
                        state.radiant[r_pos].cooldowns.storm_hammer = 3;
                        state.log.push(format!("{} casts STORM HAMMER on {}! (30 dmg, Stun)", r_hero.name, state.dire[t_pos].name));
                    }
                }
            }
            "warcry" => {
                state.radiant[r_pos].warcry_turns = 2;
                state.radiant[r_pos].cooldowns.warcry = 4;
                
                // Dash up to 2 cells towards nearest enemy
                if let Some(target) = get_nearest_enemy(&r_hero, &state.dire) {
                    let mut steps = 0;
                    for _ in 0..2 {
                        let h = &state.radiant[r_pos];
                        let dx = target.x - h.x;
                        let dy = target.y - h.y;
                        if dx == 0 && dy == 0 { break; }
                        let mut nx = h.x;
                        let mut ny = h.y;
                        if dx.abs() >= dy.abs() {
                            nx += if dx > 0 { 1 } else { -1 };
                        } else {
                            ny += if dy > 0 { 1 } else { -1 };
                        }
                        if !is_occupied(nx, ny, &state.radiant, &state.dire, h.id) {
                            state.radiant[r_pos].x = nx;
                            state.radiant[r_pos].y = ny;
                            steps += 1;
                        } else {
                            break;
                        }
                    }
                    state.log.push(format!("{} shouts a WARCRY, gaining armor and leaping {} steps!", r_hero.name, steps));
                } else {
                    state.log.push(format!("{} shouts a WARCRY, gaining armor!", r_hero.name));
                }
            }
            _ => {}
        }
    }

    // Lina combat:
    for act in dire_actions.iter() {
        let d_pos = match state.dire.iter().position(|d| d.id == act.hero_id) {
            Some(p) => p,
            None => continue,
        };
        let d_hero = state.dire[d_pos].clone();
        if d_hero.hp <= 0 || d_hero.stunned_turns > 0 {
            continue;
        }

        match act.action.as_str() {
            "attack" => {
                if let Some(target) = get_nearest_enemy(&d_hero, &state.radiant) {
                    let d = get_distance(&d_hero, &target);
                    if d <= d_hero.range {
                        let t_pos = state.radiant.iter().position(|r| r.id == target.id).unwrap();
                        if check_dodge(state.turn, d_hero.id, state.radiant[t_pos].dodge_chance) {
                            state.log.push(format!("[DODGE] {} dodged {}'s attack!", state.radiant[t_pos].name, d_hero.name));
                        } else {
                            let mut dmg = d_hero.damage;
                            if state.radiant[t_pos].warcry_turns > 0 {
                                dmg = (dmg - 8).max(0);
                            }
                            state.radiant[t_pos].hp = (state.radiant[t_pos].hp - dmg).max(0);
                            state.log.push(format!("{} attacks {} for {} damage! (Blocked: {})", d_hero.name, state.radiant[t_pos].name, dmg, d_hero.damage - dmg));
                        }
                    }
                }
            }
            "dragon_slave" => {
                if let Some(target) = get_nearest_enemy(&d_hero, &state.radiant) {
                    let d = get_distance(&d_hero, &target);
                    if d <= 4 {
                        let t_pos = state.radiant.iter().position(|r| r.id == target.id).unwrap();
                        let mut dmg = 28;
                        if state.radiant[t_pos].warcry_turns > 0 {
                            dmg = (dmg - 8).max(0);
                        }
                        state.radiant[t_pos].hp = (state.radiant[t_pos].hp - dmg).max(0);
                        state.dire[d_pos].cooldowns.dragon_slave = 2;
                        state.log.push(format!("{} casts DRAGON SLAVE on {}! ({} damage)", d_hero.name, state.radiant[t_pos].name, dmg));
                    }
                }
            }
            "laguna_blade" => {
                if let Some(target) = get_nearest_enemy(&d_hero, &state.radiant) {
                    let d = get_distance(&d_hero, &target);
                    if d <= 3 {
                        let t_pos = state.radiant.iter().position(|r| r.id == target.id).unwrap();
                        let mut dmg = 45;
                        if state.radiant[t_pos].warcry_turns > 0 {
                            dmg = (dmg - 8).max(0);
                        }
                        state.radiant[t_pos].hp = (state.radiant[t_pos].hp - dmg).max(0);
                        state.dire[d_pos].cooldowns.laguna_blade = 5;
                        state.log.push(format!("{} casts LAGUNA BLADE on {}! ({} damage)", d_hero.name, state.radiant[t_pos].name, dmg));
                    }
                }
            }
            _ => {}
        }
    }

    // Check Game Over
    let radiant_alive = state.radiant.iter().any(|r| r.hp > 0);
    let dire_alive = state.dire.iter().any(|d| d.hp > 0);

    if !radiant_alive && !dire_alive {
        state.status = "draw".to_string();
        state.log.push("Both teams have been completely wiped out! Draw!".to_string());
    } else if !radiant_alive {
        state.status = "won_dire".to_string();
        state.log.push("Dire team (Lina) has defeated Radiant! Victory for the Flame!".to_string());
    } else if !dire_alive {
        state.status = "won_radiant".to_string();
        state.log.push("Radiant team (Sven) has defeated Dire! Victory for the Rogue Knight!".to_string());
    }

    state.turn += 1;
    state.checkpoint_count += 1;

    unsafe {
        GAME_STATE = Some(state.clone());
        checkpoint();
    }

    let resp_bytes = serde_json::to_vec(&state).unwrap();
    if resp_bytes.len() > BUFFER_SIZE {
        return -2;
    }
    unsafe {
        EXCHANGE_BUFFER[..resp_bytes.len()].copy_from_slice(&resp_bytes);
    }
    resp_bytes.len() as i32
}
