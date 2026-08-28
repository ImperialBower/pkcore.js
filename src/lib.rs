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
