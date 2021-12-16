use std::collections::HashMap;

#[derive(Debug, Clone, serde::Deserialize)]
struct Config {
    region: RegionId,
    twitter_consumer_key: String,
    twitter_consumer_secret: String,
    twitter_access_token: String,
    twitter_access_secret: String,
}

#[derive(Debug, Copy, Clone)]
enum Intensity {
    Low,
    Moderate,
    High,
    VeryHigh,
}

impl<'de> serde::Deserialize<'de> for Intensity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "low" => Ok(Intensity::Low),
            "moderate" => Ok(Intensity::Moderate),
            "high" => Ok(Intensity::High),
            "very high" => Ok(Intensity::VeryHigh),
            _ => Err(serde::de::Error::custom(format!(
                "unknown intensity: {}",
                s
            ))),
        }
    }
}

#[derive(serde_repr::Serialize_repr, serde_repr::Deserialize_repr, PartialEq, Debug, Clone)]
#[repr(u16)]
enum RegionId {
    NorthScotland = 1,
    SouthScotland = 2,
    NorthWestEngland = 3,
    NorthEastEngland = 4,
    SouthYorkshire = 5,
    NorthWales = 6, // Merseyside and Chesire
    SouthWales = 7,
    WestMidlands = 8,
    EastMidlands = 9,
    EastEngland = 10,
    SouthWestEngland = 11,
    SouthEngland = 12,
    London = 13,
    SouthEastEngland = 14,
    England = 15,
    Scotland = 16,
    Wales = 17,
}

#[derive(Debug, serde::Deserialize)]
struct ErrorResponse {
    code: String,
    message: String,
}

#[derive(Debug, serde::Deserialize)]
struct DataItemResponse {
    shortname: String,
    data: Vec<ForecastResponse>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
enum RegionalResponse {
    Data(Vec<DataItemResponse>),
    Error(ErrorResponse),
}

#[derive(Debug, serde::Deserialize)]
struct ForecastResponse {
    #[serde(with = "carbon_date_format")]
    from: chrono::DateTime<chrono::Utc>,
    #[serde(with = "carbon_date_format")]
    to: chrono::DateTime<chrono::Utc>,
    intensity: IntensityResponse,
}

mod carbon_date_format {
    use chrono::TimeZone;
    use serde::Deserialize;

    const FORMAT: &str = "%Y-%m-%dT%H:%MZ";

    pub fn serialize<S>(
        date: &chrono::DateTime<chrono::Utc>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<chrono::DateTime<chrono::Utc>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        dbg!(&s);
        chrono::Utc
            .datetime_from_str(&s, FORMAT)
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, serde::Deserialize, Clone, Copy)]
struct IntensityResponse {
    index: Intensity,
    forecast: u32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();
    let config = Config {
        region: RegionId::London,
        twitter_consumer_key: todo!(),
        twitter_consumer_secret: todo!(),
        twitter_access_token: todo!(),
        twitter_access_secret: todo!(),
    };
    let url = format!(
        "https://api.carbonintensity.org.uk/regional/regionid/{}",
        config.clone().region as u16
    );
    log::trace!("{}", url);
    let resp: RegionalResponse = reqwest::get(&url).await?.json().await?;
    log::info!("{:#?}", resp);
    let intensity = match resp {
        RegionalResponse::Data(d) => d[0].data[0].intensity,
        RegionalResponse::Error(e) => {
            eprintln!("Error: {:?}", e);
            std::process::exit(1);
        }
    };
    tweet(config.clone(), intensity).await?;
    Ok(())
}

async fn tweet(
    config: Config,
    intensity: IntensityResponse,
) -> Result<(), Box<dyn std::error::Error>> {
    let con_token =
        egg_mode::KeyPair::new(config.twitter_consumer_key, config.twitter_consumer_secret);
    let access_token =
        egg_mode::KeyPair::new(config.twitter_access_token, config.twitter_access_secret);
    let token = egg_mode::Token::Access {
        consumer: con_token,
        access: access_token,
    };

    use egg_mode::tweet::DraftTweet;

    let post = DraftTweet::new(format!(
        "The current carbon intensity for London is {:?} with approximately {} gCO2/KWh.",
        intensity.index, intensity.forecast
    ))
    .send(&token)
    .await?;
    dbg!(post);

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_timestamp() {
        let j = r#"
{
    "data": [
        {
            "regionid": 13,
            "dnoregion": "UKPN London",
            "shortname": "London",
            "data": [
                {
                    "from": "2021-12-13T16:30Z",
                    "to": "2021-12-13T17:00Z",
                    "intensity": {
                        "forecast": 435,
                        "index": "very high"
                    },
                    "generationmix": [
                        {
                            "fuel": "biomass",
                            "perc": 0
                        },
                        {
                            "fuel": "coal",
                            "perc": 0.1
                        },
                        {
                            "fuel": "imports",
                            "perc": 84.1
                        },
                        {
                            "fuel": "gas",
                            "perc": 8.9
                        },
                        {
                            "fuel": "nuclear",
                            "perc": 2.4
                        },
                        {
                            "fuel": "other",
                            "perc": 0
                        },
                        {
                            "fuel": "hydro",
                            "perc": 0.2
                        },
                        {
                            "fuel": "solar",
                            "perc": 0
                        },
                        {
                            "fuel": "wind",
                            "perc": 4.3
                        }
                    ]
                }
            ]
        }
    ]
}
        "#;
        let jd = &mut serde_json::Deserializer::from_str(j);
        let res: RegionalResponse = serde_path_to_error::deserialize(jd).unwrap();
        insta::assert_debug_snapshot!(res);
    }

    #[test]
    fn test_error() {
        let j = r#"
{
    "error": {
        "code": "400 Bad Request",
        "message": "Please enter a valid region ID i.e. 1-17."
    }
}
        "#;
        let jd = &mut serde_json::Deserializer::from_str(j);
        let res: RegionalResponse = serde_path_to_error::deserialize(jd).unwrap();
        insta::assert_debug_snapshot!(res);
    }
}
