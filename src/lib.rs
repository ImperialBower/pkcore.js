#![deny(clippy::all)]

//! Node.js bindings for [`pkcore`], built with napi-rs.
//!
//! See EPIC-85 in the `pkcore` repo for the binding contract. Every class here
//! is a thin one-line delegation to a `pkcore` type; no poker logic lives in
//! this crate.

use napi_derive::napi;
use std::str::FromStr;

use pkcore::analysis::eval::Eval as PkEval;
use pkcore::analysis::hand_rank::HandRank as PkHandRank;
use pkcore::arrays::seven::Seven as PkSeven;
use pkcore::card::Card as PkCard;
use pkcore::cards::Cards as PkCards;
use pkcore::casino::action::{PlayerAction as PkPlayerAction, TableAction as PkTableAction};
use pkcore::casino::dealer::{
    Dealer as PkDealer, DealerAction as PkDealerAction, DealerError as PkDealerError,
};
use pkcore::casino::equity::seat_equity::SeatEquity as PkSeatEquity;
use pkcore::casino::equity::seatbit::Seatbit as PkSeatbit;
use pkcore::casino::game::ForcedBets as PkForcedBets;
use pkcore::casino::session::{PokerSession as PkPokerSession, SessionStep as PkSessionStep};
use pkcore::casino::table::{
    Player as PkPlayer, Seat as PkSeat, Seats as PkSeats, Table as PkTable,
};
use pkcore::casino::winnings::{PotWin as PkPotWin, Winnings as PkWinnings};
use pkcore::rank::Rank as PkRank;
use pkcore::suit::Suit as PkSuit;
use pkcore::Pile;

/// A JS error whose `.code` is a `pkcore` error-variant name.
///
/// `napi::Error<S>` is generic over its status, and napi-rs feeds
/// `status.as_ref()` straight to `napi_create_error` as the JS `code`
/// (`napi-3.12.2/src/error.rs:1573,1597`). Using `String` instead of the
/// default `Status` is therefore the supported way to carry a domain error
/// code, which the default `napi::Error` cannot do -- it can only report the
/// fixed N-API status names such as `InvalidArg`.
/// NOTE: this alias is documentation only. The `#[napi]` macro matches the
/// literal token `Result<..>` in a signature to detect a fallible method, so
/// every fallible binding must spell out `Result<Self, napi::Error<String>>`
/// rather than use this alias.
pub type PkResult<T> = std::result::Result<T, napi::Error<String>>;

/// Converts any `Debug`-only `pkcore` error into a JS `Error` with a
/// `.code` of the variant name and a `.message` of the full `Debug` form.
///
/// `pkcore`'s `PKError` is `#[non_exhaustive]` and implements only `Debug`, so
/// the variant name has to be recovered from that form. See EPIC-85 Phase 3
/// item 5 for the `Dealer` version of this.
fn pk_err<E: std::fmt::Debug>(err: E) -> napi::Error<String> {
    let debug = format!("{err:?}");
    let code = debug
        .split(['(', ' ', '{'])
        .next()
        .unwrap_or("PkError")
        .to_string();
    napi::Error::new(code, debug)
}

/// Recovers a Rust enum variant name from its `Debug` form.
///
/// Used for error codes and event kinds. Cheaper to maintain than a match with
/// one arm per variant, and it does not go stale when `pkcore` adds one.
fn variant_name<T: std::fmt::Debug>(value: &T, fallback: &str) -> String {
    let debug = format!("{value:?}");
    debug
        .split(['(', ' ', '{'])
        .next()
        .filter(|name| !name.is_empty())
        .unwrap_or(fallback)
        .to_string()
}

/// Converts a `DealerError` into a JS `Error` whose `.code` is the variant name.
fn dealer_err(err: PkDealerError) -> napi::Error<String> {
    napi::Error::new(variant_name(&err, "DealerError"), format!("{err:?}"))
}

/// Clamps a JS number to a seat index.
fn seat_index(seat: u32) -> u8 {
    seat.min(u32::from(u8::MAX)) as u8
}

/// Clamps a JS number to a `pkcore` chip count.
///
/// JS has no unsigned integer, so a caller can hand us a negative. `pkcore`
/// chip fields are `usize`, so the floor is 0 rather than a wrap.
fn chips(amount: i64) -> usize {
    amount.max(0) as usize
}

/// The `pkcore` version this addon was compiled against.
///
/// Safe to read off this crate's own version: the version-lockstep rule in
/// `CLAUDE.md` keeps them identical.
#[napi]
pub fn pkcore_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// ---------------------------------------------------------------------------
// Rank
// ---------------------------------------------------------------------------

/// A card rank: ace down to deuce, plus `blank`.
///
/// PyO3 exposes these as class attributes (`Rank.ACE`); napi-rs has no
/// equivalent, so they are static methods (`Rank.ace()`).
#[napi]
#[derive(Clone, Copy)]
pub struct Rank(PkRank);

