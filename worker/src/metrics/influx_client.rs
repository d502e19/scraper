use influx_db_client::{Client, Point, Points, Value, Precision, error};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct InfluxCredentials {
    pub username: String,
    pub password: String,
}

pub struct InfluxClient {
    address: String,
    port: i32,
    credentials: Option<InfluxCredentials>,
    database: String,
    client: Client,
}

impl InfluxClient {
    pub fn new(address: &str,
               port: i32,
               credentials: Option<InfluxCredentials>,
               database: &str,
    ) -> Self {
        let mut client = Client::new(format!("http://{}:{}", address, port),
                                 String::from(database));
        if let Some(creds) = &credentials {
            client = client.set_authentication(creds.username.clone(), creds.password.clone());
        }
        InfluxClient {
            address: String::from(address),
            port,
            credentials,
            database: String::from(database),
            client,
        }
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
            last_time: time,
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
            error!("Could not get system time", e);
            // Return zero if no time could be found to avoid breaking entire worker
            0
        }
    }
}