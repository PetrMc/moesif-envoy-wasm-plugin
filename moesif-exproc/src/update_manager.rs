use chrono::{DateTime, Utc};

// A generic struct that manages data updates using ETag and timestamp
#[derive(Default)]
pub struct UpdateManager<T> {
    current_etag: String,
    old_etag: String,
    last_updated: i64,
    data: T,
}

impl<T> UpdateManager<T> {
    // Update the manager with new data and an ETag
    pub fn update(&mut self, etag: String, data: T) {
        self.old_etag = self.current_etag.clone();
        self.current_etag = etag;
        self.last_updated = Utc::now().timestamp();
        self.data = data;
    }

    // Check if the ETag indicates that the data has been updated
    pub fn is_updated(&self, etag: &str) -> bool {
        self.current_etag != etag && self.old_etag != etag
    }

    // Check if the data is expired based on a time-to-live (TTL)
    pub fn is_expired(&self, ttl: i64) -> bool {
        self.last_updated + ttl < Utc::now().timestamp()
    }

    // Retrieve a reference to the current data
    pub fn get_data(&self) -> &T {
        &self.data
    }
}