#[napi]
impl Rank {
    #[napi(factory)]
    pub fn ace() -> Self {
        Rank(PkRank::ACE)
    }
    #[napi(factory)]
    pub fn king() -> Self {
        Rank(PkRank::KING)
    }
    #[napi(factory)]
    pub fn queen() -> Self {
        Rank(PkRank::QUEEN)
    }
    #[napi(factory)]
    pub fn jack() -> Self {
        Rank(PkRank::JACK)
    }
    #[napi(factory)]
    pub fn ten() -> Self {
        Rank(PkRank::TEN)
    }
    #[napi(factory)]
    pub fn nine() -> Self {
        Rank(PkRank::NINE)
    }
    #[napi(factory)]
    pub fn eight() -> Self {
        Rank(PkRank::EIGHT)
    }
    #[napi(factory)]
    pub fn seven() -> Self {
        Rank(PkRank::SEVEN)
    }
    #[napi(factory)]
    pub fn six() -> Self {
        Rank(PkRank::SIX)
    }
    #[napi(factory)]
    pub fn five() -> Self {
        Rank(PkRank::FIVE)
    }
    #[napi(factory)]
    pub fn four() -> Self {
        Rank(PkRank::FOUR)
    }
    #[napi(factory)]
    pub fn trey() -> Self {
        Rank(PkRank::TREY)
    }
    #[napi(factory)]
    pub fn deuce() -> Self {
        Rank(PkRank::DEUCE)
    }
    #[napi(factory)]
    pub fn blank() -> Self {
        Rank(PkRank::BLANK)
    }

    #[napi(getter)]
    pub fn value(&self) -> u32 {
        self.0 as u32
    }

    /// The prime used by the Cactus Kev evaluator for this rank.
    #[napi]
    pub fn prime(&self) -> u32 {
        self.0.prime()
    }

    /// The Cactus Kev bit flag for this rank.
    #[napi]
    pub fn bits(&self) -> u32 {
        self.0.bits()
    }

    /// 0-based index: deuce is 0, ace is 12.
    #[napi]
    pub fn number(&self) -> u32 {
        self.0.number()
    }

    #[napi(js_name = "toString")]
    pub fn to_js_string(&self) -> String {
        self.0.to_string()
    }

    #[napi]
    pub fn equals(&self, other: &Rank) -> bool {
        self.0 == other.0
    }
}

// ---------------------------------------------------------------------------
// Suit
// ---------------------------------------------------------------------------

/// A card suit.
#[napi]
#[derive(Clone, Copy)]
pub struct Suit(PkSuit);

#[napi]
impl Suit {
    #[napi(factory)]
    pub fn spades() -> Self {
        Suit(PkSuit::SPADES)
    }
    #[napi(factory)]
    pub fn hearts() -> Self {
        Suit(PkSuit::HEARTS)
    }
    #[napi(factory)]
    pub fn diamonds() -> Self {
        Suit(PkSuit::DIAMONDS)
    }
    #[napi(factory)]
    pub fn clubs() -> Self {
        Suit(PkSuit::CLUBS)
    }
    #[napi(factory)]
    pub fn blank() -> Self {
        Suit(PkSuit::BLANK)
    }

    #[napi(getter)]
    pub fn value(&self) -> u32 {
        self.0 as u32
    }

    #[napi]
    pub fn symbol(&self) -> String {
        self.0.to_char_symbol().to_string()
    }

    #[napi]
    pub fn letter(&self) -> String {
        self.0.to_char_letter().to_string()
    }

    #[napi(js_name = "toString")]
    pub fn to_js_string(&self) -> String {
        self.0.to_string()
    }

    #[napi]
    pub fn equals(&self, other: &Suit) -> bool {
        self.0 == other.0
    }
}

// ---------------------------------------------------------------------------
// Card
// ---------------------------------------------------------------------------

/// A single playing card.
#[napi]
#[derive(Clone, Copy)]
pub struct Card(PkCard);

#[napi]
impl Card {
    /// Parses a card such as `"As"`, `"Kh"`, `"Q♦"`, `"2c"`.
    #[napi(factory)]
    pub fn parse(text: String) -> Result<Self, napi::Error<String>> {
        PkCard::from_str(&text).map(Card).map_err(pk_err)
    }

    #[napi(factory)]
    pub fn from_rank_suit(rank: &Rank, suit: &Suit) -> Self {
        Card(PkCard::new(rank.0, suit.0))
    }

    #[napi(getter)]
    pub fn rank(&self) -> Rank {
        Rank(self.0.get_rank())
    }

    #[napi(getter)]
    pub fn suit(&self) -> Suit {
        Suit(self.0.get_suit())
    }

    #[napi]
    pub fn is_dealt(&self) -> bool {
        Pile::is_dealt(&self.0)
    }

    /// The raw Cactus Kev encoding of this card.
    #[napi]
    pub fn as_u32(&self) -> u32 {
        self.0.as_u32()
    }

    #[napi(js_name = "toString")]
    pub fn to_js_string(&self) -> String {
        self.0.to_string()
    }

    #[napi]
    pub fn equals(&self, other: &Card) -> bool {
        self.0 == other.0
    }
}

// ---------------------------------------------------------------------------
// Cards
// ---------------------------------------------------------------------------

