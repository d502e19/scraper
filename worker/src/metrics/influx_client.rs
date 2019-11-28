// TODO:
// 1. Connect to influxdb with given credentials (consider futures)
// 2. Write a point to db
// 2.1 Validate points being written / maybe unittest (which requires reading)
// 3. Write multiple points to db

use influx_db_client::{Client, Point, Points, Value, Precision, error};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct InfluxClient {
    address: String,
    port: i32,
    username: String,
    password: String,
    database: String,
    client: Client,
}

impl InfluxClient {
    pub fn new(address: &str,
               port: i32,
               username: &str,
               password: &str,
               database: &str) -> Self {
        return InfluxClient {
            address: String::from(address),
            port,
            username: String::from(username),
            password: String::from(password),
            database: String::from(database),
            client: Client::new(format!("http://{}:{}", address, port),
                                String::from(database))
                .set_authentication(username, password),
        };
    }

    /// Drop and create registered database, effectively resetting it.
    /// Requires sufficient user privileges to execute.
    pub fn reset_database(&self) -> Result<(), error::Error> {
        self.client.drop_database(self.database.as_ref())?;
        self.client.create_database(self.database.as_ref())?;
        Ok(())
    }

    /// Write a single point to a InfluxDB connection and log an error on failure
    pub fn write_point(&self, point: Point) {
        if let Err(e) = self.client.write_point(point, Some(Precision::Nanoseconds), None) {
            println!("Encountered error when trying to write point to InfluxDB {:?}", e);
            error!("Encountered error when trying to write point to InfluxDB {:?}", e)
        }
    }

    /// Write multiple points to a InfluxDB connection and log an error on failure
    pub fn write_points(&self, points: Points) {
        if let Err(e) = self.client.write_points(points, Some(Precision::Nanoseconds), None) {
            println!("Encountered error when trying to write points to InfluxDB {:?}", e);
            error!("Encountered error when trying to write points to InfluxDB {:?}", e)
        }
    }
}

/// Get current unix timestamp in milliseconds
pub fn get_timestamp_millis(enable: bool) -> i64 {
    if enable {
        // Return current unix time as milliseconds if possible, otherwise zero
        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(time) => { return time.as_millis() as i64; }
            Err(e) => {
                error!("Could not get system time");
                // Return zero if no time could be found to avoid breaking entire worker
                0
            }
        }
    } else {
        // Set start_time to zero if logging is unset. Probably carries a minuscule performance penalty.
        0
    }
}

/// Adds a measuring field to Point with given parameters if enabled. Returns 0 on disable.
pub fn add_data_point(point: &mut Point, field: &str, base_time: i64, enable: bool) -> i64 {
    if enable {
        let time = get_timestamp_millis(enable);
        //let time = get_timestamp_millis(enable) - base_time;
        point.add_field(field, Value::Integer(time - base_time));
        assert!(base_time >= 3600000, "Data_point time was longer than expected (1 hour) on {}, was: {}", field, time);
        time
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use crate::metrics::influx_client::InfluxClient;
    use influx_db_client::{Point, Value, Points};
    use std::time::{SystemTime, UNIX_EPOCH};

    /// Simply test that object is constructable
    #[test]
    #[ignore]
    fn create_object() {
        let client = InfluxClient::new("localhost",
                                       8086,
                                       "root",
                                       "hunter2",
                                       "scraper_db");
    }

    /// Sandbox test for InfluxDB implementation. Is ignored unless specifically requested.
    #[test]
    #[ignore]
    fn write_point() {
        let client = InfluxClient::new("localhost",
                                       8086,
                                       "root",
                                       "hunter2",
                                       "scraper_db");
        if let Err(e) = client.reset_database() {
            println!("Could not reset database. {}", e)
        }

        let point = Point::new("test1")
            //.add_timestamp(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as i64)
            .add_field("thing", Value::Boolean(false))
            .to_owned();
        let cloned_point = point.clone();
        client.write_point(point);
        let recv_point = client.client.query("select * from test1", None).unwrap();
        println!("{:?}", recv_point.unwrap()[0].series);
    }

    /// Sandbox test for InfluxDB implementation. Is ignored unless specifically requested.
    #[test]
    fn write_points() {
        let client = InfluxClient::new("localhost",
                                       8086,
                                       "root",
                                       "hunter2",
                                       "scraper_db");
        if let Err(e) = client.reset_database() {
            println!("Could not reset database. {}", e)
        }

        let points = Points::create_new((1..101)
            .into_iter()
            .map(|x|
                Point::new("test1")
                    .add_field("thing", Value::Integer(x))
                    .add_timestamp(139659585792080 + x)
                    .to_owned()
            )
            .collect::<Vec<Point>>()
        );

        client.write_points(points);
        let recv_point = client.client.query("select * from test1", None).unwrap();
        println!("{:?}", recv_point.unwrap()[0].series);
    }
}