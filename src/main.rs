use scryfall::{Card, Error, search::prelude::*};

lazy_static::lazy_static! {
    static ref CARD_NAME_PATTERN: regex::Regex = regex::Regex::new(r"(?i)^(?P<count>\d+)\s+(?P<name>.+?)\s*?$").unwrap();
}

#[tokio::main]
#[cfg(not(tarpaulin_include))]
async fn main() {
    print!("whoops")
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

// async fn find_cheapest_printing_of_list(cards: Vec<(u32, String)>) -> Result<Vec<Card>, Error> {
//     let mut cheapest_cards = Vec::new();
//     for (count, name) in cards {
//         if let Some(card) = find_cheapest_printing(&name).await {
//             cheapest_cards.push(card);
//         } else {
//             return Err(Error::NotFound(format!("No price found for {}", name)));
//         }
//     }
//     Ok(cheapest_cards)
// }

#[cfg(test)]
mod tests {
    use super::*;

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
}