/// An ordered pile of cards.
#[napi]
#[derive(Clone)]
pub struct Cards(PkCards);

#[napi]
impl Cards {
    /// Parses a space-separated pile such as `"6♠ 6♥ 9♣"`.
    #[napi(factory)]
    pub fn parse(text: String) -> Result<Self, napi::Error<String>> {
        PkCards::from_str(&text).map(Cards).map_err(pk_err)
    }

    /// A full, ordered 52-card deck.
    #[napi(factory)]
    pub fn deck() -> Self {
        Cards(PkCards::deck())
    }

    #[napi(getter)]
    pub fn length(&self) -> u32 {
        self.0.len() as u32
    }

    #[napi]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[napi]
    pub fn contains(&self, card: &Card) -> bool {
        self.0.contains(&card.0)
    }

    #[napi]
    pub fn are_unique(&self) -> bool {
        self.0.are_unique()
    }

    #[napi]
    pub fn to_array(&self) -> Vec<Card> {
        self.0.iter().map(|card| Card(*card)).collect()
    }

    #[napi(js_name = "toString")]
    pub fn to_js_string(&self) -> String {
        self.0.to_string()
    }
}

// ---------------------------------------------------------------------------
// HandRank
// ---------------------------------------------------------------------------

/// The strength of a five-card poker hand.
///
/// `value` is the Cactus Kev rank: **lower is stronger**, 1 is a royal flush.
#[napi]
#[derive(Clone, Copy)]
pub struct HandRank(PkHandRank);

#[napi]
impl HandRank {
    /// The Cactus Kev rank value. Lower is stronger; 1 is a royal flush.
    #[napi(getter)]
    pub fn value(&self) -> u32 {
        u32::from(self.0.value)
    }

    /// The hand category, such as `"FullHouse"`.
    #[napi(getter)]
    pub fn name(&self) -> String {
        format!("{:?}", self.0.name)
    }

    /// The exact hand class, such as `"SixesOverFives"`.
    #[napi(getter)]
    pub fn class(&self) -> String {
        format!("{:?}", self.0.class)
    }

    #[napi(js_name = "toString")]
    pub fn to_js_string(&self) -> String {
        self.0.to_string()
    }

    #[napi]
    pub fn equals(&self, other: &HandRank) -> bool {
        self.0 == other.0
    }
}

// ---------------------------------------------------------------------------
// Eval
// ---------------------------------------------------------------------------

/// The best five-card hand inside a larger pile, plus its rank.
#[napi]
#[derive(Clone, Copy)]
pub struct Eval(PkEval);

#[napi]
impl Eval {
    /// Evaluates exactly seven cards, given as a string such as
    /// `"6♠ 6♥ 9♣ 6♦ 5♥ 5♠ 8♠"`.
    #[napi(factory)]
    pub fn from_seven(text: String) -> Result<Self, napi::Error<String>> {
        let seven = PkSeven::from_str(&text).map_err(pk_err)?;
        Ok(Eval(PkEval::from(seven)))
    }

    /// Evaluates exactly seven cards held in a [`Cards`] pile.
    #[napi(factory)]
    pub fn from_cards(cards: &Cards) -> Result<Self, napi::Error<String>> {
        let seven = PkSeven::try_from(cards.0.clone()).map_err(pk_err)?;
        Ok(Eval(PkEval::from(seven)))
    }

    #[napi(getter)]
    pub fn hand_rank(&self) -> HandRank {
        HandRank(self.0.hand_rank)
    }

    /// The winning five cards.
    #[napi(getter)]
    pub fn best_five(&self) -> Cards {
        Cards(Pile::cards(&self.0.hand))
    }

    #[napi(js_name = "toString")]
    pub fn to_js_string(&self) -> String {
        self.0.to_string()
    }
}

// ---------------------------------------------------------------------------
// ForcedBets
// ---------------------------------------------------------------------------

/// The blinds, ante, and bring-in a table charges.
///
/// Amounts are `i64` because `pkcore` stores chips as `usize`; napi-rs maps
/// `i64` to a plain JS `number`, exact below 2^53. See EPIC-85 Scope.
#[napi]
#[derive(Clone, Copy)]
pub struct ForcedBets(PkForcedBets);

#[napi]
impl ForcedBets {
    /// Blinds only. Ante and bring-in are zero.
    #[napi(constructor)]
    pub fn new(small_blind: i64, big_blind: i64) -> Self {
        ForcedBets(PkForcedBets::new(chips(small_blind), chips(big_blind)))
    }

    #[napi(factory)]
    pub fn with_ante(small_blind: i64, big_blind: i64, ante: i64) -> Self {
        ForcedBets(PkForcedBets::new_with_ante(
            chips(small_blind),
            chips(big_blind),
            chips(ante),
        ))
    }

    #[napi(getter)]
    pub fn small_blind(&self) -> i64 {
        self.0.small_blind as i64
    }

    #[napi(getter)]
    pub fn big_blind(&self) -> i64 {
        self.0.big_blind as i64
    }

    #[napi(getter)]
    pub fn ante(&self) -> i64 {
        self.0.ante as i64
    }

