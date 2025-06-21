use scryfall::search::prelude::*;

#[tokio::main]
async fn main() {
    match find_cheapest_price("Mountainqq").await {
        Some(price) => println!("The cheapest price for Mountain is: ${}", price),
        None => println!("No price found for Mountain"),
    }
}

async fn find_cheapest_price(card_name: &str) -> Option<f64> {
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
    match results.next().await?.unwrap().prices.usd {
        Some(price) => match price.parse::<f64>() {
            Ok(price) => Some(price),
            Err(_) => None,
        },
        None => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_find_cheapest_price() {
        let price = find_cheapest_price("mountain").await;
        assert!(price.is_some());
        assert!(price.unwrap() > 0.0);
    }

    #[tokio::test]
    async fn test_find_cheapest_price_nonexistent() {
        let price = find_cheapest_price("NonExistentCard").await;
        assert!(price.is_none());
    }
}
