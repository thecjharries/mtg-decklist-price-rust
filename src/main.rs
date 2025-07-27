use governor::{Quota, RateLimiter};
use lazy_static::lazy_static;
use regex::Regex;
use scryfall::{Card, Error, search::prelude::*};
use simple_logger::SimpleLogger;
use std::fs::File;
use std::io::Read;
use std::{num::NonZero, time::Duration};

lazy_static! {
    static ref CARD_NAME_PATTERN: Regex =
        Regex::new(r"(?i)^(?P<count>\d+)\s+(?P<name>.+?)\s*?$").unwrap();
}

#[tokio::main]
#[cfg(not(tarpaulin_include))]
async fn main() {
    SimpleLogger::new().init().unwrap();
    print!("whoops");
}

fn read_list_from_file(file_path: &str) -> Result<String, std::io::Error> {
    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents.trim().to_string())
}

fn sort_raw_card_list(cards: String) -> Vec<String> {
    let mut card_list: Vec<String> = cards
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.trim().to_string())
        .collect();
    card_list.sort();
    card_list
}

fn validate_card_list_entry(entry: &str) -> Result<(u32, String), String> {
    if let Some(caps) = CARD_NAME_PATTERN.captures(entry) {
        let count = caps
            .name("count")
            .unwrap()
            .as_str()
            .parse::<u32>()
            .map_err(|_| format!("Invalid count in entry: {}", entry))?;
        let name = caps.name("name").unwrap().as_str().trim().to_string();
        Ok((count, name))
    } else {
        Err(format!("Invalid card entry format: {}", entry))
    }
}

fn validate_card_list(entries: &[&str]) -> Result<Vec<(u32, String)>, String> {
    let mut valid_entries = Vec::new();
    for entry in entries {
        match validate_card_list_entry(entry) {
            Ok(valid_entry) => valid_entries.push(valid_entry),
            Err(err) => return Err(err),
        }
    }
    Ok(valid_entries)
}

async fn find_cheapest_printing(card_name: &str) -> Result<Card, Error> {
    // log::trace!("Searching for cheapest printing of: {}", card_name);
    let query = Query::And(vec![
        exact(card_name),
        not(PrintingIs::Digital),
        usd(gt(0.0)),
    ]);
    let mut search_options = SearchOptions::with_query(query);
    search_options
        .sort(SortOrder::Usd, SortDirection::Ascending)
        .extras(false)
        .variations(false)
        .unique(UniqueStrategy::Prints);

    let mut results = search_options.search().await?;
    match results.next().await {
        Some(card) => card,
        None => Err(Error::Other(format!("No price found for {}", card_name))),
    }
}

async fn find_cheapest_printing_of_list(
    cards: Vec<(u32, String)>,
    rate_milliseconds: u64,
) -> Result<Vec<(u32, Card)>, Error> {
    let mut cheapest_cards = Vec::new();
    let rate_limiter = RateLimiter::direct(
        Quota::with_period(Duration::from_millis(rate_milliseconds))
            .unwrap()
            .allow_burst(NonZero::new(1).unwrap()),
    );
    for (count, card_name) in cards {
        rate_limiter.until_ready().await;
        match find_cheapest_printing(&card_name).await {
            Ok(card) => {
                cheapest_cards.push((count, card));
            }
            Err(err) => {
                eprintln!("Error finding card {}: {}", card_name, err);
            }
        }
    }
    Ok(cheapest_cards)
}

async fn build_decklist(
    raw_card_list: String,
    rate_milliseconds: u64,
) -> Result<Vec<(u32, Card)>, Error> {
    let sorted_cards = sort_raw_card_list(raw_card_list);
    let entries: Vec<&str> = sorted_cards.iter().map(|s| s.as_str()).collect();
    let valid_entries = validate_card_list(&entries).map_err(|err| Error::Other(err))?;
    find_cheapest_printing_of_list(valid_entries, rate_milliseconds).await
}