    #[napi(getter)]
    pub fn bring_in(&self) -> i64 {
        self.0.bring_in as i64
    }
}

// ---------------------------------------------------------------------------
// Player
// ---------------------------------------------------------------------------

/// One seated player's chips and state.
#[napi]
#[derive(Clone)]
pub struct Player(PkPlayer);

#[napi]
impl Player {
    #[napi(constructor)]
    pub fn new(handle: String, chips_amount: i64) -> Self {
        Player(PkPlayer::new_with_chips(handle, chips(chips_amount)))
    }

    #[napi(getter)]
    pub fn handle(&self) -> String {
        self.0.handle.clone()
    }

    /// Remaining stack: chips not yet committed this round.
    #[napi(getter)]
    pub fn chips(&self) -> i64 {
        self.0.chips as i64
    }

    /// Chips committed to the current betting round.
    #[napi(getter)]
    pub fn bet(&self) -> i64 {
        self.0.bet as i64
    }

    /// Chips committed across every round of the current hand.
    #[napi(getter)]
    pub fn chips_in_play(&self) -> i64 {
        self.0.chips_in_play as i64
    }

    /// Buy-in plus every reload since.
    #[napi(getter)]
    pub fn withdrawn(&self) -> i64 {
        self.0.withdrawn as i64
    }

    /// The player's state, such as `"Ready"` or `"Bet(100)"`.
    #[napi(getter)]
    pub fn state(&self) -> String {
        format!("{:?}", self.0.state)
    }

    /// Stack plus everything committed this hand.
    #[napi]
    pub fn total_chip_count(&self) -> i64 {
        self.0.total_chip_count() as i64
    }

    #[napi]
    pub fn is_active(&self) -> bool {
        self.0.is_active()
    }

    #[napi]
    pub fn is_all_in(&self) -> bool {
        self.0.is_all_in()
    }

    #[napi]
    pub fn is_in_hand(&self) -> bool {
        self.0.is_in_hand()
    }

    #[napi]
    pub fn has_bet(&self) -> bool {
        self.0.has_bet()
    }

    /// Adds chips to the stack and to `withdrawn`. Returns the new stack.
    #[napi]
    pub fn reload(&mut self, amount: i64) -> i64 {
        self.0.reload(chips(amount)) as i64
    }

    #[napi(js_name = "toString")]
    pub fn to_js_string(&self) -> String {
        self.0.to_string()
    }
}

// ---------------------------------------------------------------------------
// Seat
// ---------------------------------------------------------------------------

/// One chair at the table: a player plus their cards.
#[napi]
#[derive(Clone)]
pub struct Seat(PkSeat);

#[napi]
impl Seat {
    #[napi(constructor)]
    pub fn new(player: &Player) -> Self {
        Seat(PkSeat::new(player.0.clone()))
    }

    #[napi(getter)]
    pub fn player(&self) -> Player {
        Player(self.0.player.clone())
    }

    #[napi]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[napi]
    pub fn is_active(&self) -> bool {
        self.0.is_active()
    }

    #[napi]
    pub fn is_in_hand(&self) -> bool {
        self.0.is_in_hand()
    }

    #[napi(js_name = "toString")]
    pub fn to_js_string(&self) -> String {
        self.0.to_string()
    }
}

// ---------------------------------------------------------------------------
// Seats
// ---------------------------------------------------------------------------

/// The ring of seats at a table, in order.
#[napi]
#[derive(Clone, Default)]
pub struct Seats(PkSeats);

#[napi]
impl Seats {
    /// An empty ring. Add chairs with `push`.
    #[napi(constructor)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Builds a ring from player names, in order, each with the same stack.
    #[napi(factory)]
    pub fn from_names(names: Vec<String>, starting_chips: i64) -> Self {
        let stack = chips(starting_chips);
        Seats(PkSeats::new(
            names
                .into_iter()
                .map(|name| PkSeat::new(PkPlayer::new_with_chips(name, stack)))
                .collect(),
        ))
    }

    /// Appends a chair to the ring.
    #[napi]
    pub fn push(&mut self, seat: &Seat) {
        self.0 .0.push(seat.0.clone());
    }

    /// The number of chairs, occupied or not.
    #[napi(getter)]
    pub fn size(&self) -> u32 {
        u32::from(self.0.size())
    }

    #[napi]
    pub fn count_occupied(&self) -> u32 {
        u32::from(self.0.count_occupied())
    }

    #[napi]
    pub fn get_seat(&self, index: u32) -> Option<Seat> {
        u8::try_from(index)
            .ok()
            .and_then(|idx| self.0.get_seat(idx))
            .map(|seat| Seat(seat.clone()))
    }

    /// Every chip on the table, across all seats.
    #[napi]
    pub fn total_chip_count(&self) -> i64 {
        self.0.total_chip_count() as i64
    }

    /// The highest bet on the current street.
    #[napi]
    pub fn current_bet(&self) -> i64 {
        self.0.current_bet() as i64
    }

    #[napi]
    pub fn is_betting_complete(&self) -> bool {
        self.0.is_betting_complete()
    }

