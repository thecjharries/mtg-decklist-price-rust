use scryfall::{Card, search::prelude::*};

#[tokio::main]
#[cfg(not(tarpaulin_include))]
async fn main() {
    match find_cheapest_printing("Mountainqq").await {
        Some(card) => println!(
            "The cheapest price for Mountain is: ${}",
            card.prices.usd.unwrap().parse::<f64>().unwrap()
        ),
        None => println!("No price found for Mountain"),
    }
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

async fn find_cheapest_printing(card_name: &str) -> Option<Card> {
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

    let mut results = search_options.search().await.ok()?;
    match results.next().await {
        Some(Ok(card)) => Some(card),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_find_cheapest_printing() {
        let printing = find_cheapest_printing("mountain").await;
        assert!(printing.is_some());
        match printing {
            Some(card) => assert!(card.prices.usd.unwrap().parse::<f64>().unwrap() > 0.0),
            None => panic!("Expected a price for the card"),
        }
    }

    #[tokio::test]
    async fn test_find_cheapest_printing_nonexistent() {
        let printing = find_cheapest_printing("NonExistentCard").await;
        assert!(printing.is_none());
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
}
