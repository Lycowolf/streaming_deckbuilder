extern crate quicksilver;
extern crate json;

use quicksilver::prelude::*;
use std::collections::VecDeque;
use std::collections::HashMap;
use itertools::Itertools;
use std::iter;
use crate::game_logic::BoardState;
use crate::game_objects::*;


type CardFactory = HashMap<String, Card>;


fn cards_by_counts(factory: &CardFactory, counts: HashMap<String, usize>) -> Vec<Card> {
    counts.iter()
        .flat_map(|(key, num)| iter::repeat(key).take(*num))
        .filter_map(|key| factory.get(key))
        .cloned()
        .collect()
}

fn parse_deck(json: &serde_json::value::Value,  node_name: &str, card_factory: &CardFactory) -> Deck {
    let deck_node  = { json.get(node_name)
        .expect(format!("Deck node \"{}\" not found", node_name).as_str())
        .clone()
    };

    let data: HashMap<String, usize> =  serde_json::from_value(deck_node)
                                            .expect("Malformed deck list");

    Deck::from(cards_by_counts(card_factory, data))
}

fn parse_store(zone: BoardZone, json: &serde_json::value::Value, node: &str, factory: &CardFactory) -> Store {
    let source_node = json.get(node).expect(format!("store node {} not found", node).as_str()).clone();

    let store_type : StoreType = serde_json::from_value(source_node).expect("Malformed store description");

    match store_type.clone() {
        StoreType::Fixed{items} => {
            let cards = items.iter()
                            .filter_map(|name| factory.get(name))
                            .map(|card| card.clone())
                            .collect();

            Store { store_type: store_type,
                    menu: CardContainer{ zone: zone, cards: cards, size: None},
                    deck: None }
        },

        StoreType::Drafted{size, from_deck} => {
            let mut deck = parse_deck(json, &from_deck, factory);
            
            let cards = (0..size).filter_map(|_|deck.draw()).collect();

            Store { store_type: store_type,
                    menu: CardContainer{ zone: zone, cards: cards, size: Some(size)},
                    deck: Some(Box::new(deck)) }
        }
    }
}

fn container_counts(zone: BoardZone, json: &serde_json::value::Value, node: &str, factory: &CardFactory) -> CardContainer {
    let source_node = json.get(node).expect(format!("count node {} not found", node).as_str()).clone();
    let data: HashMap<String, usize> =  serde_json::from_value(source_node)
                                            .expect("Malformed node");

    CardContainer{ zone: zone,
                   cards: cards_by_counts(factory, data),
                   size: None }
}

pub fn load_board(filename: &str) -> BoardState {
        
    let deck_node_name = "test_deck";
    let buildings_node = "starter_buildings";
    let store_node = "build_store";
    let trade_row = "kaiju_store";
    let hand_size = 5;


    let file = load_file(filename)
        .wait()
        .expect("file should open read only");
    let json: serde_json::Value = serde_json::from_slice(file.as_slice())
        .expect("file should be proper JSON");
    
    let card_node = { json.get("cards")
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
        kaiju_zone: Box::new(kaiju)
    }
}