    #[napi(js_name = "toString")]
    pub fn to_js_string(&self) -> String {
        self.0.to_string()
    }
}

// ---------------------------------------------------------------------------
// Table
// ---------------------------------------------------------------------------

/// A poker table: the seats, the board, the pot, and the betting state.
#[napi]
#[derive(Clone)]
pub struct Table(PkTable);

#[napi]
impl Table {
    /// A no-limit hold'em table built from a ring of seats.
    #[napi(factory)]
    pub fn nlh_from_seats(seats: &Seats, forced: &ForcedBets) -> Self {
        Table(PkTable::nlh_from_seats(seats.0.clone(), forced.0))
    }

    /// The number of chairs at the table.
    #[napi]
    pub fn seat_count(&self) -> u32 {
        u32::from(self.0.seats.size())
    }

    #[napi(getter)]
    pub fn seats(&self) -> Seats {
        Seats(self.0.seats.clone())
    }

    #[napi(getter)]
    pub fn name(&self) -> String {
        self.0.name.clone()
    }

    #[napi(getter)]
    pub fn forced(&self) -> ForcedBets {
        ForcedBets(self.0.forced)
    }

    /// The betting phase, such as `"PreFlop"`.
    #[napi(getter)]
    pub fn phase(&self) -> String {
        format!("{:?}", self.0.phase)
    }

    #[napi(getter)]
    pub fn pot(&self) -> i64 {
        self.0.pot as i64
    }

    /// The highest bet on the current street.
    #[napi(getter)]
    pub fn bet(&self) -> i64 {
        self.0.bet as i64
    }

    /// The dealer button seat index.
    #[napi(getter)]
    pub fn button(&self) -> u32 {
        u32::from(self.0.button)
    }

    #[napi(getter)]
    pub fn board(&self) -> Cards {
        Cards(self.0.board.clone())
    }

    /// Every chip in the system: seats plus bets plus pot.
    #[napi]
    pub fn table_chip_count(&self) -> i64 {
        self.0.table_chip_count() as i64
    }

    /// The chip total snapshotted when the hand began. `end_hand` compares
    /// against it to catch chip-conservation failures.
    #[napi(getter)]
    pub fn hand_chip_total(&self) -> i64 {
        self.0.hand_chip_total as i64
    }

    #[napi]
    pub fn count_occupied_seats(&self) -> u32 {
        self.0.count_occupied_seats() as u32
    }

    #[napi]
    pub fn is_preflop(&self) -> bool {
        self.0.is_preflop()
    }

    #[napi]
    pub fn is_game_over(&self) -> bool {
        self.0.is_game_over()
    }

    #[napi]
    pub fn min_raise(&self) -> i64 {
        self.0.min_raise() as i64
    }

    #[napi(js_name = "toString")]
    pub fn to_js_string(&self) -> String {
        self.0.to_string()
    }
}

// ---------------------------------------------------------------------------
// SeatEquity
// ---------------------------------------------------------------------------

/// A chip amount and the seats that share it.
///
/// More than one seat means a split pot.
#[napi]
#[derive(Clone, Copy)]
pub struct SeatEquity(PkSeatEquity);

#[napi]
impl SeatEquity {
    #[napi(getter)]
    pub fn chips(&self) -> i64 {
        self.0.chips as i64
    }

    /// The seat indices sharing this amount.
    #[napi(getter)]
    pub fn seats(&self) -> Vec<u32> {
        (0..PkSeatbit::CAPACITY)
            .filter(|seat| self.0.seats.contains(*seat))
            .map(u32::from)
            .collect()
    }

    /// How many seats share this amount. Greater than 1 is a split pot.
    #[napi]
    pub fn count_ones(&self) -> u32 {
        self.0.count_ones() as u32
    }

    #[napi(js_name = "toString")]
    pub fn to_js_string(&self) -> String {
        self.0.to_string()
    }
}

// ---------------------------------------------------------------------------
// PotWin
// ---------------------------------------------------------------------------

/// One pot awarded to one or more seats, with the hand that won it.
#[napi]
#[derive(Clone, Copy)]
pub struct PotWin(PkPotWin);

#[napi]
impl PotWin {
    #[napi(getter)]
    pub fn equity(&self) -> SeatEquity {
        SeatEquity(self.0.equity)
    }

    /// The chips in this pot.
    #[napi(getter)]
    pub fn chips(&self) -> i64 {
        self.0.equity.chips as i64
    }

    /// The seat indices that won this pot.
    #[napi(getter)]
    pub fn seats(&self) -> Vec<u32> {
        SeatEquity(self.0.equity).seats()
    }

    #[napi(getter)]
    pub fn eval(&self) -> Eval {
        Eval(self.0.eval)
    }

    #[napi(getter)]
    pub fn hand_rank(&self) -> HandRank {
        HandRank(self.0.eval.hand_rank)
    }

    #[napi(js_name = "toString")]
    pub fn to_js_string(&self) -> String {
        self.0.to_string()
    }
}

// ---------------------------------------------------------------------------
// Winnings
// ---------------------------------------------------------------------------

