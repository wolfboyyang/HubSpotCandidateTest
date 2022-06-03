use chrono::NaiveDate;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
};

#[derive(Debug, Serialize, Deserialize)]
struct Partner {
    #[serde(rename = "firstName")]
    first_name: String,

    #[serde(rename = "lastName")]
    last_name: String,

    email: String,

    country: String,
    #[serde(rename = "availableDates")]
    available_dates: Vec<NaiveDate>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Results {
    partners: Vec<Partner>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Country {
    #[serde(rename = "attendeeCount")]
    attendee_count: usize,

    attendees: Vec<String>,

    name: String,

    #[serde(rename = "startDate")]
    start_date: Option<NaiveDate>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Post {
    countries: Vec<Country>,
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let user_key = "33936b6f1a56dd7d4e80f14a5def";
    let url = format!(
        "https://candidate.hubteam.com/candidateTest/v3/problem/dataset?userKey={user_key}"
    );
    let resp = reqwest::get(url).await?.json::<Results>().await?;
    //println!("parner count: {:#?}", resp.partners.len());

    /*
    let data = r#"
        {
            "partners": [
              {
                "firstName": "Darin",
                "lastName": "Daignault",
                "email": "ddaignault@hubspotpartners.com",
                "country": "United States",
                "availableDates": ["2017-05-03", "2017-05-06"]
              },
              {
                "firstName": "Crystal",
                "lastName": "Brenna",
                "email": "cbrenna@hubspotpartners.com",
                "country": "Ireland",
                "availableDates": ["2017-04-27", "2017-04-29", "2017-04-30"]
              },
              {
                "firstName": "Janyce",
                "lastName": "Gustison",
                "email": "jgustison@hubspotpartners.com",
                "country": "Spain",
                "availableDates": ["2017-04-29", "2017-04-30", "2017-05-01"]
              },
              {
                "firstName": "Tifany",
                "lastName": "Mozie",
                "email": "tmozie@hubspotpartners.com",
                "country": "Spain",
                "availableDates": ["2017-04-28", "2017-04-29", "2017-05-01", "2017-05-04"]
              },
              {
                "firstName": "Temple",
                "lastName": "Affelt",
                "email": "taffelt@hubspotpartners.com",
                "country": "Spain",
                "availableDates": ["2017-04-28", "2017-04-29", "2017-05-02", "2017-05-04"]
              },
              {
                "firstName": "Robyn",
                "lastName": "Yarwood",
                "email": "ryarwood@hubspotpartners.com",
                "country": "Spain",
                "availableDates": ["2017-04-29", "2017-04-30", "2017-05-02", "2017-05-03"]
              },
              {
                "firstName": "Shirlene",
                "lastName": "Filipponi",
                "email": "sfilipponi@hubspotpartners.com",
                "country": "Spain",
                "availableDates": ["2017-04-30", "2017-05-01"]
              },
              {
                "firstName": "Oliver",
                "lastName": "Majica",
                "email": "omajica@hubspotpartners.com",
                "country": "Spain",
                "availableDates": ["2017-04-28", "2017-04-29", "2017-05-01", "2017-05-03"]
              },
              {
                "firstName": "Wilber",
                "lastName": "Zartman",
                "email": "wzartman@hubspotpartners.com",
                "country": "Spain",
                "availableDates": ["2017-04-29", "2017-04-30", "2017-05-02", "2017-05-03"]
              },
              {
                "firstName": "Eugena",
                "lastName": "Auther",
                "email": "eauther@hubspotpartners.com",
                "country": "United States",
                "availableDates": ["2017-05-04", "2017-05-09"]
              }
            ]
          }
        "#;
    let resp: Results = serde_json::from_str(data).unwrap();
    */

    let date_map = resp
        .partners
        .iter()
        .fold(
            HashMap::<String, HashSet<NaiveDate>>::new(),
            |mut dates_map, partner| {
                if !dates_map.contains_key(&partner.country) {
                    dates_map.insert(partner.country.to_string(), HashSet::new());
                }
                let dates = dates_map.get_mut(&partner.country).unwrap();
                partner.available_dates.iter().for_each(|date| {
                    dates.insert(*date);
                });
                dates_map
            },
        )
        .into_iter()
        .fold(
            HashMap::<String, Option<NaiveDate>>::new(),
            |mut date_map, (country, posible_dates)| {
                let (date, count) = posible_dates
                    .iter()
                    .map(|date| {
                        let second_day = date.succ();
                        (
                            date,
                            resp.partners
                                .iter()
                                .filter(|partner| {
                                    partner.available_dates.contains(date)
                                        && partner.available_dates.contains(&second_day)
                                })
                                .count(),
                        )
                    })
                    .max_by(|a, b| match a.1.cmp(&b.1) {
                        Ordering::Equal => b.0.cmp(a.0),
                        order => order,
                    })
                    .unwrap();
                if count == 0 {
                    date_map.insert(country, None);
                } else {
                    date_map.insert(country, Some(*date));
                }
                date_map
            },
        );

    //for (country, date) in date_map {
    //    println!("{country}, {:?}", date);
    //}

    let country_map = date_map.iter().fold(
        HashMap::<String, Country>::new(),
        |mut countries, (country, start_date)| {
            if let Some(start_date) = start_date {
                let second_day = start_date.succ();
                let attendees: Vec<String> = resp
                    .partners
                    .iter()
                    .filter(|partner| {
                        partner.country == *country
                            && partner.available_dates.contains(start_date)
                            && partner.available_dates.contains(&second_day)
                    })
                    .map(|partner| partner.email.to_string())
                    .collect();
                countries.insert(
                    country.to_string(),
                    Country {
                        attendee_count: attendees.len(),
                        attendees,
                        name: country.to_string(),
                        start_date: Some(*start_date),
                    },
                );
            } else {
                countries.insert(
                    country.to_string(),
                    Country {
                        attendee_count: 0,
                        attendees: vec![],
                        name: country.to_string(),
                        start_date: None,
                    },
                );
            }
            countries
        },
    );

    let post_url =
        format!("https://candidate.hubteam.com/candidateTest/v3/problem/result?userKey={user_key}");

    let new_post = Post {
        countries: country_map
            .values()
            .cloned()
            .into_iter()
            .sorted_by(|a, b| a.name.cmp(&b.name))
            .collect(),
    };
    //for country in new_post.countries {
    //    println!("{:#?}", country);
    //}
    // println!("{:#?}", json!(new_post));

    let resp = reqwest::Client::new()
        .post(post_url)
        .json(&new_post)
        .send()
        .await?
        .text()
        .await?;

    println!("{:#?}", resp);

    Ok(())
}
