use serde::Serialize;

#[cfg(test)]
mod tests {
    use super::Client;
    use std::io;

    #[test]
    fn test_client_ser() {
        let mut wtr = csv::Writer::from_writer(io::stdout());
        wtr.serialize(Client {
            client: 1,
            available: 25.1234,
            held: 5.,
            total: 30.1234,
            locked: false,
        })
        .unwrap();
    }

    #[test]
    fn test_client_new() {
        let _ = Client::new(1, 3450.123);
    }

    #[test]
    fn test_client_id() {
        let client = Client::new(1, 3450.123);
        assert_eq!(client.id(), 1);
    }

    #[test]
    fn test_client_avail_add_rm_hold_unhold_lock_unlock() {
        let mut client = Client::new(1, 3450.123);
        assert!(float_cmp::approx_eq!(
            f64,
            client.available(),
            3450.123,
            ulps = 1
        ));
        client.add(10.0004);
        assert!(float_cmp::approx_eq!(
            f64,
            client.available(),
            3460.1234,
            ulps = 1
        ));
        client.rm(10.0004);
        assert!(float_cmp::approx_eq!(
            f64,
            client.available(),
            3450.123,
            ulps = 1
        ));
        client.hold(10.003);
        assert!(float_cmp::approx_eq!(
            f64,
            client.available(),
            3440.12,
            ulps = 1
        ));
        client.hold(4000.);
        assert!(float_cmp::approx_eq!(f64, client.available(), 0., ulps = 1));
        client.unhold(0.123);
        assert!(float_cmp::approx_eq!(
            f64,
            client.available(),
            0.123,
            ulps = 1
        ));
        client.unhold(4000.);
        println!("{:?}", client);
        assert!(float_cmp::approx_eq!(
            f64,
            client.available(),
            3450.123,
            ulps = 1
        ));
        client.lock();
        assert!(float_cmp::approx_eq!(
            f64,
            client.available(),
            3450.123,
            ulps = 1
        ));
        client.unlock();
        assert!(float_cmp::approx_eq!(
            f64,
            client.available(),
            3450.123,
            ulps = 1
        ));
    }

    #[test]
    fn test_client_held_add_rm_hold_unhold_lock_unlock() {
        let mut client = Client::new(1, 3450.123);
        assert!(float_cmp::approx_eq!(f64, client.held(), 0., ulps = 1));
        client.add(10.0004);
        assert!(float_cmp::approx_eq!(f64, client.held(), 0., ulps = 1));
        client.rm(10.0004);
        assert!(float_cmp::approx_eq!(f64, client.held(), 0., ulps = 1));
        client.hold(10.003);
        assert!(float_cmp::approx_eq!(f64, client.held(), 10.003, ulps = 1));
        client.hold(4000.);
        assert!(float_cmp::approx_eq!(
            f64,
            client.held(),
            3450.123,
            ulps = 1
        ));
        client.unhold(0.123);
        assert!(float_cmp::approx_eq!(f64, client.held(), 3450., ulps = 1));
        client.unhold(4000.);
        assert!(float_cmp::approx_eq!(f64, client.held(), 0., ulps = 1));
        client.lock();
        assert!(float_cmp::approx_eq!(f64, client.held(), 0., ulps = 1));
        client.unlock();
        assert!(float_cmp::approx_eq!(f64, client.held(), 0., ulps = 1));
    }

    #[test]
    fn test_client_total_add_rm_hold_unhold_lock_unlock() {
        let mut client = Client::new(1, 3450.123);
        assert!(float_cmp::approx_eq!(
            f64,
            client.total(),
            3450.123,
            ulps = 1
        ));
        client.add(10.0004);
        assert!(float_cmp::approx_eq!(
            f64,
            client.total(),
            3460.1234,
            ulps = 1
        ));
        client.rm(10.0004);
        assert!(float_cmp::approx_eq!(
            f64,
            client.total(),
            3450.123,
            ulps = 1
        ));
        client.hold(10.003);
        assert!(float_cmp::approx_eq!(
            f64,
            client.total(),
            3450.123,
            ulps = 1
        ));
        client.hold(4000.);
        assert!(float_cmp::approx_eq!(
            f64,
            client.total(),
            3450.123,
            ulps = 1
        ));
        client.unhold(0.123);
        assert!(float_cmp::approx_eq!(
            f64,
            client.total(),
            3450.123,
            ulps = 1
        ));
        client.unhold(4000.);
        assert!(float_cmp::approx_eq!(
            f64,
            client.total(),
            3450.123,
            ulps = 1
        ));
        client.lock();
        assert!(float_cmp::approx_eq!(
            f64,
            client.total(),
            3450.123,
            ulps = 1
        ));
        client.unlock();
        assert!(float_cmp::approx_eq!(
            f64,
            client.total(),
            3450.123,
            ulps = 1
        ));
    }

    #[test]
    fn is_locked() {
        let mut client = Client::new(1, 3450.123);
        assert!(!client.is_locked());
        client.lock();
        assert!(client.is_locked());
        client.unlock();
        assert!(!client.is_locked());
    }
}

#[derive(Debug, Serialize)]
pub struct Client {
    client: u16,
    available: f64,
    held: f64,
    total: f64,
    locked: bool,
}

impl Client {
    pub fn new(id: u16, available: f64) -> Self {
        Self {
            client: id,
            available: available,
            held: 0.,
            total: available,
            locked: false,
        }
    }

    pub fn id(&self) -> u16 {
        return self.client;
    }

    pub fn available(&self) -> f64 {
        return self.available;
    }

    pub fn held(&self) -> f64 {
        return self.held;
    }

    pub fn total(&self) -> f64 {
        return self.total;
    }

    pub fn add(&mut self, amt: f64) {
        self.available += amt;
        self.total += amt;
    }

    pub fn rm(&mut self, mut amt: f64) {
        if amt > self.available {
            amt = self.available;
        };
        self.total -= amt;
        self.available -= amt;
    }

    pub fn hold(&mut self, mut amt: f64) {
        if amt > self.available {
            amt = self.available;
        }
        self.held += amt;
        self.available -= amt;
    }

    pub fn unhold(&mut self, mut amt: f64) {
        if amt > self.held {
            amt = self.held;
        }
        self.held -= amt;
        self.available += amt;
    }

    pub fn lock(&mut self) {
        self.locked = true;
    }

    pub fn unlock(&mut self) {
        self.locked = false;
    }

    pub fn is_locked(&self) -> bool {
        return self.locked;
    }
}
