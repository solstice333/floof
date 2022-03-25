use serde::Serialize;

#[cfg(test)]
mod tests {
    use super::Client;
    use std::io;

    #[test]
    fn test_client_ser() {
        let mut wtr = csv::Writer::from_writer(io::stdout());
        wtr.serialize(Client {
            _client: 1,
            _available: 25.1234,
            _held: 5.,
            _total: 30.1234,
            _locked: false,
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
        assert!(!client.hold(4000.));
        assert!(float_cmp::approx_eq!(f64, client.available(), 3440.12, ulps = 1));
        client.unhold(10.003);
        assert!(float_cmp::approx_eq!(
            f64,
            client.available(),
            3450.123,
            ulps = 1
        ));
        assert!(!client.unhold(4000.));
        assert!(float_cmp::approx_eq!(
            f64,
            client.available(),
            3450.123,
            ulps = 1
        ));
        client.lock();
        client.add(100.);
        assert!(float_cmp::approx_eq!(
            f64,
            client.available(),
            3450.123,
            ulps = 1
        ));
        client.rm(100.);
        assert!(float_cmp::approx_eq!(
            f64,
            client.available(),
            3450.123,
            ulps = 1
        ));
        client.hold(100.);
        assert!(float_cmp::approx_eq!(
            f64,
            client.available(),
            3450.123,
            ulps = 1
        ));
        client.unhold(100.);
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
        assert!(!client.hold(4000.));
        assert!(float_cmp::approx_eq!(
            f64,
            client.held(),
            10.003,
            ulps = 1
        ));
        client.unhold(5.003);
        assert!(float_cmp::approx_eq!(f64, client.held(), 5., ulps = 1));
        assert!(!client.unhold(4000.));
        assert!(client.unhold(5.));
        assert!(float_cmp::approx_eq!(f64, client.held(), 0., ulps = 1));
        client.lock();
        assert!(float_cmp::approx_eq!(f64, client.held(), 0., ulps = 1));

        client.add(10.0004);
        assert!(float_cmp::approx_eq!(f64, client.held(), 0., ulps = 1));
        client.rm(10.0004);
        assert!(float_cmp::approx_eq!(f64, client.held(), 0., ulps = 1));
        client.hold(10.003);
        assert!(float_cmp::approx_eq!(f64, client.held(), 0., ulps = 1));
        client.unhold(0.123);
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
        assert!(!client.hold(4000.));
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
        assert!(!client.unhold(4000.));
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

        client.add(10.0004);
        assert!(float_cmp::approx_eq!(
            f64,
            client.total(),
            3450.123,
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

        client.unhold(0.123);
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
    _client: u16,
    _available: f64,
    _held: f64,
    _total: f64,
    _locked: bool,
}

impl Client {
    pub fn new(id: u16, available: f64) -> Self {
        Self {
            _client: id,
            _available: available,
            _held: 0.,
            _total: available,
            _locked: false,
        }
    }

    pub fn id(&self) -> u16 {
        return self._client;
    }

    pub fn available(&self) -> f64 {
        return self._available;
    }

    pub fn held(&self) -> f64 {
        return self._held;
    }

    pub fn total(&self) -> f64 {
        return self._total;
    }

    pub fn add(&mut self, amt: f64) -> bool {
        if self.is_locked() {
            return false;
        }
        self._available += amt;
        self._total += amt;
        return true;
    }

    pub fn rm(&mut self, amt: f64) -> bool {
        if self.is_locked() || amt > self.available() {
            return false;
        }
        self._total -= amt;
        self._available -= amt;
        return true;
    }

    pub fn hold(&mut self, mut amt: f64) -> bool {
        if self.is_locked() || amt > self._available {
            return false;
        }
        self._held += amt;
        self._available -= amt;
        return true;
    }

    pub fn unhold(&mut self, mut amt: f64) -> bool {
        if self.is_locked() || amt > self._held {
            return false;
        }
        self._held -= amt;
        self._available += amt;
        return true;
    }

    pub fn lock(&mut self) {
        self._locked = true;
    }

    pub fn unlock(&mut self) {
        self._locked = false;
    }

    pub fn is_locked(&self) -> bool {
        return self._locked;
    }
}
