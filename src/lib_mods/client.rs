use std::result;
use serde::Serialize;

#[cfg(test)]
mod tests {
    use super::{Client, Error};
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
        client.add(10.0004).unwrap();
        assert!(float_cmp::approx_eq!(
            f64,
            client.available(),
            3460.1234,
            ulps = 1
        ));
        client.rm(10.0004).unwrap();
        assert!(float_cmp::approx_eq!(
            f64,
            client.available(),
            3450.123,
            ulps = 1
        ));
        client.hold(10.003).unwrap();
        assert!(float_cmp::approx_eq!(
            f64,
            client.available(),
            3440.12,
            ulps = 1
        ));

        let res = client.hold(4000.).unwrap_err();
        match res {
            Error::InsufficientFunds(..) => (),
            _ => panic!("expected Error::InsufficentFunds"),
        }

        assert!(float_cmp::approx_eq!(f64, client.available(), 3440.12, ulps = 1));
        client.unhold(10.003).unwrap();
        assert!(float_cmp::approx_eq!(
            f64,
            client.available(),
            3450.123,
            ulps = 1
        ));

        let res = client.unhold(4000.).unwrap_err();
        match res {
            Error::InsufficientFunds(..) => (),
            _ => panic!("expected Error::InsufficentFunds"),
        }

        assert!(float_cmp::approx_eq!(
            f64,
            client.available(),
            3450.123,
            ulps = 1
        ));
        client.lock();
        assert!(client.add(100.).is_err());
        assert!(float_cmp::approx_eq!(
            f64,
            client.available(),
            3450.123,
            ulps = 1
        ));
        assert!(client.rm(100.).is_err());
        assert!(float_cmp::approx_eq!(
            f64,
            client.available(),
            3450.123,
            ulps = 1
        ));
        assert!(client.hold(100.).is_err());
        assert!(float_cmp::approx_eq!(
            f64,
            client.available(),
            3450.123,
            ulps = 1
        ));
        assert!(client.unhold(100.).is_err());
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
        client.add(10.0004).unwrap();
        assert!(float_cmp::approx_eq!(f64, client.held(), 0., ulps = 1));
        client.rm(10.0004).unwrap();
        assert!(float_cmp::approx_eq!(f64, client.held(), 0., ulps = 1));
        client.hold(10.003).unwrap();
        assert!(float_cmp::approx_eq!(f64, client.held(), 10.003, ulps = 1));

        let res = client.hold(4000.);
        match res.unwrap_err() {
            Error::InsufficientFunds(..) => (),
            _ => panic!("expecting Error::InsufficientFunds"),
        }

        assert!(float_cmp::approx_eq!(
            f64,
            client.held(),
            10.003,
            ulps = 1
        ));
        client.unhold(5.003).unwrap();
        assert!(float_cmp::approx_eq!(f64, client.held(), 5., ulps = 1));

        let res = client.unhold(4000.);
        match res.unwrap_err() {
            Error::InsufficientFunds(..) => (),
            _ => panic!("expecting Error::InsufficientFunds"),
        }

        client.unhold(5.).unwrap();
        assert!(float_cmp::approx_eq!(f64, client.held(), 0., ulps = 1));
        client.lock();
        assert!(float_cmp::approx_eq!(f64, client.held(), 0., ulps = 1));

        assert!(client.add(10.0004).is_err());
        assert!(float_cmp::approx_eq!(f64, client.held(), 0., ulps = 1));
        assert!(client.rm(10.0004).is_err());
        assert!(float_cmp::approx_eq!(f64, client.held(), 0., ulps = 1));
        assert!(client.hold(10.003).is_err());
        assert!(float_cmp::approx_eq!(f64, client.held(), 0., ulps = 1));
        assert!(client.unhold(0.123).is_err());
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
        client.add(10.0004).unwrap();
        assert!(float_cmp::approx_eq!(
            f64,
            client.total(),
            3460.1234,
            ulps = 1
        ));
        client.rm(10.0004).unwrap();
        assert!(float_cmp::approx_eq!(
            f64,
            client.total(),
            3450.123,
            ulps = 1
        ));
        client.hold(10.003).unwrap();
        assert!(float_cmp::approx_eq!(
            f64,
            client.total(),
            3450.123,
            ulps = 1
        ));

        let res = client.hold(4000.);
        match res.unwrap_err() {
            Error::InsufficientFunds(..) => (),
            _ => panic!("expecting Error::InsufficientFunds"),
        }

        assert!(float_cmp::approx_eq!(
            f64,
            client.total(),
            3450.123,
            ulps = 1
        ));
        client.unhold(0.123).unwrap();
        assert!(float_cmp::approx_eq!(
            f64,
            client.total(),
            3450.123,
            ulps = 1
        ));

        let res = client.unhold(4000.);
        match res.unwrap_err() {
            Error::InsufficientFunds(..) => (),
            _ => panic!("expecting Error::InsufficientFunds"),
        }

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

        assert!(client.add(10.0004).is_err());
        assert!(float_cmp::approx_eq!(
            f64,
            client.total(),
            3450.123,
            ulps = 1
        ));

        assert!(client.rm(10.0004).is_err());
        assert!(float_cmp::approx_eq!(
            f64,
            client.total(),
            3450.123,
            ulps = 1
        ));

        assert!(client.hold(10.003).is_err());
        assert!(float_cmp::approx_eq!(
            f64,
            client.total(),
            3450.123,
            ulps = 1
        ));

        assert!(client.unhold(0.123).is_err());
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

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("client {0} is locked")]
    Locked(u16),

    #[error("client {0} has insufficient funds of {1}")]
    InsufficientFunds(u16, f64),
}

#[derive(Debug, Serialize)]
pub struct Client {
    #[serde(rename = "client")]
    _client: u16,
    #[serde(rename = "available")]
    _available: f64,
    #[serde(rename = "held")]
    _held: f64,
    #[serde(rename = "total")]
    _total: f64,
    #[serde(rename = "locked")]
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

    pub fn add(&mut self, amt: f64) -> Result<()> {
        if self.is_locked() {
            return Err(Error::Locked(self.id()));
        }
        self._available += amt;
        self._total += amt;
        Ok(())
    }

    pub fn rm(&mut self, amt: f64) -> Result<()> {
        if self.is_locked() {
            return Err(Error::Locked(self.id()));
        } else if amt > self.available() {
            return Err(Error::InsufficientFunds(self.id(), self.available()));
        }
        self._total -= amt;
        self._available -= amt;
        Ok(())
    }

    pub fn hold(&mut self, amt: f64) -> Result<()> {
        if self.is_locked() {
            return Err(Error::Locked(self.id()));
        } else if amt > self.available() {
            return Err(Error::InsufficientFunds(self.id(), self.available()));
        }
        self._held += amt;
        self._available -= amt;
        Ok(())
    }

    pub fn unhold(&mut self, amt: f64) -> Result<()> {
        if self.is_locked() {
            return Err(Error::Locked(self.id()));
        } else if amt > self.held() {
            return Err(Error::InsufficientFunds(self.id(), self.held()));
        }

        self._held -= amt;
        self._available += amt;
        Ok(())
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
