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

/// Struct to encapsulate state of a time measurement on task processing
pub struct TimeSession {
    point: Point,
    start_time: i64,
    last_time: i64,
}

impl TimeSession {
    pub fn new(measurement_name: &str, worker_instance: &str) -> TimeSession {
        let time = get_timestamp_millis();
        return TimeSession {
            point: Point::new(measurement_name)
                .add_timestamp(time)
                .add_tag("instance", Value::String(worker_instance.to_string()))
                .to_owned(),
            start_time: time,
            last_time: time,
        };
    }

    /// Adds a measuring field to Point with given parameters
    pub fn add_time_field(&mut self, field: &str) {
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

/// Struct to encapsulate state of discovered work count during task processing
pub struct CountSession {
    point: Point,
    last_count: i64,
}

impl CountSession {
    pub fn new(measurement_name: &str, worker_instance: &str) -> CountSession {
        let time = get_timestamp_millis();
        return CountSession {
            point: Point::new(measurement_name)
                .add_timestamp(time)
                .add_tag("instance", Value::String(worker_instance.to_string()))
                .to_owned(),
            last_count: 0,
        };
    }

    pub fn add_first_count_field(&mut self, field: &str, count: i64) {
        self.point.add_field(field, Value::Integer(count));
        self.last_count = count;
    }

    pub fn add_count_field(&mut self, field: &str, count: i64) {
        assert!(count <= self.last_count);
        self.point.add_field(field, Value::Integer(self.last_count - count));
        self.last_count = count;
    }

    pub fn add_final_count_field(&mut self, field: &str, count: i64) {
        self.point.add_field(field, Value::Integer(count));
    }

    pub fn write_point(self, client: &InfluxClient) {
        client.write_point(self.point);
    }
}

/// Write a task_url measurement with a given name, tagged by worker instance to Influx for
/// duplicate work statistics
pub fn write_task_url(url: &str, measurement: &str, worker_instance: &str, client: &InfluxClient) {
    client.write_point(Point::new(measurement)
        .add_timestamp(get_timestamp_millis())
        .add_tag("instance", Value::String(worker_instance.to_string()))
        .add_field("task_url", Value::String(url.to_string()))
        .to_owned());
}

/// Write a task_url measurement with a given name, error-kind, tagged by worker instance
/// to Influx for error statistics
pub fn write_task_error_url(url: &str, measurement: &str, error_kind: &str, worker_instance: &str, client: &InfluxClient) {
    client.write_point(Point::new(measurement)
        .add_timestamp(get_timestamp_millis())
        .add_tag("instance", Value::String(worker_instance.to_string()))
        .add_field("task_url", Value::String(url.to_string()))
        .add_tag("error_kind", Value::String(error_kind.to_string()))
        .to_owned());
}

/// Get current unix timestamp in milliseconds
pub fn get_timestamp_millis() -> i64 {
    // Return current unix time as milliseconds if possible, otherwise zero
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(time) => { return time.as_millis() as i64; }
        Err(e) => {
            error!("Could not get system time. {}", e);
            // Return zero if no time could be found to avoid breaking entire worker
            0
        }
    }
}