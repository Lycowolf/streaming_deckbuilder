extern crate quicksilver;
extern crate json;

use quicksilver::prelude::*;
use std::collections::VecDeque;
use std::collections::HashMap;
use itertools::Itertools;
use std::iter;
use crate::game_logic::{BoardState};
use crate::game_control::*;
use crate::game_objects::*;
use crate::ai::AI;
use crate::automaton::{AutomatonState, GameEvent};
use std::mem::take;
use futures::{Async};
use derivative::*;
use quicksilver::Error as QuicksilverError;
use quicksilver::combinators::{join_all, JoinAll, Join};
use std::rc::Rc;

pub const CARD_TITLE_FONT: &'static str = "Teko-Regular.ttf";
pub const CARD_BACKGROUND_IMG: &'static str = "card_bg.png";

#[derive(Derivative, Default)]
#[derivative(Debug)]
pub struct Assets {
    #[derivative(Debug = "ignore")]
    pub fonts: HashMap<String, Box<Font>>, // we borrow fonts to create new data: there's no reason to hold it
    pub images: HashMap<String, Rc<Image>>, // UI cards do hold reference to images
}

type CardFactory = HashMap<String, Card>;


fn cards_by_counts(factory: &CardFactory, counts: HashMap<String, usize>) -> Vec<Card> {
    counts.iter()
        .flat_map(|(key, num)| iter::repeat(key).take(*num))
        .filter_map(|key| factory.get(key))
        .cloned()
        .collect()
}

fn parse_deck(json: &serde_json::value::Value, node_name: &str, card_factory: &CardFactory) -> Deck {
    let deck_node = {
        json.get(node_name)
            .expect(format!("Deck node \"{}\" not found", node_name).as_str())
            .clone()
    };

    let data: HashMap<String, usize> = serde_json::from_value(deck_node)
        .expect("Malformed deck list");

    let mut deck = Deck::from(cards_by_counts(card_factory, data));
    deck.shuffle();
    deck
}

fn parse_store(zone: BoardZone, json: &serde_json::value::Value, node: &str, factory: &CardFactory) -> Store {
    let source_node = json.get(node).expect(format!("store node {} not found", node).as_str()).clone();

    let store_type: StoreType = serde_json::from_value(source_node).expect("Malformed store description");

    match store_type.clone() {
        StoreType::Fixed { items } => {
            let cards = items.iter()
                .filter_map(|name| factory.get(name))
                .map(|card| card.clone())
                .collect();

            Store {
                store_type: store_type,
                menu: CardContainer { zone: zone, cards: cards, size: None },
                deck: None,
            }
        }

        StoreType::Drafted { size, from_deck } => {
            let mut deck = parse_deck(json, &from_deck, factory);

            let cards = (0..size).filter_map(|_| deck.draw()).collect();

            Store {
                store_type: store_type,
                menu: CardContainer { zone: zone, cards: cards, size: Some(size) },
                deck: Some(Box::new(deck)),
            }
        }
    }
}

fn container_counts(zone: BoardZone, json: &serde_json::value::Value, node: &str, factory: &CardFactory) -> CardContainer {
    let source_node = json.get(node).expect(format!("count node {} not found", node).as_str()).clone();
    let data: HashMap<String, usize> = serde_json::from_value(source_node)
        .expect("Malformed node");

    CardContainer {
        zone: zone,
        cards: cards_by_counts(factory, data),
        size: None,
    }
}

pub fn load_players(json: &serde_json::Value) -> Vec<Player> {
    let player_node = json.get("players")
        .expect("file should have \"players\" node.")
        .clone();
    
    let mut players: Vec<Player> = serde_json::from_value(player_node)
        .expect("Malformed player node");

    let game_type = json.get("game_type")
        .expect("game type not specified")
        .as_str()
        .expect("game type not string");
    
    match game_type.to_lowercase().as_str() {
        "vs" => {
            assert_eq!(players.len(), 2, "For VS game, only 2 players are possible");
            players[0].opponent_idx = 1;
            players[1].opponent_idx = 0;
        },
        _ => panic!("Unknown game type")
    }

    players
}

pub fn load_board(json: &serde_json::Value, card_factory: &CardFactory, player: Player) -> BoardState {
    let store_node = "build_store";
    let trade_row = "kaiju_store";
    let hand_size = 5;

    let draw_deck = parse_deck(&json, &player.starting_deck, card_factory);

    //let bs_node = { json.get("build_store").expect("build_store node not found").clone() };
    let build_store = parse_store(BoardZone::BuildStore, &json, store_node, card_factory);

    //let ks_node = { json.get("kaiju_store").expect("kaiju_store node not found").clone() };
    let kaiju_store = parse_store(BoardZone::KaijuStore, &json, trade_row, card_factory);

    let hand = CardContainer::new_sized(BoardZone::Hand, hand_size);

    let buildings = container_counts(BoardZone::Buildings, &json, &player.starting_buildings, card_factory);

    let kaiju = CardContainer::new(BoardZone::Kaiju);

    let ai = match player.control {
        PlayerControl::Human => None,
        PlayerControl::AI => Some(AI::new())
    };

    println!("Loading done");

    BoardState {
        player: player,
        turn: 1,
        hand: Box::new(hand),
        deck: Box::new(draw_deck),
        globals: NumberMap::new(),
        stores: Box::new(vec!(build_store, kaiju_store)),
        buildings: Box::new(buildings),
        kaiju_zone: Box::new(kaiju),
        ai: ai
    }
}