/// Every pot awarded at the end of a hand, main pot first.
///
/// There is deliberately no `total()` here: summing the pots is one line of JS,
/// and `pkcore.js` keeps all arithmetic in `pkcore`.
#[napi]
#[derive(Clone)]
pub struct Winnings(PkWinnings);

#[napi]
impl Winnings {
    /// The number of pots awarded. More than one means a side pot.
    #[napi(getter)]
    pub fn length(&self) -> u32 {
        self.0.len() as u32
    }

    #[napi]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The main pot.
    #[napi]
    pub fn first(&self) -> PotWin {
        PotWin(self.0.first())
    }

    /// The first side pot.
    #[napi]
    pub fn second(&self) -> PotWin {
        PotWin(self.0.second())
    }

    #[napi]
    pub fn to_array(&self) -> Vec<PotWin> {
        self.0.vec().iter().map(|win| PotWin(*win)).collect()
    }

    #[napi(js_name = "toString")]
    pub fn to_js_string(&self) -> String {
        self.0.to_string()
    }
}

// ---------------------------------------------------------------------------
// TableAction
// ---------------------------------------------------------------------------

/// One entry in a table's event log.
///
/// The variants carry different payloads, so rather than binding forty classes
/// this exposes the shape `pkcore.py` settled on: a `kind` name plus the
/// optional `seat` and `amount` the event concerns.
#[napi]
#[derive(Clone, Copy)]
pub struct TableAction(PkTableAction);

#[napi]
impl TableAction {
    /// The event name, such as `"Bet"`, `"Fold"`, or `"DealtFlop"`.
    ///
    /// Read off the `Debug` form rather than a forty-arm match, so a new
    /// `pkcore` variant appears here without a change to this crate.
    #[napi(getter)]
    pub fn kind(&self) -> String {
        variant_name(&self.0, "TableAction")
    }

    /// The seat this event concerns, or `null`.
    #[napi(getter)]
    pub fn seat(&self) -> Option<u32> {
        self.0.get_seat().map(u32::from)
    }

    /// The chip amount this event concerns, or `null`.
    #[napi(getter)]
    pub fn amount(&self) -> Option<i64> {
        self.0.get_amount().map(|amount| amount as i64)
    }

    #[napi(js_name = "toString")]
    pub fn to_js_string(&self) -> String {
        self.0.to_string()
    }
}

// ---------------------------------------------------------------------------
// Dealer
// ---------------------------------------------------------------------------

/// Runs one hand at a table: seats players, deals, takes actions, pays out.
///
/// Every method that changes the table takes `&mut self`, matching `pkcore`'s
/// own `Dealer`. `pkcore.py` had to make the same switch when EPIC-83 removed
/// the interior-mutability table.
#[napi]
pub struct Dealer(PkDealer);

#[napi]
impl Dealer {
    /// A dealer with an empty no-limit hold'em table of `seat_count` chairs.
    #[napi(constructor)]
    pub fn new(forced: &ForcedBets, seat_count: u32) -> Self {
        Dealer(PkDealer::new(forced.0, seat_count.min(255) as u8))
    }

    /// A dealer for an already-built table.
    #[napi(factory)]
    pub fn from_table(table: &Table) -> Self {
        Dealer(PkDealer::from_table(table.0.clone()))
    }

    // ── Seating ─────────────────────────────────────────────────────────────

    /// Seats a player in the first open chair. Returns the seat index.
    #[napi]
    pub fn seat_player(&mut self, player: &Player) -> Result<u32, napi::Error<String>> {
        self.0
            .seat_player(player.0.clone())
            .map(u32::from)
            .map_err(dealer_err)
    }

    /// Seats a player in a named chair.
    #[napi]
    pub fn seat_player_at(
        &mut self,
        player: &Player,
        seat: u32,
    ) -> Result<(), napi::Error<String>> {
        self.0
            .seat_player_at(player.0.clone(), seat_index(seat))
            .map_err(dealer_err)
    }

    #[napi]
    pub fn remove_player(&mut self, seat: u32) -> Result<Player, napi::Error<String>> {
        self.0
            .remove_player(seat_index(seat))
            .map(Player)
            .map_err(dealer_err)
    }

    // ── Hand lifecycle ──────────────────────────────────────────────────────

    /// Shuffles, posts blinds, and deals hole cards.
    #[napi]
    pub fn start_hand(&mut self) -> Result<(), napi::Error<String>> {
        self.0.start_hand().map_err(dealer_err)
    }

    /// Collects bets and deals the next street.
    #[napi]
    pub fn advance_street(&mut self) -> Result<(), napi::Error<String>> {
        self.0.advance_street().map_err(dealer_err)
    }

    /// Resolves the showdown and pays out.
    #[napi]
    pub fn end_hand(&mut self) -> Result<Winnings, napi::Error<String>> {
        self.0.end_hand().map(Winnings).map_err(dealer_err)
    }

    // ── Player actions ──────────────────────────────────────────────────────

    #[napi]
    pub fn bet(&mut self, seat: u32, amount: i64) -> Result<(), napi::Error<String>> {
        self.act(PkDealerAction::Bet {
            seat: seat_index(seat),
            amount: chips(amount),
        })
    }

