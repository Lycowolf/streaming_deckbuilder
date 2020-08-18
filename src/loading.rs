extern crate quicksilver;
extern crate json;

use quicksilver::prelude::*;
use std::collections::VecDeque;
use std::collections::HashMap;
use itertools::Itertools;
use std::iter;
use crate::game_logic::{BoardState, GameplayState};
use crate::game_objects::*;
use crate::automaton::{AutomatonState, GameEvent};
use std::mem::take;
use futures::Async;
use derivative::*;
use quicksilver::Error as QuicksilverError;
use quicksilver::combinators::{join_all, JoinAll};

#[derive(Default, Debug)]
pub struct Assets {
    pub images: HashMap<String, Box<Image>>,
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

    Deck::from(cards_by_counts(card_factory, data))
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

pub fn load_board(json: serde_json::Value) -> BoardState {
    let deck_node_name = "test_deck";
    let buildings_node = "starter_buildings";
    let store_node = "build_store";
    let trade_row = "kaiju_store";
    let hand_size = 5;

    let card_node = {
        json.get("cards")
            .expect("file should have \"cards\" node")
            .clone()
    };
    let factory: CardFactory = serde_json::from_value(card_node)
        .expect("Malformed card list");

    let draw_deck = parse_deck(&json, deck_node_name, &factory);

    //let bs_node = { json.get("build_store").expect("build_store node not found").clone() };
    let build_store = parse_store(BoardZone::BuildStore, &json, store_node, &factory);

    //let ks_node = { json.get("kaiju_store").expect("kaiju_store node not found").clone() };
    let kaiju_store = parse_store(BoardZone::KaijuStore, &json, trade_row, &factory);

    let hand = CardContainer::new_sized(BoardZone::Hand, hand_size);

    let buildings = container_counts(BoardZone::Buildings, &json, buildings_node, &factory);

    let kaiju = CardContainer::new(BoardZone::Kaiju);

    println!("Loading done");

    BoardState {
        turn: 1,
        hand: Box::new(hand),
        deck: Box::new(draw_deck),
        globals: NumberMap::new(),
        stores: Box::new(vec!(build_store, kaiju_store)),
        buildings: Box::new(buildings),
        kaiju_zone: Box::new(kaiju),
    }
}

#[derive(Derivative, Default)]
#[derivative(Debug)]
pub struct LoadingState {
    board_state: BoardState,
    image_names: Vec<String>,
    #[derivative(Debug = "ignore")]
    // list of future images, wrapped in future
    loading_images: Option<JoinAll<Vec<Box<dyn Future<Item=Image, Error=QuicksilverError>>>>>,  // Option just to get Default
}

impl LoadingState {
    pub fn new() -> Box<Self> {
        let file = load_file("cards.json")
            .wait()
            .expect("file should open read only"); // TODO: do this asynchronously, too
        let json: serde_json::Value = serde_json::from_slice(file.as_slice())
            .expect("file should be proper JSON");

        let cards: CardFactory = serde_json::from_value(
            json.get("cards").expect("file should have \"cards\" node").clone()
        ).expect("malformed card list");

        let image_names = cards.values()
            .map(|v| v.image.clone())
            .unique()
            .collect::<Vec<String>>();
        println!("Loading images: {:?}", image_names);

        let loading_images = join_all(
            image_names.iter()
                .map(|i| Box::new(Image::load(i.clone())) as Box<dyn Future<Item=Image, Error=QuicksilverError>>)
                .collect::<Vec<Box<_>>>()
        );

        let board_state = load_board(json);

        Box::new(Self {
            board_state,
            image_names,
            loading_images: Some(loading_images),
        })
    }
}

impl AutomatonState for LoadingState {
    fn event(&mut self, event: GameEvent) -> Box<dyn AutomatonState> {
        Box::new(take(self))
    }

    fn update(&mut self) -> Box<dyn AutomatonState> {
        let result = self.loading_images.as_mut().unwrap().poll();
        match result {
            Ok(Async::Ready(images)) => {
                let mut loaded_images = HashMap::new();
                for (k, v) in self.image_names.iter().zip(images.iter()) {
                    loaded_images.insert(k.clone(), Box::new(v.clone()));
                }
                GameplayState::new_with_ui(take(self).board_state, Assets { images: loaded_images }) // TODO async load board
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