/// Loading state: loads all assets to memory and passes them to GameplayState.
///
/// The asset loading in Quicksilver (as described in tutorial) is awkward: it requires conditional
/// execution whenever any asset is used. As we don't have large amount of data, it is more ergonomic
/// to just load them all to RAM and use them directly.
///
/// Loading in Quicksilver is internally done using Futures (that can, but don't have to
/// be wrapped in Assets). Futures can be nested using combinators (that themselves are Futures).
/// Every Future has a poll() method that returns Async::NotReady when it is not yet done, and
/// Async::Ready when its data are ready (i. e. loading is done).
/// It must not be called afterwards: it would panic.
///
/// It turns out this is perfect fit for our application: we combine all assets into single Future,
/// hook it into our event loop, polling it every update, while drawing a loading screen. When it
/// becomes ready, we construct a new State, pass it all the assets extracted from the Future and continue.
///
/// Sadly, it is complicated by the fact that Quicksilver re-exports Future trait and combinators, but
/// not the Async enum. As this enum comes from "futures" crate, we just install it in the exact same
/// version that Quicksilver uses and use that.

#[derive(Derivative, Default)]
#[derivative(Debug)]
pub struct LoadingState {
    board_states: Vec<BoardState>,
    image_names: Vec<String>,
    font_names: Vec<String>,
    #[derivative(Debug = "ignore")]
    // Option just to get Default
    loading: Option<
        Join<
            JoinAll<
                Vec<Box<dyn Future<Item=Font, Error=QuicksilverError>>>
            >,
            JoinAll<
                Vec<Box<dyn Future<Item=Image, Error=QuicksilverError>>>
            >
        >
    >,
}

impl LoadingState {
    pub fn new() -> Box<Self> {
        let font_names = vec![CARD_TITLE_FONT.to_string()];

        let file = load_file("cards_expanded.json")
            .wait()
            .expect("file should open read only"); // TODO: do this asynchronously, too
        let json: serde_json::Value = serde_json::from_slice(file.as_slice())
            .expect("file should be proper JSON");

        let cards: CardFactory = serde_json::from_value(
            json.get("cards").expect("file should have \"cards\" node").clone()
        ).expect("malformed card list");

        let mut image_names = cards.values()
            .map(|v| v.image.clone())
            .unique()
            .collect::<Vec<String>>();
        image_names.push(CARD_BACKGROUND_IMG.to_string());
        println!("Loading fonts {:?} and images: {:?}", font_names, image_names);

        let loading_images = join_all(
            font_names.iter()
                .map(|i| Box::new(Font::load(i.clone())) as Box<dyn Future<Item=Font, Error=QuicksilverError>>)
                .collect::<Vec<Box<_>>>()
        ).join(
            join_all(
                image_names.iter()
                    .map(|i| Box::new(Image::load(i.clone())) as Box<dyn Future<Item=Image, Error=QuicksilverError>>)
                    .collect::<Vec<Box<_>>>()
            )
        );

        let players = load_players(&json);
        let board_states = players.iter()
            .map(|p| load_board(&json, &cards, p.clone()))
            .collect();

        //let board_state = load_board(json);

        Box::new(Self {
            board_states,
            image_names,
            font_names,
            loading: Some(loading_images),
        })
    }
}

impl AutomatonState for LoadingState {
    fn event(&mut self, event: GameEvent) -> Box<dyn AutomatonState> {
        Box::new(take(self))
    }

    fn update(&mut self) -> Box<dyn AutomatonState> {
        let result = self.loading.as_mut().unwrap().poll();
        match result {
            // We use draining iterators to take ownership
            Ok(Async::Ready((mut fonts, mut images))) => {
                let mut loaded_fonts = HashMap::new();
                for (k, v) in self.font_names.drain((..)).zip(fonts.drain((..))) {
                    loaded_fonts.insert(k, Box::new(v));
                }

                let mut loaded_images = HashMap::new();
                for (k, v) in self.image_names.drain((..)).zip(images.drain((..))) {
                    loaded_images.insert(k, Rc::new(v));
                }

                let mut control_state = Box::new(GameControlState::new(
                    self.board_states.clone(),
                    Assets {
                        fonts: loaded_fonts,
                        images: loaded_images,
                    },
                )); // TODO async load board
                control_state.overtake()
            }
            Ok(Async::NotReady) => {
                Box::new(take(self))
            }
            Err(_) => { panic!("Can't load images") } // Value in Err is from another thread, and is not Sync. Yes, really.
        }
    }

    fn draw(&self, window: &mut Window) -> () {
        window.draw(&Circle::new((300, 300), 32), Col(Color::BLUE));
    }
}