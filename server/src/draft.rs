//! Season draft schedule — pure, deterministic logic for the single pod.
//!
//! Encodes the fixed 4-week / 8-beat draft arc from `design/metagame.md`:
//! twice-weekly beats that grow a player's toolkit and budget in small steps.
//! The *rhythm* (which kind of choice each beat is) is fixed and learnable; the
//! *content* (which options appear) is generated deterministically from a season
//! seed so every player in the pod sees the same offers and a replay can be
//! reproduced.
//!
//! This module is deliberately free of database / HTTP / engine-content wiring
//! (that lands in the API issue). It takes the content [`Pools`] as input and
//! returns plain data, so the schedule, offer generation, auto-resolve, and
//! unlocked-pool computation are all unit-testable in isolation.
//!
//! Wired into the API and battle runner in later issues; allow dead code until
//! then.
#![allow(dead_code)]

use std::collections::HashSet;

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::models::DraftPick;

/// Starting team budget (points) before any beat resolves.
pub const STARTING_BUDGET: u32 = 10;

/// How many options each beat offers (a curated lateral trio).
pub const OFFER_COUNT: usize = 3;

/// What kind of choice a beat presents. The kind determines which content pool
/// the offers are drawn from and how an unclaimed beat auto-resolves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BeatKind {
    /// The commander's banner (designating the team's commander).
    Banner,
    /// An item (aspect) augmenting a character.
    Item,
    /// A new character (archetype) added to the unlocked pool.
    Character,
    /// A team-wide passive buff.
    TeamPassive,
    /// Swap one drafted character for another. Never auto-applied.
    Swap,
}

/// One beat in the season schedule.
#[derive(Debug, Clone, Copy)]
pub struct Beat {
    pub kind: BeatKind,
    /// How much the team budget grows when this beat is revealed.
    pub budget_delta: u32,
    /// How many options are offered.
    pub offer_count: usize,
}

/// The fixed 8-beat season schedule (two beats per week over four weeks).
///
/// Budget grows 10 → 15 (+5 total) and the two `Character` beats lift the pool
/// from the starting ~5 to ~7. The alternating rhythm keeps each week legible:
/// establish your commander, then a steady drip of items / characters / a team
/// passive, a late swap to adapt, and a final item to round out the build.
pub fn schedule() -> Vec<Beat> {
    use BeatKind::*;
    [
        (Banner, 0),      // 1 — establish the commander's banner
        (Item, 1),        // 2 — budget + item
        (Character, 0),   // 3 — new character
        (TeamPassive, 1), // 4 — budget + team passive
        (Item, 1),        // 5 — item
        (Character, 1),   // 6 — new character + budget
        (Swap, 0),        // 7 — swap (skipped if unclaimed)
        (Item, 1),        // 8 — final item + budget
    ]
    .into_iter()
    .map(|(kind, budget_delta)| Beat {
        kind,
        budget_delta,
        offer_count: OFFER_COUNT,
    })
    .collect()
}

/// Total number of beats in a season.
pub fn beat_count() -> usize {
    schedule().len()
}

/// The kind of the beat at `index`, or `None` if out of range.
pub fn beat_kind(index: usize) -> Option<BeatKind> {
    schedule().get(index).map(|b| b.kind)
}

/// The content options available to draft from, by category. The same pools are
/// shared across the pod for a season; offers are sampled deterministically
/// unless a beat is curated (see `curated_offers`).
#[derive(Debug, Clone, Default)]
pub struct Pools {
    /// Characters (archetype names).
    pub characters: Vec<String>,
    /// Items (aspect names).
    pub items: Vec<String>,
    /// Team-wide passive names.
    pub team_passives: Vec<String>,
    /// Banner names.
    pub banners: Vec<String>,
    /// Optional hardcoded offers per beat (indexed by beat). A non-empty entry
    /// is shown verbatim for that beat; an empty entry (or a missing index)
    /// falls back to seeded sampling from the category pool. Built/validated by
    /// the server's content layer so the host can curate the pod.
    pub curated_offers: Vec<Vec<String>>,
}

