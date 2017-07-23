#![warn(missing_docs)]

extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate chrono;

use chrono::NaiveDateTime;

#[derive(Serialize, Deserialize)]
pub struct Locations {
    pub locations: Vec<Location>,
}

impl Locations {
    pub fn new(json: &str) -> Locations {
        let mut tmp: Locations = serde_json::from_str(json).unwrap();
        tmp.locations.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        tmp
    }

    pub fn average_time(&self) -> i64 {
        let mut time = 0;
        for i in 1..self.locations.len() {
            time += self.locations[i - 1].timestamp.timestamp() -
                self.locations[i].timestamp.timestamp()
        }
        time / (self.locations.len() as i64)
    }

    pub fn find_closest(&self, time: NaiveDateTime) -> Option<Location> {
        let result = self.locations.binary_search_by(|x| x.timestamp.cmp(&time));
        let index = match result {
            Ok(x) => Some(x),
            // if this is 0 or the len of locations return None
            Err(x) => {
                if x > 0 && x < self.locations.len() {
                    Some(x)
                } else {
                    None
                }
            }
        };
        if let Some(x) = index {
            if x < self.locations.len() {
                return Some(self.locations[x]);
            }
        }
        None
    }


    pub fn filter_outliers(&mut self) {
        let mut tmp = vec![self.locations[0]];
        for location in &self.locations {
            if location.speed_kmh(&tmp[tmp.len() - 1]) < 300.0 {
                tmp.push(*location);
            }
        }
        self.locations = tmp;
    }
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct Location {
    #[serde(rename = "timestampMs", deserialize_with = "parse_date")]
    pub timestamp: NaiveDateTime,
    #[serde(rename = "latitudeE7", deserialize_with = "parse_location")]
    pub latitude: f64,
    #[serde(rename = "longitudeE7", deserialize_with = "parse_location")]
    pub longitude: f64,
    pub accuracy: i32,
    pub altitude: Option<i32>,
}

impl Location {
    pub fn haversine_distance(&self, other: &Location) -> f64 {
        let long1 = self.longitude.to_radians();
        let long2 = other.longitude.to_radians();
        let lat1 = self.latitude.to_radians();
        let lat2 = other.latitude.to_radians();
        let dlon = long2 - long1;
        let dlat = lat2 - lat1;
        let a = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().asin();
        c * 6371000.0
    }

    pub fn speed_kmh(&self, other: &Location) -> f64 {
        let dist = self.haversine_distance(other);
        let time = self.timestamp.timestamp() - other.timestamp.timestamp();
        if time > 0 {
            let meter_second = dist / time as f64;
            meter_second * 3.6
        } else {
            dist / 1000.0
        }
    }
}

fn parse_date<'de, D>(de: D) -> Result<NaiveDateTime, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let deser_result: serde_json::Value = try!(serde::Deserialize::deserialize(de));
    match deser_result {
        serde_json::Value::String(ref s) => {
            Ok(NaiveDateTime::from_timestamp(
                s.parse::<i64>().unwrap() / 1000,
                0,
            ))
        }
        _ => Err(serde::de::Error::custom("Unexpected value")),
    }
}

fn parse_location<'de, D>(de: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let deser_result: serde_json::Value = try!(serde::Deserialize::deserialize(de));
    match deser_result {
        serde_json::Value::Number(ref i) => Ok(i.as_f64().unwrap() / 10000000.0),
        _ => Err(serde::de::Error::custom("Unexpected value")),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let test_data = r#"{"locations" : [ {
                            "timestampMs" : "1491801919709",
                            "latitudeE7" : 500373489,
                            "longitudeE7" : 83320934,
                            "accuracy" : 19,
                            "activitys" : [ {
                                "timestampMs" : "1491802042056",
                                "activities" : [ {
                                    "type" : "still",
                                    "confidence" : 100
                                } ]
                                }, {
                                "timestampMs" : "1491801923049",
                                "activities" : [ {
                                "type" : "still",
                                "confidence" : 100
                                } ]
                            } ]
                            }]}"#;
        let locations = ::Locations::new(&test_data);
    }
}
