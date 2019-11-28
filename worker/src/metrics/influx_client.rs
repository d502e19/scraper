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
            error!("Encountered error when trying to write point to InfluxDB {:?}", e)
        }
    }

    /// Write multiple points to a InfluxDB connection and log an error on failure
    pub fn write_points(&self, points: Points) {
        if let Err(e) = self.client.write_points(points, Some(Precision::Nanoseconds), None) {
            error!("Encountered error when trying to write points to InfluxDB {:?}", e)
        }
    }
}

pub struct MetricSession {
    point: Point,
    start_time: i64,
    last_time: i64,
}

impl MetricSession {
    pub fn new(measurement_name: &str, worker_instance: &str) -> MetricSession {
        let time = get_timestamp_millis();
        return MetricSession {
            point: Point::new(measurement_name)
                .add_timestamp(time)
                .add_tag("instance", Value::String(worker_instance.to_string()))
                .to_owned(),
            start_time: time,
            last_time: 0 as i64,
        };
    }

    /// Adds a measuring field to Point with given parameters
    pub fn add_data_point(&mut self, field: &str) {
        let time = get_timestamp_millis();
        self.point.add_field(field, Value::Integer(time - self.last_time));
        // Save current time as last_time for next add_data_point
        self.last_time = time;
    }

    /// Add task finishing time to field 'task_finishing_time'
    pub fn add_finishing_time(&mut self) {
        self.point.add_field("task_finishing_time", Value::Integer(get_timestamp_millis() - self.start_time));
    }

    /// Write point to InfluxDB, consuming self in the process
    pub fn write_point(self, client: &InfluxClient) {
        client.write_point(self.point);
    }
}

/// Get current unix timestamp in milliseconds
pub fn get_timestamp_millis() -> i64 {
    // Return current unix time as milliseconds if possible, otherwise zero
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(time) => { return time.as_millis() as i64; }
        Err(e) => {
            error!("Could not get system time");
            // Return zero if no time could be found to avoid breaking entire worker
            0
        }
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
        /*
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
        */
    }

    /// Sandbox test for InfluxDB implementation. Is ignored unless specifically requested.
    #[test]
    #[ignore]
    fn write_points() {
        /*
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
        */
    }
}