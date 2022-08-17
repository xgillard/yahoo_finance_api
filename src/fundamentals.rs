//! This module is meant to retrieve fundamental data from the yahoo api. 
//! 
//! Inspiration for this code is directly drawn from ranaroussi's excellent 
//! python binding for yahoo finance. 

use std::str::FromStr;

use serde_json::Value;

use crate::YahooError;

pub async fn get_json(url: &str) -> Result<serde_json::Value, YahooError> {
    let html = reqwest::get(url).await?.text().await?;
    let json_str = html.split("root.App.main =")
        .skip(1).next().ok_or(YahooError::InvalidJson)?
        .split("(this)").next().ok_or(YahooError::InvalidJson)?
        .split(";\n}").next().ok_or(YahooError::InvalidJson)?
        .trim();
        //.strip();
    let mut value = serde_json::Value::from_str(json_str)?;
    let data = &mut value["context"]["dispatcher"]["stores"]["QuoteSummaryStore"];
    Ok(data.take())
}

/// Check information here: 
/// https://github.com/ranaroussi/yfinance/blob/ec6279736b570815ec017691b428c90d910b1739/yfinance/base.py#L332
pub struct Ticker {
    fundamental: serde_json::Value
}
impl Ticker {
    pub async fn new(ticker: &str) -> Result<Self, YahooError> {
        let scrape = "https://finance.yahoo.com/quote";
        let ticker_url   = format!("{}/{}", scrape, ticker);
        let value = get_json(&ticker_url).await?;

        Ok(Self{fundamental: value})
    } 

    pub fn summary(&self) -> &Value{
        &self.fundamental["context"]["dispatcher"]["stores"]["QuoteSummaryStore"]
    }

    pub fn annual_basic_average_shares(&self) -> &Value {
        &self.fundamental["context"]["dispatcher"]["stores"]["QuoteTimeSeriesStore"]["timeSeries"]["annualBasicAverageShares"]
    }

    pub fn sustainability(&self) -> &Value {
        &self.fundamental["esgScores"]
    }

    // INFO
    pub fn info(&self) -> Info {
        Info{data: &self.fundamental}
    }
}

pub struct Info<'a> {
    data: &'a serde_json::Value 
}

impl <'a> Info<'a> {
    pub fn summary(&self) -> &Value {
        &self.data["summaryProfile"]
    }
    pub fn financial_data(&self) -> &Value {
        &self.data["financialData"]
    }
    pub fn quote_type(&self) -> &Value {
        &self.data["quoteType"]
    }
    pub fn default_key_statistics(&self) -> &Value {
        &self.data["defaultKeyStatistics"]
    }
    pub fn asset_profile(&self) -> &Value {
        &self.data["assetProfile"]
    }
    pub fn summary_detail(&self) -> &Value {
        &self.data["summaryDetail"]
    }
    pub fn currency(&self) -> &str {
        self.data["price"]["currency"].as_str().unwrap_or("---")
    }
    pub fn price(&self) -> f64 {
        let price = &self.data["price"]["regularMarketPrice"];
        if price.is_null() {
            self.data["price"]["regularMarketOpen"]["raw"].as_f64().unwrap_or(f64::NAN)
        } else {
            price["raw"].as_f64().unwrap_or(f64::NAN)
        }
    }
    /// For ETF only
    pub fn top_holdings(&self) -> &Value {
        &self.data["topHoldings"]
    }
}


#[cfg(test)]
mod test {
    use crate::YahooError;
    use super::*;

    #[tokio::test]
    async fn info() -> Result<(), YahooError>{
        let lcuw = Ticker::new("LCUW.DE").await?;
        println!("{}", lcuw.info().currency());

        let rgr = Ticker::new("RGR").await?;
        println!("{}", rgr.info().currency());

        Ok(())
    }
}