fn compute_decklist_price(decklist: &[(u32, Card)]) -> f64 {
    decklist.iter().fold(0.0, |acc, (count, card)| {
        acc + card.prices.usd.as_ref().map_or(0.0, |price| {
            price.parse::<f64>().unwrap_or(0.0) * (*count as f64)
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_list_from_file() {
        use std::path::Path;
        let file_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("test/sample_decklist.txt");
        let result = read_list_from_file(file_path.to_str().unwrap_or(""));
        assert!(result.is_ok());
        let contents = result.unwrap();
        assert!(!contents.is_empty());
    }

    #[tokio::test]
    async fn test_find_cheapest_printing() {
        let printing = find_cheapest_printing("mountain").await;
        assert!(printing.is_ok());
        match printing {
            Ok(card) => assert!(card.prices.usd.unwrap().parse::<f64>().unwrap() > 0.0),
            Err(_) => panic!("Expected a price for the card"),
        }
    }

    #[tokio::test]
    async fn test_find_cheapest_printing_nonexistent() {
        let printing = find_cheapest_printing("NonExistentCard").await;
        assert!(printing.is_err());
    }

    #[test]
    fn test_sort_raw_card_list() {
        // https://moxfield.com/decks/K_R2ARDl_0W6Bs-mVi-vCA
        let input = "1 Adarkar Wastes
1 Alibou, Ancient Witness
1 Ancient Den
1 Angel of the Ruins
1 Arcane Signet
1 Astral Cornucopia
1 Battlefield Forge
1 Buried Ruin
1 Cascade Bluffs
1 Chain Reaction
1 Chaos Warp
1 Chrome Host Seedshark
1 Clifftop Retreat
1 Cloud Key
1 Command Tower
1 Coretapper
1 Crystalline Crawler



1 Moxite Refinery
1 Mystic Monastery
1 Organic Extinction
1 Patrolling Peacemaker
1 Pentad Prism
1 Phyrexian Metamorph
3 Plains
1 Cyberdrive Awakener
1 Darksteel Reactor
1 Deepglow Skate
1 Depthshaker Titan
1 Dispatch
1 Empowered Autogenerator
1 Emry, Lurker of the Loch
1 Enthusiastic Mechanaut
1 Etched Oracle
1 Etherium Sculptor
1 Gavel of the Righteous
1 Glacial Fortress
1 Everflowing Chalice
1 Evolving Wilds
1 Exotic Orchard
1 Experimental Augury
1 Fumigate
1 Glittering Massif
1 Golem Foundry
1 Great Furnace
1 Hangarback Walker
1 Insight Engine
1 Irrigated Farmland
3 Island
1 Jhoira, Weatherlight Captain
1 Kappa Cannoneer
1 Karn's Bastion
1 Kilo, Apogee Mind
1 Lonely Sandbar
1 Long-Range Sensor
1 Lux Artillery
1 Lux Cannon
1 Mindless Automaton
3 Mountain
1 Pull from Tomorrow



1 Radiant Summit
1 Razortide Bridge
1 Resourceful Defense
1 Ripples of Potential
1 Rugged Prairie
1 Rustvale Bridge
1 Seat of the Synod
1 Secluded Steppe
1 Shivan Reef
1 Silverbluff Bridge
1 Skycloud Expanse
1 Sol Ring
1 Solar Array
1 Soul-Guide Lantern
1 Spire of Industry
1 Steel Overseer
1 Sulfur Falls
1 Surge Conductor
1 Swan Song
1 Swords to Plowshares
1 Tekuthal, Inquiry Dominus
1 Temple of Enlightenment
1 Temple of Epiphany
1 Temple of Triumph
1 Tezzeret's Gambit
1 The Mycosynth Gardens
1 Thirst for Knowledge
1 Thought Monitor
1 Threefold Thunderhulk
1 Thrummingbird
1 Titan Forge
1 Universal Surveillance
1 Uthros Research Craft
1 Wake the Past

1 Inspirit, Flagship Vessel";
        let expected = vec![
            "1 Adarkar Wastes",
            "1 Alibou, Ancient Witness",
            "1 Ancient Den",
            "1 Angel of the Ruins",
            "1 Arcane Signet",
            "1 Astral Cornucopia",
            "1 Battlefield Forge",
            "1 Buried Ruin",
            "1 Cascade Bluffs",
            "1 Chain Reaction",
            "1 Chaos Warp",
            "1 Chrome Host Seedshark",
            "1 Clifftop Retreat",
            "1 Cloud Key",
            "1 Command Tower",
            "1 Coretapper",
            "1 Crystalline Crawler",
            "1 Cyberdrive Awakener",
            "1 Darksteel Reactor",
            "1 Deepglow Skate",
            "1 Depthshaker Titan",
            "1 Dispatch",
            "1 Empowered Autogenerator",
            "1 Emry, Lurker of the Loch",
            "1 Enthusiastic Mechanaut",
            "1 Etched Oracle",
            "1 Etherium Sculptor",
            "1 Everflowing Chalice",
            "1 Evolving Wilds",
            "1 Exotic Orchard",
            "1 Experimental Augury",
            "1 Fumigate",
            "1 Gavel of the Righteous",
            "1 Glacial Fortress",
            "1 Glittering Massif",
            "1 Golem Foundry",
            "1 Great Furnace",
            "1 Hangarback Walker",
            "1 Insight Engine",
            "1 Inspirit, Flagship Vessel",
            "1 Irrigated Farmland",
            "1 Jhoira, Weatherlight Captain",
            "1 Kappa Cannoneer",
            "1 Karn's Bastion",
            "1 Kilo, Apogee Mind",
            "1 Lonely Sandbar",
            "1 Long-Range Sensor",
            "1 Lux Artillery",
            "1 Lux Cannon",
            "1 Mindless Automaton",
            "1 Moxite Refinery",
            "1 Mystic Monastery",
            "1 Organic Extinction",
            "1 Patrolling Peacemaker",
            "1 Pentad Prism",
            "1 Phyrexian Metamorph",
            "1 Pull from Tomorrow",
            "1 Radiant Summit",
            "1 Razortide Bridge",
            "1 Resourceful Defense",
            "1 Ripples of Potential",
            "1 Rugged Prairie",
            "1 Rustvale Bridge",
            "1 Seat of the Synod",
            "1 Secluded Steppe",
            "1 Shivan Reef",
            "1 Silverbluff Bridge",
            "1 Skycloud Expanse",
            "1 Sol Ring",
            "1 Solar Array",
            "1 Soul-Guide Lantern",
            "1 Spire of Industry",
            "1 Steel Overseer",
            "1 Sulfur Falls",
            "1 Surge Conductor",
            "1 Swan Song",
            "1 Swords to Plowshares",
            "1 Tekuthal, Inquiry Dominus",
            "1 Temple of Enlightenment",
            "1 Temple of Epiphany",
            "1 Temple of Triumph",
            "1 Tezzeret's Gambit",
            "1 The Mycosynth Gardens",
            "1 Thirst for Knowledge",
            "1 Thought Monitor",
            "1 Threefold Thunderhulk",
            "1 Thrummingbird",
            "1 Titan Forge",
            "1 Universal Surveillance",
            "1 Uthros Research Craft",
            "1 Wake the Past",
            "3 Island",
            "3 Mountain",
            "3 Plains",
        ];
        let sorted = sort_raw_card_list(input.to_string());
        assert_eq!(sorted, expected);
    }

    #[test]
    fn test_validate_card_list_entry() {
        let entry = "3 Mountain";
        let result = validate_card_list_entry(entry);
        assert!(result.is_ok());
        let (count, name) = result.unwrap();
        assert_eq!(count, 3);
        assert_eq!(name, "Mountain");

        let invalid_entry = "Mountain 3";
        let result = validate_card_list_entry(invalid_entry);
        assert!(result.is_err());

        let empty_entry = "";
        let result = validate_card_list_entry(empty_entry);
        assert!(result.is_err());

        let malformed_entry = "3";
        let result = validate_card_list_entry(malformed_entry);
        assert!(result.is_err());

        let no_count_entry = "Mountain";
        let result = validate_card_list_entry(no_count_entry);
        assert!(result.is_err());

        let whitespace_entry = "   ";
        let result = validate_card_list_entry(whitespace_entry);
        assert!(result.is_err());

        let float_entry = "3.0 Mountain";
        let result = validate_card_list_entry(float_entry);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_card_list() {
        let entries = vec!["3 Mountain", "2 Island", "1 Plains"];
        let result = validate_card_list(&entries);
        assert!(result.is_ok());
        let valid_entries = result.unwrap();
        assert_eq!(valid_entries.len(), 3);
        assert_eq!(valid_entries[0].0, 3);
        assert_eq!(valid_entries[0].1, "Mountain");
        assert_eq!(valid_entries[1].0, 2);
        assert_eq!(valid_entries[1].1, "Island");
        assert_eq!(valid_entries[2].0, 1);
        assert_eq!(valid_entries[2].1, "Plains");

        let invalid_entries = vec!["3 Mountain", "Island", "1 Plains"];
        let result = validate_card_list(&invalid_entries);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_find_cheapest_printing_of_list() {
        let cards = vec![
            (3, "Mountain".to_string()),
            (2, "Island".to_string()),
            (1, "Plains".to_string()),
        ];
        let result = find_cheapest_printing_of_list(cards, 200).await;
        assert!(result.is_ok());
        let cheapest_cards = result.unwrap();
        assert_eq!(cheapest_cards.len(), 3);
        for (count, card) in cheapest_cards {
            assert!(
                card.prices.usd.is_some(),
                "Card {} should have a price",
                card.name
            );
            assert!(
                card.prices.usd.unwrap().parse::<f64>().unwrap() > 0.0,
                "Card {} should have a positive price",
                card.name
            );
            assert!(
                vec!["Mountain", "Island", "Plains"].contains(&card.name.as_str()),
                "Card name should be one of the expected names"
            );
            assert!(
                count > 0,
                "Count should be greater than 0 for card {}",
                card.name
            );
        }
    }

    #[tokio::test]
    async fn test_build_decklist() {
        let raw_card_list = "3 Mountain\n2 Island\n1 Plains".to_string();
        let result = build_decklist(raw_card_list, 200).await;
        assert!(result.is_ok());
        let decklist = result.unwrap();
        assert_eq!(decklist.len(), 3);
        for (count, card) in decklist {
            assert!(
                card.prices.usd.is_some(),
                "Card {} should have a price",
                card.name
            );
            assert!(
                count > 0,
                "Count should be greater than 0 for card {}",
                card.name
            );
        }
    }

    #[tokio::test]
    async fn test_compute_decklist_price() {
        let raw_card_list = "3 Mountain\n2 Island\n1 Plains".to_string();
        let result = build_decklist(raw_card_list, 200).await;
        assert!(result.is_ok());
        let decklist = result.unwrap();
        let total_price = compute_decklist_price(&decklist);
        assert!(total_price > 0.0, "Total price should be greater than 0.0");
    }
}