impl Pools {
    /// The source pool a beat of `kind` draws its offers from. Swaps draw from
    /// the character pool (the replacement options).
    fn source(&self, kind: BeatKind) -> &[String] {
        match kind {
            BeatKind::Banner => &self.banners,
            BeatKind::Item => &self.items,
            BeatKind::Character | BeatKind::Swap => &self.characters,
            BeatKind::TeamPassive => &self.team_passives,
        }
    }
}

/// Mix a season seed with a beat index (and a salt) into a stable per-beat seed,
/// so each beat samples independently yet reproducibly.
fn beat_seed(season_seed: u64, beat_index: usize, salt: u64) -> u64 {
    season_seed
        .wrapping_add((beat_index as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15))
        .wrapping_add(salt)
}

/// A deterministic partial Fisher–Yates: returns up to `count` distinct items
/// sampled from `pool` using `seed`. Order is stable for a given seed.
fn sample(pool: &[String], count: usize, seed: u64) -> Vec<String> {
    let mut items: Vec<String> = pool.to_vec();
    let n = items.len();
    let take = count.min(n);
    let mut rng = StdRng::seed_from_u64(seed);
    for i in 0..take {
        let j = i + (rng.gen::<usize>() % (n - i));
        items.swap(i, j);
    }
    items.truncate(take);
    items
}

/// The options offered for the beat at `beat_index`, deterministic for a given
/// `(season_seed, beat_index, pools)`. Returns an empty vector if the index is
/// out of range.
pub fn offers(beat_index: usize, season_seed: u64, pools: &Pools) -> Vec<String> {
    let Some(beat) = schedule().get(beat_index).copied() else {
        return Vec::new();
    };
    // A curated (non-empty) entry for this beat is shown verbatim; otherwise
    // fall back to seeded sampling from the category pool.
    if let Some(curated) = pools.curated_offers.get(beat_index) {
        if !curated.is_empty() {
            return curated.clone();
        }
    }
    let pool = pools.source(beat.kind);
    sample(
        pool,
        beat.offer_count,
        beat_seed(season_seed, beat_index, 0),
    )
}

/// Resolve a beat the player let lapse (didn't claim before the next beat).
///
/// Per spec, an unclaimed pick is **auto-chosen at random** from that beat's
/// offers — except **swaps**, which are **skipped** (a character is never
/// swapped against the player's intent), so this returns `None` for a `Swap`
/// beat or when there are no offers.
pub fn resolve_missed_beat(
    beat_index: usize,
    season_seed: u64,
    pools: &Pools,
) -> Option<DraftPick> {
    let beat = schedule().get(beat_index).copied()?;
    if beat.kind == BeatKind::Swap {
        return None;
    }
    let options = offers(beat_index, season_seed, pools);
    if options.is_empty() {
        return None;
    }
    let mut rng = StdRng::seed_from_u64(beat_seed(season_seed, beat_index, 0xA5A5_5A5A));
    let idx = rng.gen::<usize>() % options.len();
    Some(DraftPick {
        beat: beat_index as u32,
        choice: options[idx].clone(),
    })
}

/// A player's currently unlocked content + budget, derived from the season clock
/// and their claimed picks. Feeds season team validation (the loader's
/// `UnlockedPool` plus a budget).
#[derive(Debug, Clone, Default)]
pub struct Unlocked {
    /// Unlocked archetypes (starting roster + drafted/swapped-in characters).
    pub archetypes: HashSet<String>,
    /// Unlocked aspects (drafted items).
    pub aspects: HashSet<String>,
    /// Unlocked team passives.
    pub team_passives: HashSet<String>,
    /// The current banner, if one has been drafted (latest claimed wins).
    pub banner: Option<String>,
    /// The current point budget.
    pub budget: u32,
}