    #[napi]
    pub fn call(&mut self, seat: u32) -> Result<(), napi::Error<String>> {
        self.act(PkDealerAction::Call {
            seat: seat_index(seat),
        })
    }

    #[napi]
    pub fn check(&mut self, seat: u32) -> Result<(), napi::Error<String>> {
        self.act(PkDealerAction::Check {
            seat: seat_index(seat),
        })
    }

    /// Raises the total bet to `amount`, not by `amount`.
    #[napi]
    pub fn raise_to(&mut self, seat: u32, amount: i64) -> Result<(), napi::Error<String>> {
        self.act(PkDealerAction::Raise {
            seat: seat_index(seat),
            amount: chips(amount),
        })
    }

    #[napi]
    pub fn all_in(&mut self, seat: u32) -> Result<(), napi::Error<String>> {
        self.act(PkDealerAction::AllIn {
            seat: seat_index(seat),
        })
    }

    #[napi]
    pub fn fold(&mut self, seat: u32) -> Result<(), napi::Error<String>> {
        self.act(PkDealerAction::Fold {
            seat: seat_index(seat),
        })
    }

    /// Marks a seat ready for the next hand. Lobby management, not an in-hand
    /// action.
    #[napi]
    pub fn ready(&mut self, seat: u32) -> Result<(), napi::Error<String>> {
        self.act(PkDealerAction::Ready {
            seat: seat_index(seat),
        })
    }

    // ── Reading the table ───────────────────────────────────────────────────

    #[napi(getter)]
    pub fn table(&self) -> Table {
        Table(self.0.table.clone())
    }

    #[napi]
    pub fn table_id(&self) -> String {
        self.0.table_id().to_string()
    }

    #[napi]
    pub fn is_hand_in_progress(&self) -> bool {
        self.0.is_hand_in_progress()
    }

    #[napi]
    pub fn next_to_act(&self) -> u32 {
        u32::from(self.0.next_to_act())
    }

    #[napi]
    pub fn pot(&self) -> i64 {
        self.0.pot() as i64
    }

    /// The stack at a seat, or `null` if the chair is empty or out of range.
    #[napi]
    pub fn chips_at(&self, seat: u32) -> Option<i64> {
        self.0
            .chips_at(seat_index(seat))
            .map(|amount| amount as i64)
    }

    /// Every event so far, oldest first.
    ///
    /// A plain array, not a `TableLog` wrapper class: arrays are native in JS.
    #[napi]
    pub fn event_log(&self) -> Vec<TableAction> {
        self.0
            .event_log()
            .iter()
            .map(|action| TableAction(*action))
            .collect()
    }
}

impl Dealer {
    /// Shared body of every per-action method. Not exposed to JS: callers get
    /// the named methods instead of an action union to build.
    fn act(&mut self, action: PkDealerAction) -> Result<(), napi::Error<String>> {
        self.0.act(action).map_err(dealer_err)
    }
}

// ---------------------------------------------------------------------------
// PlayerAction
// ---------------------------------------------------------------------------

/// What a player chooses to do when it is their turn.
#[napi]
#[derive(Clone, Copy)]
pub struct PlayerAction(PkPlayerAction);

#[napi]
impl PlayerAction {
    #[napi(factory)]
    pub fn fold() -> Self {
        PlayerAction(PkPlayerAction::Fold)
    }

    #[napi(factory)]
    pub fn check() -> Self {
        PlayerAction(PkPlayerAction::Check)
    }

    #[napi(factory)]
    pub fn call() -> Self {
        PlayerAction(PkPlayerAction::Call)
    }

    #[napi(factory)]
    pub fn all_in() -> Self {
        PlayerAction(PkPlayerAction::AllIn)
    }

    /// Opens a bet of `amount` chips.
    #[napi(factory)]
    pub fn bet(amount: i64) -> Self {
        PlayerAction(PkPlayerAction::Bet(chips(amount)))
    }

    /// Raises the total bet **to** `amount`, not by `amount`.
    #[napi(factory)]
    pub fn raise(amount: i64) -> Self {
        PlayerAction(PkPlayerAction::Raise(chips(amount)))
    }

    /// One of `"Fold"`, `"Check"`, `"Call"`, `"Bet"`, `"Raise"`, `"AllIn"`.
    #[napi(getter)]
    pub fn kind(&self) -> String {
        variant_name(&self.0, "PlayerAction")
    }

    /// The chip amount for a bet or raise, otherwise `null`.
    #[napi(getter)]
    pub fn amount(&self) -> Option<i64> {
        match self.0 {
            PkPlayerAction::Bet(amount) | PkPlayerAction::Raise(amount) => Some(amount as i64),
            _ => None,
        }
    }

    #[napi(js_name = "toString")]
    pub fn to_js_string(&self) -> String {
        self.0.to_string()
    }
}

// ---------------------------------------------------------------------------
// SessionStep
// ---------------------------------------------------------------------------

