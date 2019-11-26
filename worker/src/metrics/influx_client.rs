// TODO:
// 1. Connect to influxdb with given credentials (consider futures)
// 2. Write a point to db
// 2.1 Validate points being written / maybe unittest (which requires reading)
// 3. Write multiple points to db

use influx_db_client::{Client, Point, Points, Value, Precision};

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
            // Setup a influx_db_client on constructing
            client: Client::new(format!("http://{}:{}", address, port),
                                String::from(database))
                .set_authentication(username, password),
        };
    }

    pub fn write(self, data: &str) {
        let mut point = point!("test1");
        point
            .add_field("foo", Value::String("bar".to_string()))
            .add_field("integer", Value::Integer(11))
            .add_field("float", Value::Float(22.3))
            .add_field("'boolean'", Value::Boolean(false));

        let point1 = Point::new("test1")
            .add_tag("tags", Value::String(String::from("\\\"fda")))
            .add_tag("number", Value::Integer(12))
            .add_tag("float", Value::Float(12.6))
            .add_field("fd", Value::String("'3'".to_string()))
            .add_field("quto", Value::String("\\\"fda".to_string()))
            .add_field("quto1", Value::String("\"fda".to_string()))
            .to_owned();

        let points = points!(point1, point);

        // if Precision is None, the default is second
        // Multiple write
        let _ = self.client.write_points(points, Some(Precision::Seconds), None).unwrap();

        // query, it's type is Option<Vec<Node>>
        let res = self.client.query("select * from test1", None).unwrap();
        println!("{:?}", res.unwrap()[0].series)
    }
}

#[cfg(test)]
mod tests {
    use crate::metrics::influx_client::InfluxClient;

    #[test]
    fn create_object() {
        let client = InfluxClient::new("localhost",
                                       8086,
                                       "root",
                                       "hunter2",
                                       "scraper_db");
        println!("yeet");
    }

    #[test]
    fn send_post() {
        let client = InfluxClient::new("localhost",
                                       8086,
                                       "root",
                                       "hunter2",
                                       "scraper_db");
        client.write("point");
        println!("yote");
    }
}