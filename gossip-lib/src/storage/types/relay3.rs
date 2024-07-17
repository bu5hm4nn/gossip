use crate::error::Error;
use crate::globals::GLOBALS;
use nostr_types::{Event, Id, PublicKey, RelayInformationDocument, RelayUrl, Unixtime};
use serde::{Deserialize, Serialize};

// THIS IS HISTORICAL FOR MIGRATIONS AND THE STRUCTURES SHOULD NOT BE EDITED

/// A relay record
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Relay3 {
    /// The url
    pub url: RelayUrl,

    /// How many times we successfully connected
    pub success_count: u64,

    /// How many times we failed to connect, plus we also count when
    /// the relay drops us without us requesting that
    pub failure_count: u64,

    /// When we last connected to the relay
    pub last_connected_at: Option<u64>,

    /// When the relay last gave us an EOSE on the general feed
    pub last_general_eose_at: Option<u64>,

    /// What rank the user applied to this relay.
    /// Valid ranks go from 0 to 9, with a default of 3. 0 means do not use.
    pub rank: u64,

    /// If this should be hidden in the UI
    pub hidden: bool,

    /// What usage this relay provides to the user
    /// (hidden because 'advertise' may be set which would interfere with simple
    /// .cmp and zero tests)
    pub(in crate::storage) usage_bits: u64,

    /// The NIP-11 for this relay
    pub nip11: Option<RelayInformationDocument>,

    /// The last time we attempted to fetch the NIP-11 for this relay
    /// (in unixtime seconds)
    pub last_attempt_nip11: Option<u64>,

    /// If the user allows connection to this relay
    /// None: Ask (Default)
    /// Some(false): Never
    /// Some(true): Always
    pub allow_connect: Option<bool>,

    /// If the user allows this relay to AUTH them
    /// None: Ask (Default)
    /// Some(false): Never
    /// Some(true): Always
    pub allow_auth: Option<bool>,

    /// Avoid until this timestamp
    pub avoid_until: Option<Unixtime>,
}

impl Relay3 {
    pub const READ: u64 = 1 << 0; // 1
    pub const WRITE: u64 = 1 << 1; // 2
    const ADVERTISE: u64 = 1 << 2; // 4 // RETIRED
    pub const INBOX: u64 = 1 << 3; // 8            this is 'read' of kind 10002
    pub const OUTBOX: u64 = 1 << 4; // 16          this is 'write' of kind 10002
    pub const DISCOVER: u64 = 1 << 5; // 32
    pub const SPAMSAFE: u64 = 1 << 6; // 64
    pub const DM: u64 = 1 << 7; // 128             this is of kind 10050

    pub fn new(url: RelayUrl) -> Self {
        Self {
            url,
            success_count: 0,
            failure_count: 0,
            last_connected_at: None,
            last_general_eose_at: None,
            rank: 3,
            hidden: false,
            usage_bits: 0,
            nip11: None,
            last_attempt_nip11: None,
            allow_connect: None,
            allow_auth: None,
            avoid_until: None,
        }
    }

    #[inline]
    pub fn get_usage_bits(&self) -> u64 {
        // Automatically clear any residual ADVERTISE bit
        // ( so that simple cmp() and =0 still work... but you should use
        //   the new has_any_usage_bit() instead to be safe )
        self.usage_bits & !Self::ADVERTISE
    }

    #[inline]
    pub fn get_usage_bits_for_sorting(&self) -> u64 {
        let mut output: u64 = 0;
        if self.has_usage_bits(Self::READ) {
            output |= 1 << 6;
        }
        if self.has_usage_bits(Self::WRITE) {
            output |= 1 << 5;
        }
        if self.has_usage_bits(Self::INBOX) {
            output |= 1 << 4;
        }
        if self.has_usage_bits(Self::OUTBOX) {
            output |= 1 << 3;
        }
        if self.has_usage_bits(Self::DM) {
            output |= 1 << 2;
        }
        // DISCOVER and SPAMSAFE shouldn't affect sort
        output
    }

    #[inline]
    pub fn set_usage_bits(&mut self, bits: u64) {
        self.usage_bits |= bits;
    }

    #[inline]
    pub fn clear_usage_bits(&mut self, bits: u64) {
        self.usage_bits &= !bits;
    }

    #[inline]
    pub fn adjust_usage_bit(&mut self, bit: u64, value: bool) {
        if value {
            self.set_usage_bits(bit);
        } else {
            self.clear_usage_bits(bit);
        }
    }

    #[inline]
    pub fn has_usage_bits(&self, bits: u64) -> bool {
        self.usage_bits & bits == bits
    }

    #[inline]
    pub fn has_any_usage_bit(&self) -> bool {
        let all = Self::READ | Self::WRITE | Self::INBOX | Self::OUTBOX | Self::DISCOVER | Self::DM;
        self.usage_bits & all != 0
    }

    #[inline]
    pub fn attempts(&self) -> u64 {
        self.success_count + self.failure_count
    }