/// What the session needs next.
///
/// `kind` is `"PlayerToAct"` (read `seat`), `"StreetAdvanced"`,
/// `"HandComplete"`, or `"Failed"` (read `error`).
#[napi]
#[derive(Clone)]
pub struct SessionStep(PkSessionStep);

#[napi]
impl SessionStep {
    #[napi(getter)]
    pub fn kind(&self) -> String {
        variant_name(&self.0, "SessionStep")
    }

    /// The seat that must act, or `null` unless `kind` is `"PlayerToAct"`.
    #[napi(getter)]
    pub fn seat(&self) -> Option<u32> {
        match self.0 {
            PkSessionStep::PlayerToAct(seat) => Some(u32::from(seat)),
            _ => None,
        }
    }

    /// Why the hand cannot continue, or `null` unless `kind` is `"Failed"`.
    ///
    /// A failed hand is **not** resolvable with `endHand`; call `abortHand`.
    #[napi(getter)]
    pub fn error(&self) -> Option<String> {
        match &self.0 {
            PkSessionStep::Failed(err) => Some(format!("{err:?}")),
            _ => None,
        }
    }

    #[napi]
    pub fn is_complete(&self) -> bool {
        matches!(self.0, PkSessionStep::HandComplete)
    }
}

// ---------------------------------------------------------------------------
// PokerSession
// ---------------------------------------------------------------------------

/// A multi-hand game on one table: deal, act, showdown, repeat.
///
/// `pkcore`'s `PokerSession::run_hand` takes a Rust closure. It is
/// deliberately **not** bound here. Calling a JS function from inside a
/// `&mut self` method would let that callback re-enter the same session object
/// and alias the mutable borrow, which is undefined behaviour. Drive the loop
/// from JS instead — it reads better anyway:
///
/// ```js
/// session.startHand()
/// let seat
/// while ((seat = session.nextActor()) !== null) {
///   session.applyAction(seat, PlayerAction.call())
/// }
/// const winnings = session.endHand()
/// ```
#[napi]
pub struct PokerSession(PkPokerSession);

#[napi]
impl PokerSession {
    #[napi(constructor)]
    pub fn new(table: &Table) -> Self {
        PokerSession(PkPokerSession::new(table.0.clone()))
    }

    /// Shuffles, posts blinds, deals, and increments `handNumber`.
    #[napi]
    pub fn start_hand(&mut self) -> Result<(), napi::Error<String>> {
        self.0.start_hand().map_err(pk_err)
    }

    /// The seat that must act next, or `null` when the betting is done.
    ///
    /// Deals the next street on its own when a betting round closes, so a
    /// caller only ever loops on this one method.
    #[napi]
    pub fn next_actor(&mut self) -> Result<Option<u32>, napi::Error<String>> {
        self.0
            .next_actor()
            .map(|seat| seat.map(u32::from))
            .map_err(pk_err)
    }

    #[napi]
    pub fn apply_action(
        &mut self,
        seat: u32,
        action: &PlayerAction,
    ) -> Result<(), napi::Error<String>> {
        self.0
            .apply_action(seat_index(seat), action.0)
            .map_err(pk_err)
    }

    /// What the session needs next, without changing whose turn it is.
    #[napi]
    pub fn next_step(&mut self) -> SessionStep {
        SessionStep(self.0.next_step())
    }

    /// Resolves the showdown and pays out.
    #[napi]
    pub fn end_hand(&mut self) -> Result<Winnings, napi::Error<String>> {
        self.0.end_hand().map(Winnings).map_err(pk_err)
    }

    /// Returns every committed chip and resets the table after a failed hand.
    /// Returns the number of chips returned.
    #[napi]
    pub fn abort_hand(&mut self) -> Result<i64, napi::Error<String>> {
        self.0.abort_hand().map(|n| n as i64).map_err(pk_err)
    }

    #[napi]
    pub fn is_hand_complete(&self) -> bool {
        self.0.is_hand_complete()
    }

    #[napi]
    pub fn is_hand_in_progress(&self) -> bool {
        self.0.is_hand_in_progress()
    }

    /// How many hands have been started.
    #[napi(getter)]
    pub fn hand_number(&self) -> u32 {
        self.0.hand_number
    }

    #[napi(getter)]
    pub fn table(&self) -> Table {
        Table(self.0.table.clone())
    }

    /// The full 52-card deck as shuffled at the start of the current hand, or
    /// `null` before the first hand.
    #[napi(getter)]
    pub fn shuffled_deck(&self) -> Option<String> {
        self.0.shuffled_deck_str.clone()
    }

    /// Applies new blinds at the start of the next hand.
    #[napi]
    pub fn set_blinds(&mut self, forced: &ForcedBets) {
        self.0.set_blinds(forced.0);
    }

    /// Removes players with no chips. Returns the seats emptied.
    #[napi]
    pub fn eliminate_busted(&mut self) -> Vec<u32> {
        self.0
            .eliminate_busted()
            .into_iter()
            .map(u32::from)
            .collect()
    }

    /// How many seated players still have chips.
    #[napi]
    pub fn count_funded(&self) -> u32 {
        self.0.count_funded() as u32
    }
}
