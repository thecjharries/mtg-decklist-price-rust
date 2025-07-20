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
}