    #[inline]
    pub fn success_rate(&self) -> f32 {
        let attempts = self.attempts();
        if attempts == 0 {
            return 0.5;
        } // unknown, so we put it in the middle
        self.success_count as f32 / attempts as f32
    }

    pub fn should_avoid(&self) -> bool {
        if let Some(when) = self.avoid_until {
            when >= Unixtime::now()
        } else {
            false
        }
    }

    pub fn is_good_for_advertise(&self) -> bool {
        if self.should_avoid() {
            return false;
        }

        self.has_usage_bits(Self::INBOX)
            || self.has_usage_bits(Self::OUTBOX)
            || self.has_usage_bits(Self::DISCOVER)
            || (self.rank > 0 && self.success_rate() > 0.50 && self.success_count > 15)
    }

    /// This generates a "recommended_relay_url" for an 'e' tag.
    pub fn recommended_relay_for_reply(reply_to: Id) -> Result<Option<RelayUrl>, Error> {
        let seen_on_relays: Vec<(RelayUrl, Unixtime)> =
            GLOBALS.storage.get_event_seen_on_relay(reply_to)?;

        let maybepubkey = GLOBALS.storage.read_setting_public_key();
        if let Some(pubkey) = maybepubkey {
            let my_inbox_relays: Vec<RelayUrl> =
                GLOBALS.storage.get_best_relays_min(pubkey, false, 0)?;

            // Find the first-best intersection
            for mir in &my_inbox_relays {
                for sor in &seen_on_relays {
                    if *mir == sor.0 {
                        return Ok(Some(mir.clone()));
                    }
                }
            }

            // Else fall through to seen on relays only
        }

        if let Some(sor) = seen_on_relays.first() {
            return Ok(Some(sor.0.clone()));
        }

        Ok(None)
    }

    pub fn choose_relays<F>(bits: u64, f: F) -> Result<Vec<Relay3>, Error>
    where
        F: Fn(&Relay3) -> bool,
    {
        GLOBALS
            .storage
            .filter_relays(|r| r.has_usage_bits(bits) && r.rank != 0 && !r.should_avoid() && f(r))
    }

    pub fn choose_relay_urls<F>(bits: u64, f: F) -> Result<Vec<RelayUrl>, Error>
    where
        F: Fn(&Relay3) -> bool,
    {
        Ok(GLOBALS
            .storage
            .filter_relays(|r| r.has_usage_bits(bits) && r.rank != 0 && !r.should_avoid() && f(r))?
            .iter()
            .map(|r| r.url.clone())
            .collect())
    }

    // Which relays are best for a reply to this event (used to find replies to this event)
    pub fn relays_for_reply(event: &Event) -> Result<Vec<RelayUrl>, Error> {
        let mut seen_on: Vec<RelayUrl> = GLOBALS
            .storage
            .get_event_seen_on_relay(event.id)?
            .drain(..)
            .map(|(url, _time)| url)
            .collect();

        let inbox: Vec<RelayUrl> = GLOBALS.storage.get_best_relays_fixed(event.pubkey, false)?;

        // Take all inbox relays, and up to 2 seen_on relays that aren't inbox relays
        let mut answer = inbox;
        let mut extra = 2;
        for url in seen_on.drain(..) {
            if extra == 0 {
                break;
            }
            if answer.contains(&url) {
                continue;
            }
            answer.push(url);
            extra -= 1;
        }

        Ok(answer)
    }

    // Which relays should an event be posted to (that it hasn't already been
    // seen on)?
    pub fn relays_for_event(event: &Event) -> Result<Vec<RelayUrl>, Error> {
        let mut relay_urls: Vec<RelayUrl> = Vec::new();

        // Get all of the relays that we write to
        let write_relay_urls: Vec<RelayUrl> = Relay3::choose_relay_urls(Relay3::WRITE, |_| true)?;
        relay_urls.extend(write_relay_urls);

        // Get 'read' relays for everybody tagged in the event.
        let mut tagged_pubkeys: Vec<PublicKey> = event
            .tags
            .iter()
            .filter_map(|t| {
                if let Ok((pubkey, _, _)) = t.parse_pubkey() {
                    Some(pubkey)
                } else {
                    None
                }
            })
            .collect();
        for pubkey in tagged_pubkeys.drain(..) {
            let best_relays: Vec<RelayUrl> =
                GLOBALS.storage.get_best_relays_fixed(pubkey, false)?;
            relay_urls.extend(best_relays);
        }

        // Remove all the 'seen_on' relays for this event
        let seen_on: Vec<RelayUrl> = GLOBALS
            .storage
            .get_event_seen_on_relay(event.id)?
            .iter()
            .map(|(url, _time)| url.to_owned())
            .collect();
        relay_urls.retain(|r| !seen_on.contains(r));

        relay_urls.sort();
        relay_urls.dedup();

        Ok(relay_urls)
    }
}