/// Compute a player's unlocked pool and budget.
///
/// - `starting_characters` is the season's starter roster (always unlocked).
/// - `beats_revealed` is how many beats have been revealed; budget grows by each
///   revealed beat's `budget_delta` regardless of whether the player claimed it.
/// - `claimed` are the player's claimed picks, each routed into the right bucket
///   by its beat's kind. A `Swap` pick adds its replacement to the archetype
///   pool (the old character simply stops being used in the team config).
pub fn unlocked(
    starting_characters: &[String],
    beats_revealed: usize,
    claimed: &[DraftPick],
) -> Unlocked {
    let sched = schedule();

    let mut out = Unlocked {
        archetypes: starting_characters.iter().cloned().collect(),
        budget: STARTING_BUDGET,
        ..Default::default()
    };

    // Budget grows as beats are revealed.
    let revealed = beats_revealed.min(sched.len());
    for beat in &sched[..revealed] {
        out.budget += beat.budget_delta;
    }

    // Route each claimed pick into the right bucket by its beat kind.
    for pick in claimed {
        let Some(beat) = sched.get(pick.beat as usize) else {
            continue;
        };
        match beat.kind {
            BeatKind::Banner => out.banner = Some(pick.choice.clone()),
            BeatKind::Item => {
                out.aspects.insert(pick.choice.clone());
            }
            BeatKind::Character | BeatKind::Swap => {
                out.archetypes.insert(pick.choice.clone());
            }
            BeatKind::TeamPassive => {
                out.team_passives.insert(pick.choice.clone());
            }
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pools() -> Pools {
        Pools {
            characters: vec![
                "The Emperor".into(),
                "The Empress".into(),
                "The Tower".into(),
                "The Star".into(),
                "The Moon".into(),
                "The Sun".into(),
            ],
            items: vec![
                "Sharp".into(),
                "Sturdy".into(),
                "Swift".into(),
                "Arcane".into(),
                "Vital".into(),
            ],
            team_passives: vec!["Aegis".into(), "War Drums".into(), "Reservoir".into()],
            banners: vec!["Rally".into(), "Bulwark".into(), "Resolve".into()],
            curated_offers: vec![],
        }
    }

    #[test]
    fn schedule_shape_and_budget_progression() {
        let sched = schedule();
        assert_eq!(sched.len(), 8, "two beats per week over four weeks");

        // Budget grows from 10 to 15 across the season.
        let total_delta: u32 = sched.iter().map(|b| b.budget_delta).sum();
        assert_eq!(STARTING_BUDGET + total_delta, 15);

        // Exactly two new-character beats (5 → ~7).
        let chars = sched
            .iter()
            .filter(|b| b.kind == BeatKind::Character)
            .count();
        assert_eq!(chars, 2);

        // Exactly one swap, and it is late in the season.
        let swap_positions: Vec<usize> = sched
            .iter()
            .enumerate()
            .filter(|(_, b)| b.kind == BeatKind::Swap)
            .map(|(i, _)| i)
            .collect();
        assert_eq!(swap_positions, vec![6]);
    }

    #[test]
    fn offers_are_deterministic_and_sized() {
        let p = pools();
        for beat in 0..beat_count() {
            let a = offers(beat, 42, &p);
            let b = offers(beat, 42, &p);
            assert_eq!(a, b, "same seed → same offers (beat {beat})");
            assert_eq!(a.len(), OFFER_COUNT, "trio per beat (beat {beat})");
            // Distinct options within a trio.
            let set: HashSet<&String> = a.iter().collect();
            assert_eq!(set.len(), a.len(), "no duplicate offers (beat {beat})");
        }
    }

    #[test]
    fn offers_vary_by_seed_and_draw_from_the_right_pool() {
        let p = pools();
        // Different seeds generally produce different trios.
        let a = offers(1, 1, &p);
        let b = offers(1, 2, &p);
        assert_ne!(a, b);

        // Beat 0 is a Banner beat — offers come from the banner pool.
        let banner_set: HashSet<&String> = p.banners.iter().collect();
        for opt in offers(0, 7, &p) {
            assert!(banner_set.contains(&opt), "banner offer {opt} not in pool");
        }
        // Beat 2 is a Character beat — offers come from the character pool.
        let char_set: HashSet<&String> = p.characters.iter().collect();
        for opt in offers(2, 7, &p) {
            assert!(char_set.contains(&opt), "character offer {opt} not in pool");
        }
    }

    #[test]
    fn small_pool_offers_all_without_duplicates() {
        let mut p = pools();
        p.banners = vec!["Rally".into(), "Bulwark".into()]; // fewer than OFFER_COUNT
        let o = offers(0, 3, &p);
        assert_eq!(o.len(), 2);
        let set: HashSet<&String> = o.iter().collect();
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn curated_offers_override_sampling_per_beat() {
        let mut p = pools();
        // Curate only beat 2 (Character); leave the rest to sampling.
        p.curated_offers = vec![
            vec![],                                      // 0 — sampled
            vec![],                                      // 1 — sampled
            vec!["The Tower".into(), "The Star".into()], // 2 — curated (verbatim)
        ];
        // Beat 2 returns exactly the curated list, regardless of seed.
        assert_eq!(offers(2, 1, &p), vec!["The Tower", "The Star"]);
        assert_eq!(offers(2, 999, &p), vec!["The Tower", "The Star"]);
        // An empty curated entry falls back to seeded sampling (full trio).
        assert_eq!(offers(1, 42, &p).len(), OFFER_COUNT);
        // A beat past the curated vec also falls back to sampling.
        assert_eq!(offers(3, 42, &p).len(), OFFER_COUNT);
        // Auto-resolve of a curated beat draws from the curated offers.
        let pick = resolve_missed_beat(2, 7, &p).unwrap();
        assert!(["The Tower", "The Star"].contains(&pick.choice.as_str()));
    }

    #[test]
    fn missed_non_swap_auto_picks_from_offers() {
        let p = pools();
        // Beat 1 is an Item beat — auto-resolves to one of its offers.
        let pick = resolve_missed_beat(1, 99, &p).expect("item beat auto-resolves");
        assert_eq!(pick.beat, 1);
        assert!(offers(1, 99, &p).contains(&pick.choice));
        // Deterministic.
        assert_eq!(resolve_missed_beat(1, 99, &p).unwrap().choice, pick.choice);
    }

    #[test]
    fn missed_swap_is_skipped() {
        let p = pools();
        assert!(
            resolve_missed_beat(6, 99, &p).is_none(),
            "an unclaimed swap is never auto-applied"
        );
    }

    #[test]
    fn unlocked_starts_with_roster_and_base_budget() {
        let starting = vec!["The Emperor".to_string(), "The Empress".to_string()];
        let u = unlocked(&starting, 0, &[]);
        assert_eq!(u.budget, STARTING_BUDGET);
        assert!(u.archetypes.contains("The Emperor"));
        assert!(u.archetypes.contains("The Empress"));
        assert!(u.aspects.is_empty());
        assert!(u.banner.is_none());
    }

    #[test]
    fn unlocked_grows_budget_with_revealed_beats() {
        let starting = vec!["The Emperor".to_string()];
        // After all 8 beats revealed, budget is the full 15.
        assert_eq!(unlocked(&starting, 8, &[]).budget, 15);
        // After the first two beats (Banner +0, Item +1), budget is 11.
        assert_eq!(unlocked(&starting, 2, &[]).budget, 11);
        // Revealing past the end is clamped.
        assert_eq!(unlocked(&starting, 99, &[]).budget, 15);
    }

    #[test]
    fn unlocked_routes_claims_by_beat_kind() {
        let starting = vec!["The Emperor".to_string()];
        let claimed = vec![
            DraftPick {
                beat: 0,
                choice: "Rally".into(),
            }, // Banner
            DraftPick {
                beat: 1,
                choice: "Sharp".into(),
            }, // Item
            DraftPick {
                beat: 2,
                choice: "The Tower".into(),
            }, // Character
            DraftPick {
                beat: 3,
                choice: "Aegis".into(),
            }, // TeamPassive
            DraftPick {
                beat: 6,
                choice: "The Star".into(),
            }, // Swap
        ];
        let u = unlocked(&starting, 8, &claimed);
        assert_eq!(u.banner.as_deref(), Some("Rally"));
        assert!(u.aspects.contains("Sharp"));
        assert!(u.archetypes.contains("The Tower"));
        assert!(u.archetypes.contains("The Star")); // swap replacement unlocked
        assert!(u.archetypes.contains("The Emperor")); // starter retained
        assert!(u.team_passives.contains("Aegis"));
    }

    #[test]
    fn unlocked_latest_banner_wins() {
        let starting: Vec<String> = vec![];
        // Two banner claims on beat 0 (re-pick) — the later one wins.
        let claimed = vec![
            DraftPick {
                beat: 0,
                choice: "Rally".into(),
            },
            DraftPick {
                beat: 0,
                choice: "Bulwark".into(),
            },
        ];
        assert_eq!(
            unlocked(&starting, 1, &claimed).banner.as_deref(),
            Some("Bulwark")
        );
    }
}
