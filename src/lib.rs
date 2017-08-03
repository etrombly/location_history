#![warn(missing_docs)]
//! Library to parse google location history data

extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate chrono;

use chrono::NaiveDateTime;

/// group of locations
pub type Locations = Vec<Location>;

/// methods used for locations
pub trait LocationsExt {
    /// calculate average time between locations
    fn average_time(&self) -> i64;

    /// find the closest Location to a datetime
    fn find_closest(&self, time: NaiveDateTime) -> Option<Location>;

    /// remove locations that are offset more than 300km/h from last location
    fn filter_outliers(self) -> Locations;
}

impl LocationsExt for Locations {
    fn average_time(&self) -> i64 {
        let mut time = 0;
        for i in 1..self.len() {
            time += self[i - 1].timestamp.timestamp() -
                self[i].timestamp.timestamp()
        }
        time / (self.len() as i64)
    }

    fn find_closest(&self, time: NaiveDateTime) -> Option<Location> {
        let result = self.binary_search_by(|x| x.timestamp.cmp(&time));
        let index = match result {
            Ok(x) => Some(x),
            // if this is 0 or the len of locations return None
            Err(x) => {
                if x > 0 && x < self.len() {
                    Some(x)
                } else {
                    None
                }
            }
        };

        if let Some(x) = index {
            if x < self.len() {
                return Some(self[x]);
            }
        }
        None
    }

    fn filter_outliers(self) -> Locations {
        let mut tmp: Vec<Location> = vec![self[0]];
        for location in self.into_iter() {
            if location.speed_kmh(&tmp[tmp.len() - 1]) < 300.0 {
                tmp.push(location);
            }
        }
        tmp
    }
}

/// deserialize location history
pub fn deserialize(from: &str) -> Locations {
    #[derive(Deserialize)]
    struct LocationList {
        locations: Vec<Location>
    }

    let mut deserialized: LocationList = serde_json::from_str(from)
        .expect("Failed to deserialize");
    
    deserialized.locations.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
    deserialized.locations
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
/// Location sample parsed from LocationHistory.json
pub struct Location {
    #[serde(rename = "timestampMs", deserialize_with = "parse_date")]
    /// timestampt this location was sampled at, converted from milliseconds
    pub timestamp: NaiveDateTime,
    #[serde(rename = "latitudeE7", deserialize_with = "parse_location")]
    /// latitude, converted from lat E7
    pub latitude: f64,
    #[serde(rename = "longitudeE7", deserialize_with = "parse_location")]
    /// longitude, converted from long E7
    pub longitude: f64,
    /// accuracy of location sample in meters
    pub accuracy: i32,
    /// altitude in meters, if available
    pub altitude: Option<i32>,
}

impl Location {
    /// calculate the haversine distance between this and another location
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

    /// calculate the speed in km/h from this location to another location
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
        use ::LocationsExt;
        
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
        let locations = ::deserialize(&test_data).filter_outliers();
    }
}
